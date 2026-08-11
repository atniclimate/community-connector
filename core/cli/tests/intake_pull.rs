//! `cn intake pull` puller battery (blueprint intake-relay.md section 8 step 7;
//! ADR-005 D1/D3/D4/D6). The relay control plane is injected behind a
//! [`RelayHttp`] mock and the D8 bundle behind a `fetch` closure, so the whole
//! flow - dedup arms, decrypt failures, consent recognition, staging, delete
//! discipline, reconciliation - runs with canned responses and no real server.
//! Envelopes are sealed to a fresh test keypair with `cn_ingest::seal`; every
//! fixture is synthetic and lives in a tempdir, committed nowhere (I1).

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use sha2::{Digest, Sha256};

use cn::intake::pull::{HttpResult, PullConfig, RelayHttp, execute_pull, parse_config};
use cn_ingest::{
    ConsentBlock, InnerPayload, Keypair, OuterEnvelope, PublicKey, RemoteContext, ReviewSidecar,
    fingerprint, generate_keypair, new_uuid_v7, seal, stage_remote_record,
};
use cn_model::Timestamp;

/// A synthetic 2026 instant used as the pull clock.
const NOW_MS: i64 = 1_786_000_000_000;
/// An arbitrary SHA-256-shaped consent digest (synthetic).
const DIGEST: &str = "ae5f7cc0d740726e81785919d1dbe6ad715895c94da278610da354d730bcd36c";
const PAGES: &str = "https://example.github.io/community-connector";
const RELAY: &str = "https://relay.example.workers.dev";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// One test's world: a queue root, the puller's keypair (+ its fingerprint), a
/// pinned config whose bundle is a single index.html, and the good bundle
/// responses for a `Verified` fetch.
struct Setup {
    dir: tempfile::TempDir,
    queue: PathBuf,
    kp: Keypair,
    fp: String,
    config: PullConfig,
    good_bundle: HashMap<String, Vec<u8>>,
}

fn setup(known_digests: &[&str]) -> Setup {
    let dir = tempfile::tempdir().expect("tempdir");
    let queue = dir.path().join("queue");
    std::fs::create_dir_all(&queue).expect("queue dir");

    let kp = generate_keypair();
    let fp = fingerprint(&kp.public).to_string();

    // A one-file pinned bundle so the D8 verification has something to check.
    let file_body = b"<html>form</html>".to_vec();
    let manifest = serde_json::json!({
        "files": [{ "path": "index.html", "bytes": file_body.len(), "sha256": sha256_hex(&file_body) }],
        "provenance": { "key_fingerprint": fp.as_str() },
    });
    let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
    let manifest_path = dir.path().join("dist.manifest.json");
    std::fs::write(&manifest_path, &manifest_bytes).unwrap();
    let manifest_hash = sha256_hex(&manifest_bytes);

    let config_json = serde_json::json!({
        "config_version": "0.1.0",
        "key_dir": dir.path().join("keys").to_str().unwrap(),
        "key_fingerprint_pin": fp.as_str(),
        "relay_origin": RELAY,
        "credential_path": dir.path().join("cred.txt").to_str().unwrap(),
        "pages_origin": PAGES,
        "manifest_path": manifest_path.to_str().unwrap(),
        "manifest_pin": {
            "manifest_hash": manifest_hash,
            "commit_sha": "abc123",
            "pinned_at": "2026-08-11T00:00:00Z",
            "pinned_by": "test",
        },
        "known_consent_digests": known_digests.to_vec(),
        "blob_ttl_seconds": 86400,
        "max_pull_interval_seconds": 3600,
    });
    let config = parse_config(&serde_json::to_vec(&config_json).unwrap()).expect("config parses");

    let mut good_bundle = HashMap::new();
    good_bundle.insert(format!("{PAGES}/index.html"), file_body);

    Setup {
        dir,
        queue,
        kp,
        fp,
        config,
        good_bundle,
    }
}

/// A `Verified`-path bundle fetch: serves the pinned files.
fn verified_fetch(s: &Setup) -> impl Fn(&str) -> HttpResult {
    let good = s.good_bundle.clone();
    move |url: &str| {
        good.get(url)
            .cloned()
            .map(|body| (200u16, body))
            .ok_or_else(|| format!("no route for {url}"))
    }
}

/// A `Degraded`-path bundle fetch: the origin never answers.
fn degraded_fetch() -> impl Fn(&str) -> HttpResult {
    |_url: &str| Err("connection refused".to_string())
}

fn make_inner(submission_id: &str, digest: &str) -> InnerPayload {
    InnerPayload {
        submission_version: "0.1.0".parse().unwrap(),
        submission_id: submission_id.to_string(),
        form_version: "0.1.0".to_string(),
        consent: ConsentBlock {
            consent_text_digest: digest.to_string(),
            consent_affirmed: true,
            consent_affirmed_at: "2026-08-11T00:00:00.000Z".to_string(),
        },
        captured_at: "2026-08-11T00:00:00.000Z".to_string(),
        kind: None,
        fields: BTreeMap::new(),
        extras: BTreeMap::new(),
    }
}

/// Seals `inner` to `seal_to` and wraps it in an outer envelope claiming
/// `recipient_fp` (STANDARD base64, matching D-083). Returns the exact bytes a
/// relay blob holds.
fn seal_outer(seal_to: &PublicKey, recipient_fp: &str, inner: &InnerPayload) -> Vec<u8> {
    let inner_bytes = serde_json::to_vec(inner).unwrap();
    let sealed = seal(&inner_bytes, seal_to);
    let outer = OuterEnvelope {
        intake_envelope_version: "0.1.0".parse().unwrap(),
        recipient_key_fingerprint: recipient_fp.to_string(),
        ciphertext: BASE64.encode(&sealed),
        extras: BTreeMap::new(),
    };
    serde_json::to_vec(&outer).unwrap()
}

/// Writes a staged remote record (+ pending sidecar) into `queue` directly, as
/// though a prior run had staged it - the fixture for the dedup tests.
fn prestage(queue: &Path, inner: &InnerPayload, receipt_id: &str, ciphertext_hash: &str, fp: &str) {
    let ctx = RemoteContext {
        claimed_fingerprint: fp.to_string(),
        envelope_version: "0.1.0".to_string(),
        key_used: fp.to_string(),
        receipt_id: receipt_id.to_string(),
        relay_received_at: None,
        pulled_at: Timestamp(1),
        ciphertext_hash: ciphertext_hash.to_string(),
    };
    let record = stage_remote_record(inner, ctx, new_uuid_v7(), Timestamp(1)).unwrap();
    let sidecar = ReviewSidecar::initial(&record).unwrap();
    std::fs::write(
        queue.join(format!("{}.record.json", record.record_id)),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .unwrap();
    std::fs::write(
        queue.join(format!("{}.sidecar.json", record.record_id)),
        serde_json::to_vec_pretty(&sidecar).unwrap(),
    )
    .unwrap();
}

fn read_one_record(queue: &Path) -> serde_json::Value {
    let path = std::fs::read_dir(queue)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| path.to_string_lossy().ends_with(".record.json"))
        .expect("a staged record file");
    serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap()
}

fn record_count(queue: &Path) -> usize {
    std::fs::read_dir(queue)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().to_string_lossy().ends_with(".record.json"))
        .count()
}

/// An injectable relay: serves `/receipts` from `order` (id, arrived_at,
/// has_ledger; has_blob computed live from the blob store), `/blob/:id` from the
/// store, and records deletes.
struct MockRelay {
    order: Vec<(String, Option<String>, bool)>,
    blobs: RefCell<HashMap<String, Vec<u8>>>,
    deletes: RefCell<Vec<String>>,
    get_calls: RefCell<usize>,
    /// Forward-compat pagination cursor the `/receipts` body reports (None -> the
    /// body carries `"cursor": null`, the no-more-pages case).
    cursor: Option<String>,
}

impl MockRelay {
    fn new() -> Self {
        Self {
            order: Vec::new(),
            blobs: RefCell::new(HashMap::new()),
            deletes: RefCell::new(Vec::new()),
            get_calls: RefCell::new(0),
            cursor: None,
        }
    }

    fn with_receipt(mut self, id: &str, arrived: Option<&str>, blob: Option<Vec<u8>>) -> Self {
        self.order
            .push((id.to_string(), arrived.map(str::to_string), true));
        if let Some(bytes) = blob {
            self.blobs.borrow_mut().insert(id.to_string(), bytes);
        }
        self
    }

    /// Makes `/receipts` report a non-null pagination cursor (a relay that
    /// paginated), which the puller must refuse to follow silently.
    fn with_cursor(mut self, cursor: &str) -> Self {
        self.cursor = Some(cursor.to_string());
        self
    }

    /// A receipt whose blob is present but ledger absent (the D6 orphan case).
    fn with_orphan(mut self, id: &str, blob: Vec<u8>) -> Self {
        self.order.push((id.to_string(), None, false));
        self.blobs.borrow_mut().insert(id.to_string(), blob);
        self
    }
}

fn blob_id(url: &str) -> String {
    url.rsplit('/').next().unwrap_or_default().to_string()
}

impl RelayHttp for MockRelay {
    fn get(&self, url: &str) -> HttpResult {
        *self.get_calls.borrow_mut() += 1;
        if url.ends_with("/receipts") {
            let receipts: Vec<serde_json::Value> = self
                .order
                .iter()
                .map(|(id, arrived, has_ledger)| {
                    let mut row = serde_json::Map::new();
                    row.insert(
                        "receipt_id".to_string(),
                        serde_json::Value::String(id.clone()),
                    );
                    row.insert(
                        "has_ledger".to_string(),
                        serde_json::Value::Bool(*has_ledger),
                    );
                    row.insert(
                        "has_blob".to_string(),
                        serde_json::Value::Bool(self.blobs.borrow().contains_key(id)),
                    );
                    if let Some(arrived) = arrived {
                        row.insert(
                            "arrived_at".to_string(),
                            serde_json::Value::String(arrived.clone()),
                        );
                    }
                    serde_json::Value::Object(row)
                })
                .collect();
            let body = serde_json::json!({ "receipts": receipts, "cursor": self.cursor.clone() });
            return Ok((200, serde_json::to_vec(&body).unwrap()));
        }
        match self.blobs.borrow().get(&blob_id(url)) {
            Some(bytes) => Ok((200, bytes.clone())),
            None => Ok((404, Vec::new())),
        }
    }

    fn delete(&self, url: &str) -> HttpResult {
        let id = blob_id(url);
        let existed = self.blobs.borrow_mut().remove(&id).is_some();
        self.deletes.borrow_mut().push(id);
        Ok((if existed { 200 } else { 204 }, Vec::new()))
    }
}

// 1. Happy path: one sealed envelope -> one staged remote record, blob deleted.
#[test]
fn happy_path_stages_remote_record() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-happy", DIGEST);
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    let ciphertext_hash = sha256_hex(&outer);
    let mock =
        MockRelay::new().with_receipt("rcpt-happy", Some("2026-08-11T00:00:00.000Z"), Some(outer));

    let summary = execute_pull(
        &s.config,
        &s.kp,
        &s.queue,
        &mock,
        verified_fetch(&s),
        NOW_MS,
    )
    .expect("pull");
    let report = &summary.report;
    assert_eq!(
        report["preconditions"]["bundle_check"]["status"],
        "verified"
    );
    assert_eq!(report["main_loop"]["staged"], 1);
    assert_eq!(report["main_loop"]["blobs_deleted"], 1);
    assert_eq!(report["main_loop"]["errors"].as_array().unwrap().len(), 0);
    assert_eq!(report["reconciliation"]["staged"], 1);
    assert!(!summary.failure);
    assert_eq!(mock.deletes.borrow().len(), 1);

    let record = read_one_record(&s.queue);
    assert_eq!(record["source"]["kind"], "remote");
    assert_eq!(record["source"]["receipt_id"], "rcpt-happy");
    assert_eq!(record["source"]["key_used"], s.fp);
    assert_eq!(record["source"]["ciphertext_hash"], ciphertext_hash);
    assert!(
        !record["source"]["relay_received_at"].is_null(),
        "arrived_at parsed into a timestamp"
    );
    assert_eq!(record["payload"]["submission_id"], "sub-happy");
}

// 2. Transport replay: same (receipt_id, ciphertext_hash) already staged.
#[test]
fn transport_replay_is_a_noop_and_retains_the_blob() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-replay", DIGEST);
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    let ciphertext_hash = sha256_hex(&outer);
    prestage(&s.queue, &inner, "rcpt-replay", &ciphertext_hash, &s.fp);
    let mock = MockRelay::new().with_receipt("rcpt-replay", None, Some(outer));

    let summary =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(summary.report["main_loop"]["transport_replays"], 1);
    assert_eq!(summary.report["main_loop"]["staged"], 0);
    assert_eq!(summary.report["main_loop"]["blobs_deleted"], 0);
    assert!(
        mock.deletes.borrow().is_empty(),
        "a replay never deletes the relay blob"
    );
    assert!(!summary.failure);
}

// 3. Semantic replay: same (submission_id, payload_hash), different receipt.
#[test]
fn semantic_replay_is_a_noop() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-sem", DIGEST);
    prestage(
        &s.queue,
        &inner,
        "rcpt-old",
        "an-old-ciphertext-hash",
        &s.fp,
    );
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    let mock = MockRelay::new().with_receipt("rcpt-new", None, Some(outer));

    let summary =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(summary.report["main_loop"]["semantic_replays"], 1);
    assert_eq!(summary.report["main_loop"]["staged"], 0);
    assert_eq!(summary.report["main_loop"]["blobs_deleted"], 0);
}

// 4. Fingerprint mismatch: envelope addressed to another key -> retained.
#[test]
fn fingerprint_mismatch_retains_the_blob() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-fp", DIGEST);
    let outer = seal_outer(
        &s.kp.public,
        "0000-0000-0000-0000-0000-0000-0000-0000",
        &inner,
    );
    let mock = MockRelay::new().with_receipt("rcpt-fp", None, Some(outer));

    let summary =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(summary.report["main_loop"]["staged"], 0);
    let errors = summary.report["main_loop"]["errors"].as_array().unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.as_str().unwrap().contains("addressed to"))
    );
    assert!(
        mock.deletes.borrow().is_empty(),
        "a fingerprint mismatch never deletes"
    );
    assert!(summary.failure);
}

// 5. Decrypt failure: sealed to another key -> open fails, blob retained.
#[test]
fn decrypt_failure_retains_the_blob() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-dec", DIGEST);
    // Sealed to a DIFFERENT key but claiming our fingerprint: passes the
    // fingerprint check, then fails to open.
    let other = generate_keypair();
    let outer = seal_outer(&other.public, &s.fp, &inner);
    let mock = MockRelay::new().with_receipt("rcpt-dec", None, Some(outer));

    let summary =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(summary.report["main_loop"]["staged"], 0);
    let errors = summary.report["main_loop"]["errors"].as_array().unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e.as_str().unwrap().contains("open failed"))
    );
    assert!(mock.deletes.borrow().is_empty());
    assert!(summary.failure);
}

// 6. Unknown consent digest: a warning, still staged.
#[test]
fn unknown_consent_digest_warns_but_stages() {
    let s = setup(&[]); // no known digests
    let inner = make_inner("sub-consent", DIGEST);
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    let mock = MockRelay::new().with_receipt("rcpt-consent", None, Some(outer));

    let summary =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(summary.report["main_loop"]["staged"], 1);
    let warnings = summary.report["main_loop"]["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w.as_str().unwrap().contains("consent digest"))
    );
    assert!(
        !summary.failure,
        "an unknown consent digest is a warning, not a failure"
    );
}

// 7. Bundle verification failed: halt before any receipt is touched.
#[test]
fn bundle_failed_halts_before_processing() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-x", DIGEST);
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    let mock = MockRelay::new().with_receipt("rcpt-x", None, Some(outer));
    // A nonexistent pinned manifest -> verify_bundle -> Failed.
    let mut config = s.config.clone();
    config.manifest_path = s.dir.path().join("no-manifest.json");

    let summary =
        execute_pull(&config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(
        summary.report["preconditions"]["bundle_check"]["status"],
        "failed"
    );
    assert!(
        summary.report.get("main_loop").is_none(),
        "no receipts processed after a failed bundle"
    );
    assert_eq!(
        *mock.get_calls.borrow(),
        0,
        "the relay is never contacted on a bundle halt"
    );
    assert!(summary.failure);
}

// 8. Bundle degraded: proceed with a warning.
#[test]
fn bundle_degraded_proceeds() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-deg", DIGEST);
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    let mock = MockRelay::new().with_receipt("rcpt-deg", None, Some(outer));

    let summary =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(
        summary.report["preconditions"]["bundle_check"]["status"],
        "degraded"
    );
    assert_eq!(
        summary.report["main_loop"]["staged"], 1,
        "degraded proceeds"
    );
}

// 9. Run report structure: every expected top-level field is present.
#[test]
fn report_has_all_expected_fields() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-struct", DIGEST);
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    let mock = MockRelay::new().with_receipt("rcpt-struct", None, Some(outer));

    let summary = execute_pull(
        &s.config,
        &s.kp,
        &s.queue,
        &mock,
        verified_fetch(&s),
        NOW_MS,
    )
    .expect("pull");
    let report = &summary.report;
    for key in ["preconditions", "main_loop", "reconciliation"] {
        assert!(report.get(key).is_some(), "missing top-level {key}");
    }
    for key in ["config_version", "key_pin", "queue_root", "bundle_check"] {
        assert!(
            report["preconditions"].get(key).is_some(),
            "missing preconditions.{key}"
        );
    }
    for key in [
        "receipts_listed",
        "receipts_with_blob",
        "staged",
        "transport_replays",
        "transport_conflicts",
        "semantic_replays",
        "semantic_conflicts",
        "orphan_blobs",
        "errors",
        "warnings",
        "blobs_deleted",
    ] {
        assert!(
            report["main_loop"].get(key).is_some(),
            "missing main_loop.{key}"
        );
    }
    for key in [
        "total_receipts",
        "staged",
        "deleted_by_me",
        "unpulled",
        "integrity_alert",
        "expired",
        "oldest_unpulled_age_secs",
        "half_ttl_warning",
        "warnings",
    ] {
        assert!(
            report["reconciliation"].get(key).is_some(),
            "missing reconciliation.{key}"
        );
    }
    assert_eq!(report["preconditions"]["key_pin"], "match");
}

// 10. Orphan blob (has_blob, no has_ledger): D6 anomaly flagged, still staged.
#[test]
fn orphan_blob_is_flagged_but_still_staged() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-orphan", DIGEST);
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    let mock = MockRelay::new().with_orphan("rcpt-orphan", outer);

    let summary =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(
        summary.report["main_loop"]["orphan_blobs"], 1,
        "the orphan (blob without ledger) is counted"
    );
    let warnings = summary.report["main_loop"]["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w.as_str().unwrap().contains("ORPHAN BLOB")),
        "the orphan anomaly is surfaced as a warning"
    );
    // The intact ciphertext is still pulled and staged - no consented data lost.
    assert_eq!(summary.report["main_loop"]["staged"], 1);
}

// 11. A present-but-unparseable arrived_at is diagnosed, not silently degraded.
#[test]
fn unparseable_arrived_at_is_diagnosed_in_reconciliation() {
    let s = setup(&[DIGEST]);
    // Blob-absent (None) receipt with a garbage timestamp: it is skipped by the
    // main loop and reaches reconciliation, where the age cannot be computed.
    let mock = MockRelay::new().with_receipt("rcpt-badts", Some("not-a-timestamp"), None);

    let summary =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    let warnings = summary.report["reconciliation"]["warnings"]
        .as_array()
        .unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w.as_str().unwrap().contains("did not parse")),
        "the unparseable arrived_at is surfaced, not silently degraded"
    );
    // Blob absent + no local record + unknown age -> IntegrityAlert (D-082).
    assert_eq!(summary.report["reconciliation"]["integrity_alert"], 1);
}

// 12. The envelope cap is a config knob: a submission over the 8 KiB default is
// rejected under the default but stages once the cap is raised (F6 - a puller
// cap below the relay's would silently drop a valid consented submission).
#[test]
fn envelope_cap_is_configurable() {
    let s = setup(&[DIGEST]);
    let mut inner = make_inner("sub-big", DIGEST);
    inner.fields.insert(
        "bio".to_string(),
        serde_json::Value::String("x".repeat(12_000)),
    );
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    assert!(
        outer.len() > 8192,
        "the test envelope must exceed the 8 KiB default cap"
    );

    // Default cap (8 KiB): rejected as oversized, not staged.
    let mock = MockRelay::new().with_receipt("rcpt-big", None, Some(outer.clone()));
    let default_run =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(
        default_run.report["main_loop"]["staged"], 0,
        "oversized under the default cap"
    );
    assert!(
        default_run.report["main_loop"]["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e.as_str().unwrap().contains("outer envelope parse failed"))
    );

    // Raised cap: the same submission stages.
    let mut raised = s.config.clone();
    raised.max_envelope_bytes = 65_536;
    let mock2 = MockRelay::new().with_receipt("rcpt-big", None, Some(outer));
    let raised_run =
        execute_pull(&raised, &s.kp, &s.queue, &mock2, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(
        raised_run.report["main_loop"]["staged"], 1,
        "raising the cap stages the larger submission"
    );
}

// 13. Transport conflict: same receipt id, different ciphertext -> stage BOTH,
// retain the blob, loud, failure exit (ADR-005 D4).
#[test]
fn transport_conflict_stages_both_and_retains_blob() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-tc", DIGEST);
    // A prior record for the SAME receipt id but a DIFFERENT ciphertext hash.
    prestage(
        &s.queue,
        &inner,
        "rcpt-tc",
        "a-different-ciphertext-hash",
        &s.fp,
    );
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    let mock = MockRelay::new().with_receipt("rcpt-tc", None, Some(outer));

    let summary =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(summary.report["main_loop"]["transport_conflicts"], 1);
    assert_eq!(
        summary.report["main_loop"]["staged"], 1,
        "the new copy is staged alongside the existing"
    );
    assert!(
        mock.deletes.borrow().is_empty(),
        "a transport conflict retains the relay blob as evidence"
    );
    assert!(summary.failure);
    assert_eq!(record_count(&s.queue), 2, "both copies on disk");
}

// 14. Semantic conflict: same submission id, different bytes -> stage BOTH,
// retain the blob, loud, failure exit (ADR-005 D4).
#[test]
fn semantic_conflict_stages_both_and_retains_blob() {
    let s = setup(&[DIGEST]);
    let existing = make_inner("sub-sc", DIGEST);
    // A prior record for a DIFFERENT receipt but the SAME submission id.
    prestage(
        &s.queue,
        &existing,
        "rcpt-old-sc",
        "an-old-ciphertext-hash",
        &s.fp,
    );
    // The new envelope shares the submission id but has different bytes.
    let mut fresh = make_inner("sub-sc", DIGEST);
    fresh.fields.insert(
        "changed".to_string(),
        serde_json::Value::String("v2".to_string()),
    );
    let outer = seal_outer(&s.kp.public, &s.fp, &fresh);
    let mock = MockRelay::new().with_receipt("rcpt-new-sc", None, Some(outer));

    let summary =
        execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(summary.report["main_loop"]["semantic_conflicts"], 1);
    assert_eq!(summary.report["main_loop"]["staged"], 1);
    assert!(
        mock.deletes.borrow().is_empty(),
        "a semantic conflict retains the relay blob as evidence"
    );
    assert!(summary.failure);
    assert_eq!(record_count(&s.queue), 2);
}

// 15. Reconciliation Expired arm: a blob-absent receipt past TTL + margin.
#[test]
fn reconciliation_classifies_expired() {
    let s = setup(&[DIGEST]);
    let mut cfg = s.config.clone();
    cfg.blob_ttl_seconds = 2; // ttl + margin = 302s; a 2020 arrival is far past it
    let mock = MockRelay::new().with_receipt("rcpt-exp", Some("2020-01-01T00:00:00.000Z"), None);

    let summary =
        execute_pull(&cfg, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(summary.report["reconciliation"]["total_receipts"], 1);
    assert_eq!(summary.report["reconciliation"]["expired"], 1);
    assert_eq!(summary.report["reconciliation"]["integrity_alert"], 0);
}

// 16. Reconciliation Unpulled arm + half-TTL warning + oldest-age tracking.
#[test]
fn reconciliation_classifies_unpulled_with_half_ttl_warning() {
    let s = setup(&[DIGEST]);
    let mut cfg = s.config.clone();
    cfg.blob_ttl_seconds = 2; // half_ttl = 1s
    let inner = make_inner("sub-unp", DIGEST);
    // Addressed to another key: retained (never staged), blob still present.
    let outer = seal_outer(
        &s.kp.public,
        "0000-0000-0000-0000-0000-0000-0000-0000",
        &inner,
    );
    let mock =
        MockRelay::new().with_receipt("rcpt-unp", Some("2020-01-01T00:00:00.000Z"), Some(outer));

    let summary =
        execute_pull(&cfg, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS).expect("pull");
    assert_eq!(summary.report["reconciliation"]["unpulled"], 1);
    assert_eq!(summary.report["reconciliation"]["half_ttl_warning"], true);
    assert!(
        summary.report["reconciliation"]["oldest_unpulled_age_secs"]
            .as_i64()
            .unwrap()
            > 1
    );
}

// 17. Pagination cursor (R2-3): a non-null `/receipts` cursor means the relay
// paginated and later pages exist. The puller does not follow cursors, so it
// halts LOUDLY rather than silently processing only page 1 and dropping the
// rest.
#[test]
fn nonnull_receipts_cursor_halts_loudly() {
    let s = setup(&[DIGEST]);
    let inner = make_inner("sub-cursor", DIGEST);
    let outer = seal_outer(&s.kp.public, &s.fp, &inner);
    let mock = MockRelay::new()
        .with_receipt("rcpt-cursor", None, Some(outer))
        .with_cursor("page-2-token");

    let err = match execute_pull(&s.config, &s.kp, &s.queue, &mock, degraded_fetch(), NOW_MS) {
        Ok(_) => panic!("a paginated listing must halt the run, not succeed"),
        Err(err) => err,
    };
    assert!(
        err.contains("cursor") && err.contains("page 1"),
        "the halt names the pagination cursor and the page-1 risk: {err}"
    );
    // The run halted at listing, before any receipt: nothing staged.
    assert_eq!(record_count(&s.queue), 0);
}

// 18. Config guard (F7): a max_envelope_bytes above the fixed HTTP read cap is
// rejected at parse time (a cap above the read cap would let an oversized body
// be truncated before the size gate, turning a valid submission into an
// unstageable loud error). At the cap it parses; one byte over is rejected.
#[test]
fn config_rejects_envelope_cap_above_read_cap() {
    const READ_CAP: usize = 8 * 1024 * 1024;
    let base = serde_json::json!({
        "config_version": "0.1.0",
        "key_dir": "keys",
        "key_fingerprint_pin": "3f9a-0000-0000-0000-0000-0000-0000-0000",
        "relay_origin": RELAY,
        "credential_path": "cred.txt",
        "pages_origin": PAGES,
        "manifest_path": "dist.manifest.json",
        "manifest_pin": {
            "manifest_hash": "deadbeef",
            "commit_sha": "abc123",
            "pinned_at": "2026-08-11T00:00:00Z",
            "pinned_by": "test",
        },
        "known_consent_digests": [DIGEST],
        "blob_ttl_seconds": 86400,
        "max_pull_interval_seconds": 3600,
    });

    // Exactly at the read cap: accepted.
    let mut at_cap = base.clone();
    at_cap["max_envelope_bytes"] = serde_json::json!(READ_CAP);
    assert!(
        parse_config(&serde_json::to_vec(&at_cap).unwrap()).is_ok(),
        "a cap equal to the read cap is allowed"
    );

    // One byte over: rejected loudly, naming the knob and the read cap.
    let mut over_cap = base;
    over_cap["max_envelope_bytes"] = serde_json::json!(READ_CAP + 1);
    let err = parse_config(&serde_json::to_vec(&over_cap).unwrap())
        .expect_err("a cap above the read cap must be rejected");
    assert!(
        err.contains("max_envelope_bytes") && err.contains("read cap"),
        "the rejection names the offending knob and the read cap: {err}"
    );
}
