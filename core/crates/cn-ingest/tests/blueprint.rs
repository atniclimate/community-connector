//! ADR-005 D4 queue formats, admission table, and recovery classification.

use cn_ingest::*;
use cn_model::Timestamp;
use serde_json::json;

fn ts(ms: i64) -> Timestamp {
    Timestamp(ms)
}

fn payload() -> serde_json::Value {
    json!({
        "submission_version": "0.1.0",
        "submission_id": "sub-1",
        "form_version": "form-0.1",
        "consent": {
            "consent_text_digest": "digest-consent",
            "consent_affirmed": true,
            "consent_affirmed_at": 5
        },
        "captured_at": 4,
        "fields": {
            "display_name": "Synthetic Person",
            "affiliation": ["River Alliance"]
        }
    })
}

fn record() -> QueueRecord {
    QueueRecord::new(
        "rec-1".to_string(),
        ts(10),
        SubmissionSource::InApp {},
        payload(),
    )
    .expect("record")
}

fn remote_record() -> QueueRecord {
    QueueRecord::new(
        "rec-2".to_string(),
        ts(11),
        SubmissionSource::Remote {
            claimed_fingerprint: "fp-1".to_string(),
            envelope_version: "0.1.0".to_string(),
            key_used: "fp-1".to_string(),
            receipt_id: "receipt-1".to_string(),
            relay_received_at: None,
            pulled_at: ts(11),
            ciphertext_hash: "ct-hash".to_string(),
        },
        payload(),
    )
    .expect("record")
}

fn decision(
    id: &str,
    decided_at: i64,
    expected_state: ReviewState,
    expected_generation: u64,
    decision: DecisionType,
) -> DecisionMessage {
    let record = record();
    DecisionMessage {
        queue_record_version: decision_message_version(),
        decision_id: id.to_string(),
        record_id: record.record_id.clone(),
        payload_digest: record.record_checksum.clone(),
        expected_review_state: expected_state,
        expected_decision_generation: expected_generation,
        decision,
        reviewer: "facilitator".to_string(),
        decided_at: ts(decided_at),
    }
}

// --- formats: versioning, checksums, binding ---

#[test]
fn record_and_sidecar_verify_and_round_trip() {
    let record = remote_record();
    record.verify().expect("valid record");
    let sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    sidecar.verify_pair(&record).expect("bound pair");
    let json = serde_json::to_string(&record).expect("serialize");
    let back: QueueRecord = serde_json::from_str(&json).expect("parse");
    back.verify().expect("round trip verifies");
    assert_eq!(back, record);
}

#[test]
fn checksum_tamper_is_detected() {
    let mut record = record();
    record.payload["fields"]["name"] = json!("Tampered");
    assert!(matches!(
        record.verify(),
        Err(IngestError::ChecksumMismatch { .. })
    ));
}

#[test]
fn unknown_major_version_rejected_loudly() {
    let mut record = record();
    record.queue_record_version = semver::Version::new(9, 0, 0);
    assert!(matches!(
        record.verify(),
        Err(IngestError::UnknownMajorVersion { .. })
    ));
}

#[test]
fn unknown_minor_fields_are_preserved_across_sidecar_rewrite() {
    let record = record();
    let sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let mut value = serde_json::to_value(&sidecar).expect("value");
    value["future_minor_field"] = json!("kept");
    // Recompute the checksum as a newer-minor writer would have.
    let mut parsed: ReviewSidecar = serde_json::from_value(value).expect("parses with extras");
    assert_eq!(
        parsed.extras.get("future_minor_field"),
        Some(&json!("kept"))
    );
    parsed.seal_rewrite().expect("seal");
    let rewritten = serde_json::to_value(&parsed).expect("serialize");
    assert_eq!(
        rewritten["future_minor_field"],
        json!("kept"),
        "ignore-and-preserve across rewrite"
    );
}

#[test]
fn sidecar_binding_mismatch_is_loud() {
    let record_a = record();
    let record_b = remote_record();
    let sidecar_a = ReviewSidecar::initial(&record_a).expect("sidecar");
    assert!(matches!(
        sidecar_a.verify_pair(&record_b),
        Err(IngestError::BindingMismatch { .. })
    ));
}

// --- admission table ---

#[test]
fn admit_approve_sets_intent_and_advances_generation() {
    let record = record();
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let message = decision("d-1", 1, ReviewState::Pending, 0, DecisionType::Approve);
    let verdict = admit(&sidecar, &message).expect("verdict");
    let AdmissionVerdict::Admit {
        resulting_state,
        resulting_generation,
        ..
    } = &verdict
    else {
        panic!("expected admit, got {verdict:?}");
    };
    assert_eq!(*resulting_state, ReviewState::ApprovedIntent);
    assert_eq!(*resulting_generation, 1);
    apply_verdict(&mut sidecar, &verdict).expect("apply");
    assert_eq!(sidecar.review_state, ReviewState::ApprovedIntent);
    assert_eq!(sidecar.decision_generation, 1);
    assert_eq!(sidecar.sidecar_revision, 1);
}

#[test]
fn round6_crash_sequence_replayed_note_is_writeless_and_approve_still_admits() {
    // Note admitted at generation G; crash before retirement; approve
    // authored against G+1; the replayed note retires writeless; the
    // approve STILL ADMITS (the round-6 mandatory sequence).
    let record = record();
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let note = decision(
        "d-note",
        1,
        ReviewState::Pending,
        0,
        DecisionType::SetAsideNote {
            note: "checking affiliation".to_string(),
        },
    );
    let verdict = admit(&sidecar, &note).expect("verdict");
    apply_verdict(&mut sidecar, &verdict).expect("apply note");
    assert_eq!(sidecar.review_state, ReviewState::Pending);
    assert_eq!(sidecar.decision_generation, 1, "note advances generation");

    // Facilitator reloads at generation 1 and authors an approve.
    let approve = decision(
        "d-approve",
        2,
        ReviewState::Pending,
        1,
        DecisionType::Approve,
    );

    // Crash retry: the note replays first (deterministic order).
    let replay = admit(&sidecar, &note).expect("verdict");
    assert_eq!(replay, AdmissionVerdict::WritelessReplay);
    apply_verdict(&mut sidecar, &replay).expect("no-op");
    assert_eq!(sidecar.sidecar_revision, 1, "replay moved no counter");

    // The approve still admits.
    let verdict = admit(&sidecar, &approve).expect("verdict");
    assert!(
        matches!(verdict, AdmissionVerdict::Admit { .. }),
        "approve must still admit after the writeless replay, got {verdict:?}"
    );
}

#[test]
fn round6_stale_before_current_leaves_current_decision_admissible() {
    let record = record();
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    // Advance to generation 1 via an admitted note.
    let note = decision(
        "d-note",
        1,
        ReviewState::Pending,
        0,
        DecisionType::SetAsideNote {
            note: "n".to_string(),
        },
    );
    let verdict = admit(&sidecar, &note).expect("verdict");
    apply_verdict(&mut sidecar, &verdict).expect("apply");

    // An OLDER stale message (authored against generation 0) processes
    // before a current decision.
    let stale = decision("d-old", 0, ReviewState::Pending, 0, DecisionType::Reject);
    let verdict = admit(&sidecar, &stale).expect("verdict");
    let AdmissionVerdict::Stale { .. } = &verdict else {
        panic!("expected stale, got {verdict:?}");
    };
    apply_verdict(&mut sidecar, &verdict).expect("record stale");
    assert_eq!(
        sidecar.decision_generation, 1,
        "stale audit entry must not advance the generation"
    );

    // A decision authored against the CURRENT generation still admits.
    let current = decision("d-cur", 3, ReviewState::Pending, 1, DecisionType::Approve);
    let verdict = admit(&sidecar, &current).expect("verdict");
    assert!(
        matches!(verdict, AdmissionVerdict::Admit { .. }),
        "current decision must remain admissible, got {verdict:?}"
    );
}

#[test]
fn aba_cycle_fails_cas_for_old_view() {
    // pending -> failed -> clear_failed -> pending: state returns but the
    // generation advanced, so a decision authored against the ORIGINAL
    // pending fails CAS instead of admitting.
    let record = record();
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    // Drive to failed via a transaction event after an admitted approve.
    let approve = decision("d-1", 1, ReviewState::Pending, 0, DecisionType::Approve);
    let verdict = admit(&sidecar, &approve).expect("verdict");
    apply_verdict(&mut sidecar, &verdict).expect("apply");
    record_transaction(
        &mut sidecar,
        "d-1",
        TransactionKind::DurableConflict,
        ReviewState::Failed,
        ts(2),
        None,
    )
    .expect("transaction");
    // Explicit disposition back to pending.
    let clear = decision(
        "d-2",
        3,
        ReviewState::Failed,
        sidecar.decision_generation,
        DecisionType::ClearFailed,
    );
    let verdict = admit(&sidecar, &clear).expect("verdict");
    apply_verdict(&mut sidecar, &verdict).expect("apply clear");
    assert_eq!(sidecar.review_state, ReviewState::Pending);

    // A decision authored against the ORIGINAL (pending, generation 0)
    // view must be stale.
    let old_view = decision("d-3", 4, ReviewState::Pending, 0, DecisionType::Approve);
    let verdict = admit(&sidecar, &old_view).expect("verdict");
    assert!(
        matches!(verdict, AdmissionVerdict::Stale { .. }),
        "ABA old view must fail CAS, got {verdict:?}"
    );
}

#[test]
fn two_concurrent_decisions_first_admits_second_stale_including_note_first() {
    let record = record();
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let note = decision(
        "d-a",
        1,
        ReviewState::Pending,
        0,
        DecisionType::SetAsideNote {
            note: "n".to_string(),
        },
    );
    let reject = decision("d-b", 2, ReviewState::Pending, 0, DecisionType::Reject);
    let verdict = admit(&sidecar, &note).expect("verdict");
    apply_verdict(&mut sidecar, &verdict).expect("apply first");
    let verdict = admit(&sidecar, &reject).expect("verdict");
    assert!(
        matches!(verdict, AdmissionVerdict::Stale { .. }),
        "second concurrent decision must be stale even when the first was note-only"
    );
}

#[test]
fn same_id_different_digest_is_conflict_never_replay() {
    let record = record();
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let original = decision("d-1", 1, ReviewState::Pending, 0, DecisionType::Reject);
    let verdict = admit(&sidecar, &original).expect("verdict");
    apply_verdict(&mut sidecar, &verdict).expect("apply");
    let mut reused = decision("d-1", 9, ReviewState::Pending, 1, DecisionType::Approve);
    reused.reviewer = "someone-else".to_string();
    let verdict = admit(&sidecar, &reused).expect("verdict");
    assert_eq!(verdict, AdmissionVerdict::IdReuseConflict);
}

#[test]
fn illegal_transition_gets_durable_entry_without_generation_movement() {
    let record = record();
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let clear = decision("d-1", 1, ReviewState::Pending, 0, DecisionType::ClearFailed);
    let verdict = admit(&sidecar, &clear).expect("verdict");
    let AdmissionVerdict::Illegal { .. } = &verdict else {
        panic!("expected illegal, got {verdict:?}");
    };
    apply_verdict(&mut sidecar, &verdict).expect("record illegal");
    assert_eq!(sidecar.decision_generation, 0);
    assert_eq!(sidecar.history.len(), 1);
    assert_eq!(sidecar.sidecar_revision, 1, "physical revision still bumps");
}

#[test]
fn preflight_failure_transaction_event_is_representable_and_serialized() {
    let record = record();
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let approve = decision("d-1", 1, ReviewState::Pending, 0, DecisionType::Approve);
    let verdict = admit(&sidecar, &approve).expect("verdict");
    apply_verdict(&mut sidecar, &verdict).expect("apply");
    record_transaction(
        &mut sidecar,
        "d-1",
        TransactionKind::PreflightFailed,
        ReviewState::Pending,
        ts(2),
        Some("denied: test".to_string()),
    )
    .expect("transaction");
    assert_eq!(sidecar.review_state, ReviewState::Pending);
    // All four transaction kinds round-trip through the tagged union.
    for kind in [
        TransactionKind::IntentCompleted,
        TransactionKind::PreflightFailed,
        TransactionKind::DurableConflict,
        TransactionKind::DurableInconsistency,
    ] {
        let entry = HistoryEntry::Transaction {
            linked_decision_id: "d-1".to_string(),
            kind,
            prior_state: ReviewState::ApprovedIntent,
            resulting_state: ReviewState::Failed,
            at: ts(3),
            detail: None,
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        let back: HistoryEntry = serde_json::from_str(&json).expect("parse");
        assert_eq!(back, entry);
    }
}

#[test]
fn deterministic_processing_order() {
    let mut messages = vec![
        decision("zz", 2, ReviewState::Pending, 0, DecisionType::Reject),
        decision("aa", 2, ReviewState::Pending, 0, DecisionType::Reject),
        decision("mm", 1, ReviewState::Pending, 0, DecisionType::Reject),
    ];
    processing_order(&mut messages);
    let ids: Vec<&str> = messages.iter().map(|m| m.decision_id.as_str()).collect();
    assert_eq!(ids, vec!["mm", "aa", "zz"]);
}

// --- recovery classification ---

fn found(
    payload: Option<QueueRecord>,
    payload_corrupt: bool,
    sidecar: FoundSidecar,
    marker: bool,
    decisions: usize,
) -> FoundRecord {
    FoundRecord {
        record_id: "rec-1".to_string(),
        payload,
        payload_corrupt,
        sidecar,
        marker_present: marker,
        pending_decisions: decisions,
    }
}

#[test]
fn recovery_table_rows() {
    let record = record();
    let sidecar = ReviewSidecar::initial(&record).expect("sidecar");

    // Payload without sidecar, no marker: reconstruct pending only.
    assert_eq!(
        classify(&found(
            Some(record.clone()),
            false,
            FoundSidecar::Missing,
            false,
            0
        ))
        .expect("classify"),
        RecoveryAction::ReconstructPendingSidecar {
            record_id: "rec-1".to_string()
        }
    );
    // Marker present without readable sidecar: lost decision state.
    assert_eq!(
        classify(&found(
            Some(record.clone()),
            false,
            FoundSidecar::Missing,
            true,
            0
        ))
        .expect("classify"),
        RecoveryAction::HaltLostDecisionState {
            record_id: "rec-1".to_string()
        }
    );
    // Corrupt anything: quarantine.
    assert_eq!(
        classify(&found(None, true, FoundSidecar::Corrupt, false, 0)).expect("classify"),
        RecoveryAction::QuarantineCorrupt {
            record_id: "rec-1".to_string()
        }
    );
    // approved_intent: run approval recovery first.
    let mut intent = sidecar.clone();
    intent.review_state = ReviewState::ApprovedIntent;
    intent.seal_rewrite().expect("seal");
    assert_eq!(
        classify(&found(
            Some(record.clone()),
            false,
            FoundSidecar::Valid(Box::new(intent)),
            false,
            0
        ))
        .expect("classify"),
        RecoveryAction::RunApprovalRecovery {
            record_id: "rec-1".to_string()
        }
    );
    // pending + marker + unconsumed decision: benign, admit normally.
    assert_eq!(
        classify(&found(
            Some(record.clone()),
            false,
            FoundSidecar::Valid(Box::new(sidecar.clone())),
            true,
            1
        ))
        .expect("classify"),
        RecoveryAction::ProceedWithAdmission {
            record_id: "rec-1".to_string()
        }
    );
    // pending + marker + no decision + no history: anomaly report.
    assert_eq!(
        classify(&found(
            Some(record.clone()),
            false,
            FoundSidecar::Valid(Box::new(sidecar.clone())),
            true,
            0
        ))
        .expect("classify"),
        RecoveryAction::ReportMarkerAnomaly {
            record_id: "rec-1".to_string()
        }
    );
    // Binding mismatch: halt.
    let other = remote_record();
    let foreign_sidecar = ReviewSidecar::initial(&other).expect("sidecar");
    assert_eq!(
        classify(&found(
            Some(record),
            false,
            FoundSidecar::Valid(Box::new(foreign_sidecar)),
            false,
            0
        ))
        .expect("classify"),
        RecoveryAction::HaltBindingMismatch {
            record_id: "rec-1".to_string()
        }
    );
}

// --- near-duplicate surfacing (D-056.4) ---

fn projected(name: &str, orgs: &[&str]) -> cn_perm::ProjectedEntity {
    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert(
        cn_model::AttrId::new("display_name").expect("attr id"),
        cn_model::AttributeValue::Text(name.to_string()),
    );
    if !orgs.is_empty() {
        attributes.insert(
            cn_model::AttrId::new("affiliation").expect("attr id"),
            cn_model::AttributeValue::Tags(orgs.iter().map(|s| s.to_string()).collect()),
        );
    }
    cn_perm::ProjectedEntity {
        id: "00000000-0000-0000-0000-00000000e001".parse().expect("id"),
        kind: cn_model::KindId::new("person").expect("kind"),
        owner_is_viewer: false,
        attributes,
    }
}

fn projection_with(entities: Vec<cn_perm::ProjectedEntity>) -> cn_perm::Projection {
    cn_perm::Projection {
        group_id: "00000000-0000-0000-0000-00000000000a".parse().expect("id"),
        viewer_fingerprint: "test".to_string(),
        revision: 1,
        entities,
        edges: Vec::new(),
        stories: Vec::new(),
    }
}

fn dup_payload(name: &str) -> serde_json::Value {
    json!({
        "submission_id": "sub-9",
        "fields": { "display_name": name, "affiliation": ["River Alliance"] }
    })
}

#[test]
fn near_dup_reasons_present_and_projection_bounded() {
    let name_attrs = [cn_model::AttrId::new("display_name").expect("attr")];
    let affil_attrs = [cn_model::AttrId::new("affiliation").expect("attr")];
    let payload = dup_payload("Jo\u{2028}Doe");

    // The wider projection (e.g. governance) contains the matching entity;
    // the narrower projection (facilitator without reach) does not. The
    // scorer sees ONLY what the projection exposes - candidates differ.
    let wide = projection_with(vec![projected("jo doe", &["river alliance"])]);
    let narrow = projection_with(Vec::new());

    let hits = near_duplicates(&payload, &[], &wide, &name_attrs, &affil_attrs);
    assert_eq!(hits.len(), 1, "wide projection surfaces the candidate");
    assert!(
        hits[0]
            .reasons
            .iter()
            .any(|r| r.contains("exact normalized name match")),
        "reasons: {:?}",
        hits[0].reasons
    );
    assert!(
        hits[0]
            .reasons
            .iter()
            .any(|r| r.contains("shared affiliation")),
        "affiliation overlap strengthens with its own reason"
    );

    let none = near_duplicates(&payload, &[], &narrow, &name_attrs, &affil_attrs);
    assert!(none.is_empty(), "no candidate outside the projection");
}

#[test]
fn near_dup_queue_and_containment_matching() {
    let name_attrs = [cn_model::AttrId::new("display_name").expect("attr")];
    let payload = dup_payload("Jo Doe");
    let queue = vec![
        ("rec-x".to_string(), dup_payload("Jo A. Doe")),
        ("rec-y".to_string(), dup_payload("Completely Different")),
    ];
    let hits = near_duplicates(
        &payload,
        &queue,
        &projection_with(Vec::new()),
        &name_attrs,
        &[],
    );
    assert_eq!(hits.len(), 1);
    assert_eq!(
        hits[0].source,
        CandidateSource::QueueRecord("rec-x".to_string())
    );
    assert!(hits[0].reasons[0].contains("token containment"));
}

#[test]
fn one_line_matches_d057_characterization() {
    assert_eq!(one_line("a\u{2028}b\u{2029}c\nd"), "a b c d");
    assert_eq!(one_line("a\n\n\nb"), "a b");
    assert_eq!(one_line("\n\u{2028}\u{2029}"), "");
}

// --- approval planning (ADR-005 D4/D5) ---

fn plan_template() -> cn_schema::GroupTemplate {
    let json = r##"{
        "schema_version": "0.1.0",
        "template_id": "research",
        "name": "Research Network",
        "description": "Synthetic",
        "kinds": [{
            "id": "person",
            "label": "Person",
            "shape": "sphere",
            "color_role": "kind-1",
            "attributes": [
                { "id": "display_name", "type": "text", "required": true },
                { "id": "affiliation", "type": "tags" }
            ]
        }],
        "edge_kinds": [],
        "theme": { "mode": "light", "roles": { "kind-1": "#112233" } }
    }"##;
    let (template, report) = cn_schema::parse_template(json).expect("template parses");
    assert!(report.errors.is_empty(), "clean template: {report:?}");
    template
}

fn plan_context() -> PlanContext {
    PlanContext {
        group_id: "00000000-0000-0000-0000-00000000000a".parse().expect("id"),
        facilitator: "00000000-0000-0000-0000-000000000001".parse().expect("id"),
        kind: cn_model::KindId::new("person").expect("kind"),
        now_ms: 1000,
        template_version: semver::Version::new(0, 1, 0),
    }
}

fn deterministic_ids() -> impl FnMut() -> uuid::Uuid {
    let mut n: u128 = 0;
    move || {
        n += 1;
        uuid::Uuid::from_u128(0xABCD0000_0000_0000_0000_000000000000 + n)
    }
}

#[test]
fn plan_is_deterministic_and_links_batch_digest_into_every_envelope() {
    let record = remote_record();
    let template = plan_template();
    let context = plan_context();
    let plan_a =
        plan_approval(&record, &template, &context, &mut deterministic_ids()).expect("plan");
    let plan_b =
        plan_approval(&record, &template, &context, &mut deterministic_ids()).expect("plan");
    assert_eq!(
        plan_a.plan_ref, plan_b.plan_ref,
        "same record + ids -> same digests"
    );
    assert_eq!(plan_a.plan_ref.op_ids.len(), 1);
    assert!(!plan_a.plan_ref.batch_digest.is_empty());
    assert_ne!(
        plan_a.plan_ref.per_op_digests[0], plan_a.plan_ref.batch_digest,
        "final per-op digest is over POPULATED bytes, distinct from the pre-link digest"
    );

    let cn_store::OpKind::EntityCreate { entity } = &plan_a.ops[0].kind else {
        panic!("expected EntityCreate");
    };
    let block = entity
        .provenance
        .intake()
        .expect("entity carries intake block");
    assert_eq!(block.batch_digest, plan_a.plan_ref.batch_digest);
    assert_eq!(block.record_id, record.record_id);
    assert_eq!(block.receipt_id.as_deref(), Some("receipt-1"));
    assert!(
        block.consent_affirmed,
        "consent assertion survives into provenance"
    );
    assert_eq!(block.consent_text_digest, "digest-consent");
    assert!(!entity.attributes.is_empty(), "payload fields mapped");
    for instance in entity.attributes.values() {
        let attr_block = instance
            .provenance
            .intake()
            .expect("every modeled value carries the block");
        assert_eq!(attr_block.batch_digest, plan_a.plan_ref.batch_digest);
    }
    assert!(
        plan_a.validation.errors.is_empty(),
        "valid payload validates: {:?}",
        plan_a.validation
    );
}

#[test]
fn plan_surfaces_missing_required_attribute_via_validation() {
    let mut bare = payload();
    bare["fields"] = json!({});
    let record = QueueRecord::new(
        "rec-3".to_string(),
        ts(12),
        SubmissionSource::InApp {},
        bare,
    )
    .expect("record");
    let plan = plan_approval(
        &record,
        &plan_template(),
        &plan_context(),
        &mut deterministic_ids(),
    )
    .expect("plan builds; validation reports");
    assert!(
        !plan.validation.errors.is_empty(),
        "missing required display_name must surface as a finding"
    );
}
