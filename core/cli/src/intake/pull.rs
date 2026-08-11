//! `cn intake pull`: the native remote puller (blueprint
//! docs/blueprints/intake-relay.md section 6; ADR-005 D1/D3/D4/D6). This is the
//! ONE component that crosses the module fence (D1): it makes HTTPS requests to
//! the Worker relay (control plane) and the Pages origin (bundle check). All
//! network I/O lives here, in the CLI crate; the trust decisions - dedup,
//! reconciliation classification, consent recognition, sealed-box open, staging
//! conversion - stay in cn-ingest (I2). The puller calls them.
//!
//! One run: verify the pinned bundle (D8), list receipts, and for each blob
//! transport-dedup -> fetch -> decrypt -> consent-check -> semantic-dedup ->
//! stage a `QueueRecord` with `SubmissionSource::Remote` (the SAME format the
//! facilitator wizard and `cn intake apply` already consume) -> delete the relay
//! blob ONLY after verified staging. Then reconcile every ledger receipt by the
//! D6 precedence and emit an I12 run report (JSON on stdout).
//!
//! HTTP is injected behind [`RelayHttp`] and a `fetch` closure so the whole
//! decision table runs in tests with canned responses - no real server. The
//! production path builds a redirect-disabled, status-tolerant `ureq` agent
//! (pure-Rust rustls TLS, vendored roots).

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest as _, Sha256};

use cn_ingest::{
    ConsentDigestVerdict, DEFAULT_MAX_ENVELOPE_BYTES, DedupKey, DedupVerdict, InnerPayload,
    Keypair, OuterEnvelope, QueueRecord, ReceiptClass, ReceiptObservation, RemoteContext,
    ReviewSidecar, SubmissionSource, canonical_digest, check_consent_digest, classify_dedup,
    classify_receipt, fingerprint, new_uuid_v7, parse_public_key, parse_secret_key,
};
use cn_model::Timestamp;

use super::bundle::{self, BundleResult};
use super::queue::{self, QueuePaths};
use crate::Exit;

/// Current puller-config format version (I7; unknown MAJOR rejected loudly).
const PULL_CONFIG_VERSION: &str = "0.1.0";

/// Extra seconds added to the blob TTL before a blob-absent receipt is treated
/// as cleanly expired rather than an integrity alert. KV is eventually
/// consistent (ADR-005 D6): a receipt that JUST crossed its TTL may still be
/// briefly listed, or a just-written ledger may briefly precede its blob, so a
/// margin keeps the reconciliation from crying alert during that window. Five
/// minutes is generous for KV convergence at pilot scale.
const CONSISTENCY_MARGIN_SECS: i64 = 300;

/// Response-body read cap. Blobs are single-digit KB (ADR-005 D6), but bundle
/// assets (the form's JS chunk) can be larger; 8 MiB bounds memory while
/// comfortably covering both.
const MAX_RESPONSE_BYTES: u64 = 8 * 1024 * 1024;

/// Off-repo puller config (blueprint 6.3, extended with `manifest_path`). Lives
/// in the facilitator ops directory alongside the keys and queue root; passed
/// via `--config <path>`.
#[derive(Debug, Clone, Deserialize)]
pub struct PullConfig {
    /// Config format version (I7).
    pub config_version: semver::Version,
    /// Directory holding `public.json` and `secret.json`.
    pub key_dir: PathBuf,
    /// The `3f9a-...` fingerprint the puller must hold (pin).
    pub key_fingerprint_pin: String,
    /// Worker relay origin (control plane).
    pub relay_origin: String,
    /// File holding the bearer token (plain text, trailing whitespace trimmed).
    pub credential_path: PathBuf,
    /// Pages origin serving the deployed form (bundle check).
    pub pages_origin: String,
    /// Local path to the ceremony-pinned D8 manifest FILE. The manifest is not
    /// deployed (D8 no-deploy rule), so its content is read locally; `manifest_
    /// pin.manifest_hash` guards this file against substitution. This field
    /// extends blueprint 6.3, which pins only the hash (see D-085).
    pub manifest_path: PathBuf,
    /// The ceremony pin of the manifest above.
    pub manifest_pin: ManifestPin,
    /// Consent-text digests the puller recognizes as currently deployed.
    pub known_consent_digests: Vec<String>,
    /// Blob TTL (seconds); reconciliation's expiry threshold baseline.
    pub blob_ttl_seconds: i64,
    /// Maximum pull interval (seconds); ceremony/ops provenance.
    pub max_pull_interval_seconds: i64,
}

/// The off-repo ceremony pin of the D8 manifest (blueprint 6.3 `manifest_pin`).
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestPin {
    /// SHA-256 over the pinned manifest file's exact bytes.
    pub manifest_hash: String,
    /// Git commit the form was built from.
    pub commit_sha: String,
    /// When the manifest was pinned.
    pub pinned_at: String,
    /// Who pinned it.
    pub pinned_by: String,
}

/// Parses and version-checks a puller config. Unknown MAJOR is rejected loudly
/// (I7); everything downstream trusts the version line.
pub fn parse_config(bytes: &[u8]) -> Result<PullConfig, String> {
    let config: PullConfig = serde_json::from_slice(bytes)
        .map_err(|err| format!("puller config does not parse: {err}"))?;
    let current = semver::Version::parse(PULL_CONFIG_VERSION).expect("valid const");
    if config.config_version.major != current.major {
        return Err(format!(
            "unknown puller config major version {} (I7)",
            config.config_version
        ));
    }
    Ok(config)
}

/// One HTTP call's result: `(status, body)` or a transport-error string. The
/// shared shape for the relay client and the bundle fetch closure, so a real
/// `ureq` caller and a test mock are interchangeable.
pub type HttpResult = Result<(u16, Vec<u8>), String>;

/// The relay control-plane surface the puller needs, injected for testability.
/// The implementation owns the origin and bearer credential; callers pass full
/// URLs. Object-safe (used as `&dyn RelayHttp`).
pub trait RelayHttp {
    /// `GET url` with the relay bearer credential attached.
    fn get(&self, url: &str) -> HttpResult;
    /// `DELETE url` with the relay bearer credential attached.
    fn delete(&self, url: &str) -> HttpResult;
}

/// The result of one pull run: the I12 report (already serialized) and whether
/// the run needs facilitator attention (non-zero exit).
pub struct PullSummary {
    pub report: serde_json::Value,
    pub failure: bool,
}

/// A parsed receipt row from `GET /receipts` (relay `receipts.ts`). Only the
/// fields the puller consumes; the rest are ignored.
#[derive(Debug, Clone, Deserialize)]
struct ReceiptRow {
    receipt_id: String,
    #[serde(default)]
    has_blob: bool,
    #[serde(default)]
    arrived_at: Option<String>,
}

/// The `GET /receipts` envelope shape.
#[derive(Debug, Deserialize)]
struct ReceiptListing {
    #[serde(default)]
    receipts: Vec<ReceiptRow>,
}

/// Main-loop tallies (I12 report `main_loop`).
#[derive(Debug, Default, Serialize)]
struct MainLoop {
    receipts_listed: usize,
    receipts_with_blob: usize,
    staged: usize,
    transport_replays: usize,
    transport_conflicts: usize,
    semantic_replays: usize,
    semantic_conflicts: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
    blobs_deleted: usize,
}

/// Reconciliation tallies (I12 report `reconciliation`).
#[derive(Debug, Default, Serialize)]
struct Reconciliation {
    total_receipts: usize,
    staged: usize,
    deleted_by_me: usize,
    unpulled: usize,
    integrity_alert: usize,
    expired: usize,
    oldest_unpulled_age_secs: Option<i64>,
    half_ttl_warning: bool,
}

/// Runs one pull against injected HTTP (the testable core; `run` wires the real
/// clients). Preconditions: queue-root safety + lock, key-pin re-assertion,
/// bundle verification. A `Failed` bundle halts before any receipt is touched
/// (decrypt nothing, stage nothing, delete nothing); `Degraded` proceeds with a
/// recorded warning; `Verified` proceeds. `keypair` is assumed already loaded
/// and pin-checked by the caller.
pub fn execute_pull(
    config: &PullConfig,
    keypair: &Keypair,
    queue_root: &Path,
    relay: &dyn RelayHttp,
    bundle_fetch: impl Fn(&str) -> HttpResult,
    now_ms: i64,
) -> Result<PullSummary, String> {
    // Preconditions 3-4: queue root outside any worktree/cloud-sync, single
    // native-mutator lock (shared with `cn intake apply`). Held for the whole
    // run via `_lock`.
    let root = queue::refuse_unsafe_root(queue_root)?;
    let paths = QueuePaths::new(&root);
    let _lock = queue::acquire_lock(&paths)?;
    run_locked(config, keypair, &paths, relay, bundle_fetch, now_ms)
}

/// The pull run once the queue root is validated and the lock is held (blueprint
/// 6.1 preconditions 5-7 + the main loop + reconciliation). Split from
/// [`execute_pull`] so `run` can acquire the lock BEFORE prompting for the key
/// passphrase - an unsafe root or a held lock must fail fast, never after the
/// operator has typed a passphrase or the secret key has been decrypted into
/// memory (blueprint 6.1 precondition order). Tests still exercise the whole
/// path through `execute_pull`.
fn run_locked(
    config: &PullConfig,
    keypair: &Keypair,
    paths: &QueuePaths,
    relay: &dyn RelayHttp,
    bundle_fetch: impl Fn(&str) -> HttpResult,
    now_ms: i64,
) -> Result<PullSummary, String> {
    // Precondition 5 re-asserted: the loaded key must match the pin. `run`
    // already checked the on-disk public key; re-checking the constructed pair
    // keeps the report field honest and lets the core be tested without file I/O.
    let key_fp = fingerprint(&keypair.public).to_string();
    if key_fp != config.key_fingerprint_pin {
        return Err(format!(
            "loaded key fingerprint {key_fp} does not match the pinned fingerprint {} (I3)",
            config.key_fingerprint_pin
        ));
    }

    // Precondition 7: bundle verification (D8).
    let bundle = bundle::verify_bundle(config, bundle_fetch);
    let preconditions = json!({
        "config_version": config.config_version.to_string(),
        "key_pin": "match",
        "queue_root": paths.root().display().to_string(),
        "bundle_check": serde_json::to_value(&bundle).map_err(|err| err.to_string())?,
    });

    if let BundleResult::Failed { .. } = bundle {
        // Halt: the deployed bundle is compromised or misdeployed. Nothing is
        // decrypted, staged, or deleted (blueprint 6.1 precondition 4).
        return Ok(PullSummary {
            report: json!({ "preconditions": preconditions }),
            failure: true,
        });
    }

    // Main loop.
    let mut main = MainLoop::default();
    let mut existing_keys = scan_dedup_keys(paths)?;
    let mut delete_journal: HashSet<String> = HashSet::new();

    let listing = fetch_receipts(relay, config)?;
    main.receipts_listed = listing.len();
    for row in &listing {
        if !row.has_blob {
            continue;
        }
        main.receipts_with_blob += 1;
        process_receipt(
            config,
            keypair,
            paths,
            relay,
            &key_fp,
            now_ms,
            row,
            &mut existing_keys,
            &mut delete_journal,
            &mut main,
        )?;
    }

    // Post-loop reconciliation (blueprint 6.1 post-loop; D6 precedence).
    let recon = reconcile(config, paths, relay, now_ms, &delete_journal)?;

    let failure = !main.errors.is_empty()
        || main.transport_conflicts > 0
        || main.semantic_conflicts > 0
        || recon.integrity_alert > 0;

    let report = json!({
        "preconditions": preconditions,
        "main_loop": serde_json::to_value(&main).map_err(|err| err.to_string())?,
        "reconciliation": serde_json::to_value(&recon).map_err(|err| err.to_string())?,
    });
    Ok(PullSummary { report, failure })
}

/// `GET {relay}/receipts` -> parsed rows. A 404 is a CREDENTIAL problem (the
/// endpoint always returns data when authed, even an empty list; D6 no-existence
/// oracle), so it halts the run; any other non-200 halts too.
fn fetch_receipts(relay: &dyn RelayHttp, config: &PullConfig) -> Result<Vec<ReceiptRow>, String> {
    let url = format!("{}/receipts", config.relay_origin.trim_end_matches('/'));
    let (status, body) = relay
        .get(&url)
        .map_err(|err| format!("GET /receipts transport error: {err}"))?;
    match status {
        200 => {
            let listing: ReceiptListing = serde_json::from_slice(&body)
                .map_err(|err| format!("GET /receipts returned unparseable JSON: {err}"))?;
            Ok(listing.receipts)
        }
        404 => Err(
            "GET /receipts returned 404 - treat as a credential problem (the endpoint \
                    always returns data when authed, even an empty list); halting (I3)"
                .to_string(),
        ),
        other => Err(format!(
            "GET /receipts returned unexpected status {other}; halting (I3)"
        )),
    }
}

/// Processes one receipt with a blob: fetch -> transport dedup -> decrypt ->
/// consent -> semantic dedup -> stage -> delete. Per-receipt faults are LOUD but
/// local: the blob is RETAINED on the relay and the loop continues (blueprint
/// 6.1); only a genuinely unexpected internal failure returns `Err` (halting).
#[allow(clippy::too_many_arguments)]
fn process_receipt(
    config: &PullConfig,
    keypair: &Keypair,
    paths: &QueuePaths,
    relay: &dyn RelayHttp,
    key_fp: &str,
    now_ms: i64,
    row: &ReceiptRow,
    existing_keys: &mut Vec<DedupKey>,
    delete_journal: &mut HashSet<String>,
    main: &mut MainLoop,
) -> Result<(), String> {
    let receipt_id = &row.receipt_id;
    let origin = config.relay_origin.trim_end_matches('/');

    // (a) fetch the blob.
    let (status, body) = match relay.get(&format!("{origin}/blob/{receipt_id}")) {
        Ok(response) => response,
        Err(err) => {
            main.errors.push(format!(
                "receipt {receipt_id}: GET blob transport error ({err}); blob retained"
            ));
            return Ok(());
        }
    };
    match status {
        200 => {}
        404 => {
            // Blob gone (expired or already pulled): not an error (D6 indistinct
            // 404). Skip; the reconciliation pass classifies it.
            main.warnings
                .push(format!("receipt {receipt_id}: blob gone (404); skipped"));
            return Ok(());
        }
        other => {
            main.errors.push(format!(
                "receipt {receipt_id}: GET blob returned status {other}; blob retained"
            ));
            return Ok(());
        }
    }

    // (b) ciphertext hash over the RAW body, before base64 decode.
    let ciphertext_hash = sha256_hex(&body);

    // (c) transport dedup (pre-decrypt): the only two facts known so far.
    let mut delete_allowed = true;
    match classify_dedup(
        &DedupKey::transport(receipt_id, &ciphertext_hash),
        existing_keys,
    ) {
        DedupVerdict::TransportReplay => {
            // Already staged from a prior run; recorded no-op. NOT deleted - a
            // replay proves nothing about whether the earlier delete landed.
            main.transport_replays += 1;
            return Ok(());
        }
        DedupVerdict::TransportConflict => {
            // Same receipt id, DIFFERENT ciphertext: relay-side substitution or
            // a platform fault. Stage the new copy alongside the existing and
            // RETAIN the relay blob (ADR-005 D4, I3). Skip semantic dedup - the
            // transport conflict is already the disposition.
            main.transport_conflicts += 1;
            main.errors.push(format!(
                "receipt {receipt_id}: TRANSPORT CONFLICT - same receipt id with a different \
                 ciphertext hash; staging the new copy alongside the existing and RETAINING the \
                 relay blob (ADR-005 D4, I3)"
            ));
            delete_allowed = false;
        }
        DedupVerdict::Fresh => {}
        other => {
            main.errors.push(format!(
                "receipt {receipt_id}: impossible transport dedup verdict {other:?}; blob retained"
            ));
            return Ok(());
        }
    }
    let is_transport_conflict = !delete_allowed;

    // (d) parse the outer envelope (size already relay-capped; re-checked here).
    let outer = match OuterEnvelope::parse(&body, DEFAULT_MAX_ENVELOPE_BYTES) {
        Ok(outer) => outer,
        Err(err) => {
            main.errors.push(format!(
                "receipt {receipt_id}: outer envelope parse failed ({err}); blob retained"
            ));
            return Ok(());
        }
    };

    // (e) the envelope must be addressed to the key we hold (routing hint, then
    // proven by decryption below). A different fingerprint -> retain, never
    // delete (I3).
    if outer.recipient_key_fingerprint != *key_fp {
        main.errors.push(format!(
            "receipt {receipt_id}: outer envelope addressed to {} but this puller holds {key_fp}; \
             blob RETAINED, never deleted (I3)",
            outer.recipient_key_fingerprint
        ));
        return Ok(());
    }

    // (f) decode the ciphertext with the STANDARD (padded) engine - the variant
    // the form encodes with (`sodium.to_base64(.., ORIGINAL)`), pinned in D-083.
    let ciphertext_bytes = match BASE64_STANDARD.decode(outer.ciphertext.as_bytes()) {
        Ok(bytes) => bytes,
        Err(err) => {
            main.errors.push(format!(
                "receipt {receipt_id}: ciphertext base64 decode failed ({err}); blob retained"
            ));
            return Ok(());
        }
    };

    // (g) open the sealed box. A failed open is content-free (I3): it leaks
    // nothing about why.
    let plaintext = match cn_ingest::open(&ciphertext_bytes, keypair) {
        Ok(plaintext) => plaintext,
        Err(_) => {
            main.errors.push(format!(
                "receipt {receipt_id}: sealed-box open failed (content-free); blob retained"
            ));
            return Ok(());
        }
    };

    // (h) parse the inner payload (version + consent_affirmed enforced here).
    let inner = match InnerPayload::parse(&plaintext) {
        Ok(inner) => inner,
        Err(err) => {
            main.errors.push(format!(
                "receipt {receipt_id}: inner payload parse failed ({err}); blob retained"
            ));
            return Ok(());
        }
    };

    // (i) consent-digest recognition: unknown is a WARNING, never a halt (the
    // form may have redeployed between affirmation and this pull).
    if let ConsentDigestVerdict::Unknown = check_consent_digest(
        &inner.consent.consent_text_digest,
        &config.known_consent_digests,
    ) {
        main.warnings.push(format!(
            "receipt {receipt_id}: consent digest {} is not among the known deployed digests \
             (form may have redeployed); staged anyway",
            inner.consent.consent_text_digest
        ));
    }

    // (j) semantic dedup (only when transport was Fresh; a transport conflict is
    // already dispositioned above).
    let payload_hash =
        canonical_digest(&inner).map_err(|err| format!("receipt {receipt_id}: {err}"))?;
    if !is_transport_conflict {
        let semantic_key = DedupKey {
            receipt_id: Some(receipt_id.clone()),
            ciphertext_hash: Some(ciphertext_hash.clone()),
            submission_id: inner.submission_id.clone(),
            payload_hash: payload_hash.clone(),
        };
        match classify_dedup(&semantic_key, existing_keys) {
            DedupVerdict::SemanticReplay => {
                // Same submission already staged; recorded no-op.
                main.semantic_replays += 1;
                return Ok(());
            }
            DedupVerdict::Conflict => {
                // Same submission id, DIFFERENT bytes: stage BOTH, LOUD, and
                // RETAIN the blob as evidence (ADR-005 D4).
                main.semantic_conflicts += 1;
                main.errors.push(format!(
                    "receipt {receipt_id}: SEMANTIC CONFLICT - submission id {} already staged \
                     with different bytes; staging BOTH, linked, and RETAINING the relay blob \
                     (ADR-005 D4, I3)",
                    inner.submission_id
                ));
                delete_allowed = false;
            }
            DedupVerdict::Fresh => {}
            other => {
                main.errors.push(format!(
                    "receipt {receipt_id}: impossible semantic dedup verdict {other:?}; blob \
                     retained"
                ));
                return Ok(());
            }
        }
    }

    // (k) stage: build the remote context, convert, write record + sidecar, and
    // read them back to verify the pair (the integrity contract `cn intake
    // apply` relies on). A staging fault RETAINS the blob (never deleted).
    let ctx = RemoteContext {
        claimed_fingerprint: outer.recipient_key_fingerprint.clone(),
        envelope_version: outer.intake_envelope_version.to_string(),
        key_used: key_fp.to_string(),
        receipt_id: receipt_id.clone(),
        relay_received_at: row
            .arrived_at
            .as_deref()
            .and_then(parse_iso8601_utc_to_unix_ms)
            .map(Timestamp),
        pulled_at: Timestamp(now_ms),
        ciphertext_hash: ciphertext_hash.clone(),
    };
    let record = match persist_staged(paths, &inner, ctx, now_ms) {
        Ok(record) => record,
        Err(err) => {
            main.errors.push(format!(
                "receipt {receipt_id}: staging failed ({err}); blob RETAINED, not deleted (I3)"
            ));
            return Ok(());
        }
    };
    // The next receipt in THIS loop deduplicates against what we just staged.
    existing_keys.push(DedupKey::from_record(&record));
    main.staged += 1;

    // (l) delete the relay blob ONLY after verified staging, and never on a
    // conflict path (evidence retained). A lost/failed delete is a warning, not
    // an error: the delete is idempotent, so next run retries.
    if delete_allowed {
        match relay.delete(&format!("{origin}/blob/{receipt_id}")) {
            Ok((200, _)) | Ok((204, _)) => {
                delete_journal.insert(receipt_id.clone());
                main.blobs_deleted += 1;
            }
            Ok((other, _)) => main.warnings.push(format!(
                "receipt {receipt_id}: DELETE blob returned status {other}; not retried this run \
                 (idempotent - next run retries)"
            )),
            Err(err) => main.warnings.push(format!(
                "receipt {receipt_id}: DELETE blob transport error ({err}); not retried this run \
                 (idempotent - next run retries)"
            )),
        }
    }
    Ok(())
}

/// Builds, writes, and read-back-verifies a staged remote record + its initial
/// pending sidecar. Returns the in-memory record for dedup bookkeeping.
fn persist_staged(
    paths: &QueuePaths,
    inner: &InnerPayload,
    ctx: RemoteContext,
    now_ms: i64,
) -> Result<QueueRecord, String> {
    let record = cn_ingest::stage_remote_record(inner, ctx, new_uuid_v7(), Timestamp(now_ms))
        .map_err(|err| format!("stage_remote_record failed: {err}"))?;
    let sidecar =
        ReviewSidecar::initial(&record).map_err(|err| format!("initial sidecar failed: {err}"))?;

    let record_bytes = serde_json::to_vec_pretty(&record)
        .map_err(|err| format!("cannot serialize record: {err}"))?;
    let sidecar_bytes = serde_json::to_vec_pretty(&sidecar)
        .map_err(|err| format!("cannot serialize sidecar: {err}"))?;
    queue::write_atomic(&paths.record(&record.record_id), &record_bytes)?;
    queue::write_atomic(&paths.sidecar(&record.record_id), &sidecar_bytes)?;

    // Read both back and verify the pair binding + checksums, so a torn or
    // mangled write is caught HERE, before the blob is deleted.
    let read_record: QueueRecord = read_json(&paths.record(&record.record_id))?;
    let read_sidecar: ReviewSidecar = read_json(&paths.sidecar(&record.record_id))?;
    read_sidecar
        .verify_pair(&read_record)
        .map_err(|err| format!("staged record failed read-back verification: {err}"))?;
    Ok(record)
}

/// Post-loop reconciliation: re-list receipts and classify each by the D6
/// precedence (blueprint 6.1 post-loop). Local durable facts (staged, deleted-
/// by-me this run) outrank the eventually-consistent relay observations.
fn reconcile(
    config: &PullConfig,
    paths: &QueuePaths,
    relay: &dyn RelayHttp,
    now_ms: i64,
    delete_journal: &HashSet<String>,
) -> Result<Reconciliation, String> {
    let listing = fetch_receipts(relay, config)?;
    let staged_ids = scan_remote_receipt_ids(paths)?;
    let now_secs = now_ms.div_euclid(1000);
    let ttl_margin = config.blob_ttl_seconds + CONSISTENCY_MARGIN_SECS;
    let half_ttl = config.blob_ttl_seconds / 2;

    let mut recon = Reconciliation {
        total_receipts: listing.len(),
        ..Reconciliation::default()
    };
    for row in &listing {
        let age = row
            .arrived_at
            .as_deref()
            .and_then(parse_iso8601_utc_to_unix_ms)
            .map(|ms| now_secs - ms.div_euclid(1000));
        let obs = ReceiptObservation {
            local_transport_record: staged_ids.contains(&row.receipt_id),
            local_delete_journal: delete_journal.contains(&row.receipt_id),
            relay_blob_present: row.has_blob,
            age,
            ttl_margin_threshold: ttl_margin,
        };
        match classify_receipt(&obs) {
            ReceiptClass::Staged => recon.staged += 1,
            ReceiptClass::DeletedByMe => recon.deleted_by_me += 1,
            ReceiptClass::Unpulled => {
                recon.unpulled += 1;
                if let Some(age) = age {
                    recon.oldest_unpulled_age_secs = Some(
                        recon
                            .oldest_unpulled_age_secs
                            .map_or(age, |cur| cur.max(age)),
                    );
                    if age > half_ttl {
                        recon.half_ttl_warning = true;
                    }
                }
            }
            ReceiptClass::IntegrityAlert => recon.integrity_alert += 1,
            ReceiptClass::Expired => recon.expired += 1,
        }
    }
    Ok(recon)
}

/// Scans the queue for every staged record's dedup key (pre-loop seed).
fn scan_dedup_keys(paths: &QueuePaths) -> Result<Vec<DedupKey>, String> {
    let outcome = queue::scan(paths)?;
    Ok(outcome
        .records
        .values()
        .filter_map(|record| record.payload.as_ref())
        .map(DedupKey::from_record)
        .collect())
}

/// Scans the queue for the receipt ids of every locally-staged REMOTE record
/// (reconciliation's strongly-consistent local fact).
fn scan_remote_receipt_ids(paths: &QueuePaths) -> Result<HashSet<String>, String> {
    let outcome = queue::scan(paths)?;
    let mut ids = HashSet::new();
    for record in outcome.records.values() {
        if let Some(payload) = &record.payload
            && let SubmissionSource::Remote { receipt_id, .. } = &payload.source
        {
            ids.insert(receipt_id.clone());
        }
    }
    Ok(ids)
}

/// The `cn intake pull --config <path> --queue <queue-root>` entry point. Loads
/// the config, credential, and passphrase-protected key OUT of band, builds the
/// real HTTP clients, and delegates to [`execute_pull`]. `--queue` matches `cn
/// intake apply`'s interface: the same queue root, the same lock.
pub(crate) fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<Exit> {
    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            super::usage_to(err)?;
            return Ok(Exit::Usage);
        }
    };

    let config = match fs::read(&parsed.config).map_err(|io_err| io_err.to_string()) {
        Ok(bytes) => match parse_config(&bytes) {
            Ok(config) => config,
            Err(message) => {
                writeln!(err, "error: {message}")?;
                return Ok(Exit::Failure);
            }
        },
        Err(io_err) => {
            writeln!(
                err,
                "error: cannot read config '{}': {io_err}",
                parsed.config
            )?;
            return Ok(Exit::Failure);
        }
    };

    // Bearer credential (plain text; trailing whitespace trimmed).
    let credential = match fs::read_to_string(&config.credential_path) {
        Ok(text) => text.trim().to_string(),
        Err(io_err) => {
            writeln!(
                err,
                "error: cannot read credential file '{}': {io_err}",
                config.credential_path.display()
            )?;
            return Ok(Exit::Failure);
        }
    };
    if credential.is_empty() {
        writeln!(
            err,
            "error: credential file '{}' is empty",
            config.credential_path.display()
        )?;
        return Ok(Exit::Failure);
    }

    // Preconditions 3-4 FIRST (fail-fast, blueprint 6.1 order): validate the
    // queue root and take the single-mutator lock BEFORE prompting for the
    // passphrase or decrypting the secret - an unsafe root or a held lock must
    // never cost the operator a passphrase entry or a needless key decrypt.
    let root = match queue::refuse_unsafe_root(Path::new(&parsed.queue)) {
        Ok(root) => root,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            return Ok(Exit::Failure);
        }
    };
    let paths = QueuePaths::new(&root);
    let _lock = match queue::acquire_lock(&paths) {
        Ok(lock) => lock,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            return Ok(Exit::Failure);
        }
    };

    // Preconditions 5-6: load + pin-check the public key, then decrypt the
    // secret and cross-check (the passphrase prompt happens only now that the
    // lock is held).
    let keypair = match load_keypair(&config, err)? {
        Ok(keypair) => keypair,
        Err(exit) => return Ok(exit),
    };

    let now_ms = match crate::export::unix_now_ms() {
        Ok(now_ms) => now_ms,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            return Ok(Exit::Failure);
        }
    };

    // Redirect-disabled, status-tolerant agent (rustls TLS, vendored roots).
    let agent = build_agent();
    let relay = UreqRelay {
        agent: agent.clone(),
        bearer: format!("Bearer {credential}"),
    };
    let bundle_agent = agent.clone();
    let bundle_fetch = move |url: &str| bundle_get(&bundle_agent, url);

    match run_locked(&config, &keypair, &paths, &relay, bundle_fetch, now_ms) {
        Ok(summary) => {
            super::emit_report(out, &summary.report)?;
            if summary.failure {
                writeln!(
                    err,
                    "error: the pull report above records halts, conflicts, integrity alerts, or \
                     errors needing facilitator attention"
                )?;
                Ok(Exit::Failure)
            } else {
                Ok(Exit::Ok)
            }
        }
        Err(message) => {
            writeln!(err, "error: {message}")?;
            Ok(Exit::Failure)
        }
    }
}

struct PullArgs {
    config: String,
    queue: String,
}

fn parse_args(args: &[String]) -> Result<PullArgs, String> {
    let mut config = None;
    let mut queue = None;
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let slot = match flag.as_str() {
            "--config" => &mut config,
            "--queue" => &mut queue,
            other => return Err(format!("unknown intake pull argument '{other}'")),
        };
        let value = iter.next().ok_or_else(|| format!("{flag} needs a value"))?;
        if slot.replace(value.clone()).is_some() {
            return Err(format!("{flag} given more than once"));
        }
    }
    Ok(PullArgs {
        config: config.ok_or_else(|| "--config is required".to_string())?,
        queue: queue.ok_or_else(|| "--queue is required".to_string())?,
    })
}

/// Loads the keypair from `config.key_dir`: parse + pin-check `public.json`,
/// prompt for the passphrase, open `secret.json`, and cross-check the halves.
/// Returns `Ok(Ok(keypair))` on success, `Ok(Err(exit))` on a handled failure
/// already written to `err` (so `run` returns that exit).
fn load_keypair(
    config: &PullConfig,
    err: &mut dyn Write,
) -> std::io::Result<Result<Keypair, Exit>> {
    let public_path = config.key_dir.join("public.json");
    let public_bytes = match fs::read(&public_path) {
        Ok(bytes) => bytes,
        Err(io_err) => {
            writeln!(
                err,
                "error: cannot read '{}': {io_err}",
                public_path.display()
            )?;
            return Ok(Err(Exit::Failure));
        }
    };
    let public = match parse_public_key(&public_bytes) {
        Ok(public) => public,
        Err(ingest_err) => {
            writeln!(
                err,
                "error: cannot parse public key '{}': {ingest_err}",
                public_path.display()
            )?;
            return Ok(Err(Exit::Failure));
        }
    };
    let key_fp = fingerprint(&public).to_string();
    if key_fp != config.key_fingerprint_pin {
        writeln!(
            err,
            "error: key at '{}' has fingerprint {key_fp} but the config pins {}; refusing (I3)",
            public_path.display(),
            config.key_fingerprint_pin
        )?;
        return Ok(Err(Exit::Failure));
    }

    let secret_path = config.key_dir.join("secret.json");
    let secret_bytes = match fs::read(&secret_path) {
        Ok(bytes) => bytes,
        Err(io_err) => {
            writeln!(
                err,
                "error: cannot read '{}': {io_err}",
                secret_path.display()
            )?;
            return Ok(Err(Exit::Failure));
        }
    };
    let passphrase = match super::keymat::read_existing_passphrase(
        err,
        "unlock the intake secret key for the pull",
    ) {
        Ok(passphrase) => passphrase,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            return Ok(Err(Exit::Failure));
        }
    };
    let secret = match parse_secret_key(&secret_bytes, &passphrase) {
        Ok(secret) => secret,
        Err(ingest_err) => {
            writeln!(err, "error: cannot open secret key: {ingest_err}")?;
            return Ok(Err(Exit::Failure));
        }
    };
    if secret.public_key() != public {
        writeln!(
            err,
            "error: the secret key does not match the public key in '{}'; refusing (I3)",
            public_path.display()
        )?;
        return Ok(Err(Exit::Failure));
    }
    Ok(Ok(Keypair { public, secret }))
}

/// Builds the puller's HTTP agent: NO redirect following (a redirect is a
/// deployment anomaly, surfaced as a non-200 status rather than an error), and
/// non-2xx returned as a response (the puller decides per status).
fn build_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .max_redirects(0)
        .max_redirects_will_error(false)
        .build()
        .into()
}

enum Method {
    Get,
    Delete,
}

/// One HTTP call: applies the headers, forces non-2xx to a returned response
/// (not an error), and reads the body up to [`MAX_RESPONSE_BYTES`].
fn http_call(
    agent: &ureq::Agent,
    method: Method,
    url: &str,
    headers: &[(&str, &str)],
) -> HttpResult {
    let mut request = match method {
        Method::Get => agent.get(url),
        Method::Delete => agent.delete(url),
    };
    for &(name, value) in headers {
        request = request.header(name, value);
    }
    let mut response = request
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|err| err.to_string())?;
    let status = response.status().as_u16();
    let body = response
        .body_mut()
        .with_config()
        .limit(MAX_RESPONSE_BYTES)
        .read_to_vec()
        .map_err(|err| err.to_string())?;
    Ok((status, body))
}

/// Bundle-check GET: cache-bypassed, no auth (the Pages origin is public).
fn bundle_get(agent: &ureq::Agent, url: &str) -> HttpResult {
    http_call(
        agent,
        Method::Get,
        url,
        &[("Cache-Control", "no-cache, no-store")],
    )
}

/// The production relay client: a shared `ureq` agent and the bearer header.
struct UreqRelay {
    agent: ureq::Agent,
    bearer: String,
}

impl RelayHttp for UreqRelay {
    fn get(&self, url: &str) -> HttpResult {
        http_call(
            &self.agent,
            Method::Get,
            url,
            &[("Authorization", self.bearer.as_str())],
        )
    }

    fn delete(&self, url: &str) -> HttpResult {
        http_call(
            &self.agent,
            Method::Delete,
            url,
            &[("Authorization", self.bearer.as_str())],
        )
    }
}

/// SHA-256 (lowercase hex) over exact bytes.
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|err| format!("cannot read '{}': {err}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| format!("cannot parse '{}': {err}", path.display()))
}

/// Parses the relay's `arrived_at` (always `new Date().toISOString()` =
/// `YYYY-MM-DDTHH:MM:SS.sssZ`, UTC) to Unix milliseconds. Returns `None` on any
/// deviation from that exact shape - a focused parser for the ONE format the
/// relay emits, consistent with keymat.rs hand-rolling its ISO handling rather
/// than adding a date crate to this tool.
fn parse_iso8601_utc_to_unix_ms(text: &str) -> Option<i64> {
    let text = text.strip_suffix('Z')?;
    let (date, time) = text.split_once('T')?;

    let mut date_parts = date.split('-');
    let year = parse_fixed(date_parts.next()?, 4)? as i64;
    let month = parse_fixed(date_parts.next()?, 2)?;
    let day = parse_fixed(date_parts.next()?, 2)?;
    if date_parts.next().is_some() {
        return None;
    }

    let (hms, millis) = match time.split_once('.') {
        Some((hms, frac)) => (hms, parse_millis(frac)?),
        None => (time, 0),
    };
    let mut time_parts = hms.split(':');
    let hour = parse_fixed(time_parts.next()?, 2)? as i64;
    let minute = parse_fixed(time_parts.next()?, 2)? as i64;
    let second = parse_fixed(time_parts.next()?, 2)? as i64;
    if time_parts.next().is_some() {
        return None;
    }

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // Allow a leap second (60); reject anything further out of range.
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }

    let days = days_from_civil(year, month, day);
    let secs = days * 86_400 + hour * 3600 + minute * 60 + second;
    Some(secs * 1000 + millis)
}

/// Parses a fixed-width, all-ASCII-digit numeric group (rejects `+`, spaces, and
/// short/long groups so a malformed timestamp fails rather than mis-parses).
fn parse_fixed(group: &str, width: usize) -> Option<u32> {
    if group.len() != width || !group.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    group.parse().ok()
}

/// Parses the fractional-seconds part to whole milliseconds (first three digits,
/// zero-padded). `toISOString` always emits exactly three, but this tolerates
/// 1-9 digits.
fn parse_millis(frac: &str) -> Option<i64> {
    if frac.is_empty() || !frac.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let mut millis = 0i64;
    for index in 0..3 {
        let digit = frac
            .as_bytes()
            .get(index)
            .map_or(0, |byte| (byte - b'0') as i64);
        millis = millis * 10 + digit;
    }
    Some(millis)
}

/// Howard Hinnant's `days_from_civil`: (year, month, day) in the proleptic
/// Gregorian calendar -> days since 1970-01-01. The civil->days inverse of
/// keymat.rs's `civil_from_days`.
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = (if year >= 0 { year } else { year - 399 }) / 400;
    let year_of_era = year - era * 400; // [0, 399]
    let month = month as i64;
    let day = day as i64;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1; // [0, 365]
    let doe = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_parser_pins_known_instants() {
        assert_eq!(
            parse_iso8601_utc_to_unix_ms("1970-01-01T00:00:00Z"),
            Some(0)
        );
        assert_eq!(
            parse_iso8601_utc_to_unix_ms("2000-01-01T00:00:00.000Z"),
            Some(946_684_800_000)
        );
        // +59 days from 2024-01-01 lands on the leap day.
        assert_eq!(
            parse_iso8601_utc_to_unix_ms("2024-02-29T00:00:00.000Z"),
            Some(1_709_164_800_000)
        );
        // Time-of-day plus milliseconds.
        assert_eq!(
            parse_iso8601_utc_to_unix_ms("2000-01-01T13:45:30.999Z"),
            Some(946_684_800_000 + 49_530_999)
        );
    }

    #[test]
    fn iso_parser_rejects_malformed() {
        assert_eq!(parse_iso8601_utc_to_unix_ms(""), None);
        assert_eq!(parse_iso8601_utc_to_unix_ms("2000-01-01T00:00:00"), None); // no Z
        assert_eq!(parse_iso8601_utc_to_unix_ms("2000-13-01T00:00:00Z"), None); // month 13
        assert_eq!(parse_iso8601_utc_to_unix_ms("2000-01-01 00:00:00Z"), None); // space, no T
        assert_eq!(parse_iso8601_utc_to_unix_ms("2000-1-01T00:00:00Z"), None); // short month
        assert_eq!(parse_iso8601_utc_to_unix_ms("2000-01-01T25:00:00Z"), None); // hour 25
    }

    #[test]
    fn days_from_civil_matches_known_epochs() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 1, 1), 10_957);
        assert_eq!(days_from_civil(1969, 12, 31), -1);
    }
}
