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
    let stale = decision(
        "d-old",
        0,
        ReviewState::Pending,
        0,
        DecisionType::Reject {
            reason: "synthetic reject reason".to_string(),
        },
    );
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
    let reject = decision(
        "d-b",
        2,
        ReviewState::Pending,
        0,
        DecisionType::Reject {
            reason: "synthetic reject reason".to_string(),
        },
    );
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
    let original = decision(
        "d-1",
        1,
        ReviewState::Pending,
        0,
        DecisionType::Reject {
            reason: "synthetic reject reason".to_string(),
        },
    );
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
            extras: std::collections::BTreeMap::new(),
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        let back: HistoryEntry = serde_json::from_str(&json).expect("parse");
        assert_eq!(back, entry);
    }
}

#[test]
fn deterministic_processing_order() {
    let mut messages = vec![
        decision(
            "zz",
            2,
            ReviewState::Pending,
            0,
            DecisionType::Reject {
                reason: "synthetic reject reason".to_string(),
            },
        ),
        decision(
            "aa",
            2,
            ReviewState::Pending,
            0,
            DecisionType::Reject {
                reason: "synthetic reject reason".to_string(),
            },
        ),
        decision(
            "mm",
            1,
            ReviewState::Pending,
            0,
            DecisionType::Reject {
                reason: "synthetic reject reason".to_string(),
            },
        ),
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

// --- transport and semantic dedup (ADR-005 D4, round-1 amendment) ---

#[test]
fn dedup_classifies_every_arm() {
    let existing = vec![DedupKey::from_record(&remote_record())];

    // Transport replay: same (receipt_id, ciphertext_hash) - recorded no-op.
    assert_eq!(
        classify_dedup(&DedupKey::from_record(&remote_record()), &existing),
        DedupVerdict::TransportReplay
    );

    // Transport conflict: same receipt, DIFFERENT ciphertext - loud
    // integrity conflict, never a no-op.
    let mut substituted = DedupKey::from_record(&remote_record());
    substituted.ciphertext_hash = Some("other-ct".to_string());
    assert_eq!(
        classify_dedup(&substituted, &existing),
        DedupVerdict::TransportConflict
    );

    // Semantic replay: same (submission_id, payload_hash), no transport key.
    assert_eq!(
        classify_dedup(&DedupKey::from_record(&record()), &existing),
        DedupVerdict::SemanticReplay
    );

    // Conflict: same submission_id, different payload - a DISTINCT typed
    // outcome for facilitator disposition, never a drop.
    let mut conflicting = payload();
    conflicting["fields"]["display_name"] = json!("Different Person");
    let conflict_record = QueueRecord::new(
        "rec-9".to_string(),
        ts(12),
        SubmissionSource::InApp {},
        conflicting,
    )
    .expect("record");
    assert_eq!(
        classify_dedup(&DedupKey::from_record(&conflict_record), &existing),
        DedupVerdict::Conflict
    );

    // Fresh: different submission id and payload.
    let mut fresh = payload();
    fresh["submission_id"] = json!("sub-2");
    fresh["fields"]["display_name"] = json!("Someone Else");
    let fresh_record = QueueRecord::new(
        "rec-10".to_string(),
        ts(13),
        SubmissionSource::InApp {},
        fresh,
    )
    .expect("record");
    assert_eq!(
        classify_dedup(&DedupKey::from_record(&fresh_record), &existing),
        DedupVerdict::Fresh
    );
}

#[test]
fn empty_submission_ids_never_match_each_other() {
    let mut blank_a = payload();
    blank_a["submission_id"] = json!("");
    let mut blank_b = payload();
    blank_b["submission_id"] = json!("");
    blank_b["fields"]["display_name"] = json!("Other");
    let a = QueueRecord::new(
        "rec-a".to_string(),
        ts(1),
        SubmissionSource::InApp {},
        blank_a,
    )
    .expect("record");
    let b = QueueRecord::new(
        "rec-b".to_string(),
        ts(2),
        SubmissionSource::InApp {},
        blank_b,
    )
    .expect("record");
    assert_eq!(
        classify_dedup(&DedupKey::from_record(&b), &[DedupKey::from_record(&a)]),
        DedupVerdict::Fresh,
        "client-controlled blank ids are not the same submission"
    );
}

// --- kind resolution and record validation (facade + CLI shared logic) ---

#[test]
fn resolve_kind_prefers_payload_and_falls_back_loudly() {
    let default_kind = cn_model::KindId::new(DEFAULT_PILOT_KIND).expect("kind");

    let (kind, warning) = resolve_kind(&record(), &default_kind);
    assert_eq!(kind, default_kind, "no payload kind: default");
    assert!(warning.is_none());

    let mut with_kind = payload();
    with_kind["kind"] = json!("person");
    let carried = QueueRecord::new(
        "rec-k".to_string(),
        ts(1),
        SubmissionSource::InApp {},
        with_kind,
    )
    .expect("record");
    let (kind, warning) = resolve_kind(&carried, &default_kind);
    assert_eq!(kind.as_str(), "person");
    assert!(warning.is_none());

    let mut bad_kind = payload();
    bad_kind["kind"] = json!("Not Valid Kind!");
    let invalid = QueueRecord::new(
        "rec-l".to_string(),
        ts(1),
        SubmissionSource::InApp {},
        bad_kind,
    )
    .expect("record");
    let (kind, warning) = resolve_kind(&invalid, &default_kind);
    assert_eq!(kind, default_kind, "invalid payload kind falls back");
    assert!(
        warning
            .expect("loud fallback")
            .contains("not a valid kind id"),
        "the fallback is recorded, never silent"
    );
}

#[test]
fn validate_record_reports_template_fit_like_the_plan_path() {
    let template = plan_template();
    let kind = cn_model::KindId::new("person").expect("kind");
    let clean = validate_record(&record(), &template, &kind).expect("report");
    assert!(clean.errors.is_empty(), "valid payload: {clean:?}");

    let mut bare = payload();
    bare["fields"] = json!({});
    let missing = QueueRecord::new("rec-m".to_string(), ts(1), SubmissionSource::InApp {}, bare)
        .expect("record");
    let flagged = validate_record(&missing, &template, &kind).expect("report");
    assert!(
        !flagged.errors.is_empty(),
        "missing required display_name surfaces as a finding"
    );

    let mut tampered = record();
    tampered.payload["fields"]["display_name"] = json!("Tampered");
    assert!(
        validate_record(&tampered, &template, &kind).is_err(),
        "checksum failure is a typed error, not a validation finding"
    );
}

// --- round-1 amendments (2026-07-25 adversarial round) ---

#[test]
fn empty_reject_reason_is_illegal_in_core() {
    let record = record();
    let sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let message = decision(
        "d-empty",
        1,
        ReviewState::Pending,
        0,
        DecisionType::Reject {
            reason: "   ".to_string(),
        },
    );
    let verdict = admit(&sidecar, &message).expect("admission runs");
    assert!(
        matches!(verdict, AdmissionVerdict::Illegal { .. }),
        "core enforces the required reason; UI prompts are advisory: {verdict:?}"
    );
}

#[test]
fn marker_with_no_files_is_lost_decision_state() {
    let found = FoundRecord {
        record_id: "rec-gone".to_string(),
        payload: None,
        payload_corrupt: false,
        sidecar: FoundSidecar::Missing,
        marker_present: true,
        pending_decisions: 0,
    };
    assert_eq!(
        classify(&found).expect("classify"),
        RecoveryAction::HaltLostDecisionState {
            record_id: "rec-gone".to_string()
        },
        "review provably began; nothing remains to reconstruct from"
    );
}

#[test]
fn validate_submission_rejects_hostile_and_malformed_payloads() {
    let template = plan_template();

    let clean = validate_submission(&payload(), &template);
    assert!(
        clean.errors.is_empty(),
        "fixture payload is clean: {clean:?}"
    );

    let mut unaffirmed = payload();
    unaffirmed["consent"]["consent_affirmed"] = json!(false);
    assert!(
        validate_submission(&unaffirmed, &template)
            .errors
            .iter()
            .any(|e| e.contains("consent_affirmed")),
        "unaffirmed consent is a rejecting finding (D-030)"
    );

    let mut unknown_top = payload();
    unknown_top["tracking_pixel"] = json!("x");
    assert!(
        validate_submission(&unknown_top, &template)
            .errors
            .iter()
            .any(|e| e.contains("unknown top-level")),
        "top-level allowlist enforced"
    );

    let mut unknown_field = payload();
    unknown_field["fields"]["shoe_size"] = json!("12");
    assert!(
        validate_submission(&unknown_field, &template)
            .errors
            .iter()
            .any(|e| e.contains("shoe_size")),
        "field allowlist against the template enforced"
    );

    let mut oversized = payload();
    oversized["fields"]["display_name"] = json!("x".repeat(PAYLOAD_TEXT_MAX + 1));
    assert!(
        validate_submission(&oversized, &template)
            .errors
            .iter()
            .any(|e| e.contains("display_name")),
        "length cap enforced"
    );

    let mut hostile = payload();
    hostile["fields"]["display_name"] = json!("Jo\u{0007}Doe");
    assert!(
        validate_submission(&hostile, &template)
            .errors
            .iter()
            .any(|e| e.contains("control")),
        "control characters rejected"
    );

    let mut no_version = payload();
    no_version
        .as_object_mut()
        .expect("obj")
        .remove("submission_version");
    assert!(
        !validate_submission(&no_version, &template)
            .errors
            .is_empty(),
        "missing submission_version rejected"
    );
}

#[test]
fn verify_persisted_plan_accepts_real_plans_and_rejects_tampering() {
    let record = remote_record();
    let template = plan_template();
    let context = plan_context();
    let plan = plan_approval(&record, &template, &context, &mut deterministic_ids()).expect("plan");

    let ops = verify_persisted_plan(&plan.plan_ref).expect("self-consistent plan verifies");
    assert_eq!(ops.len(), 1);

    // Drifted op id list.
    let mut drifted = plan.plan_ref.clone();
    drifted.op_ids[0] = "00000000-0000-0000-0000-00000000dead".to_string();
    assert!(
        verify_persisted_plan(&drifted).is_err(),
        "op-id drift detected"
    );

    // Unrelated batch digest.
    let mut wrong_batch = plan.plan_ref.clone();
    wrong_batch.batch_digest = "0".repeat(64);
    assert!(
        verify_persisted_plan(&wrong_batch).is_err(),
        "batch_digest must recompute from the ops"
    );

    // Tampered op bytes with a stale digest list.
    let mut tampered = plan.plan_ref.clone();
    tampered.ops[0]["recorded_at"] = json!(999_999);
    assert!(
        verify_persisted_plan(&tampered).is_err(),
        "per-op digest mismatch detected"
    );

    // Length disagreement.
    let mut short = plan.plan_ref.clone();
    short.per_op_digests.clear();
    assert!(verify_persisted_plan(&short).is_err(), "length checks hold");
}

#[test]
fn nested_unknown_minor_fields_survive_sidecar_rewrite() {
    let record = record();
    let mut sidecar = ReviewSidecar::initial(&record).expect("sidecar");
    let message = decision("d-n", 1, ReviewState::Pending, 0, DecisionType::Approve);
    let verdict = admit(&sidecar, &message).expect("verdict");
    apply_verdict(&mut sidecar, &verdict).expect("admitted");

    // A newer-minor writer added fields inside the history entry.
    let mut value = serde_json::to_value(&sidecar).expect("value");
    value["history"][0]["future_note"] = json!("kept");
    let mut parsed: ReviewSidecar = serde_json::from_value(value).expect("parses");
    parsed.seal_rewrite().expect("reseal");
    let round_tripped = serde_json::to_value(&parsed).expect("value");
    assert_eq!(
        round_tripped["history"][0]["future_note"], "kept",
        "nested unknown-minor data survives the rewrite (I7)"
    );
}

#[test]
fn canonical_digest_golden_vectors_pin_the_byte_domain() {
    // The canonical domain is the sorted-key JSON tree (ADR-005 D4).
    // These vectors pin it: any serializer or schema drift that changes
    // the bytes fails HERE, not in a live queue.
    let simple = json!({ "b": 1, "a": { "d": true, "c": "x" } });
    assert_eq!(
        canonical_digest(&simple).expect("digest"),
        "8db12c99f4ab6cb9510437ba655f2ce027fe900d0c2bcb0d9f4a8ad05ce2cda9",
        "sorted-key object digest"
    );

    let record = QueueRecord::new(
        "golden-rec".to_string(),
        ts(42),
        SubmissionSource::InApp {},
        json!({ "submission_id": "sub-g", "fields": { "display_name": "Golden Vector" } }),
    )
    .expect("record");
    assert_eq!(
        record.record_checksum, "6200bcf8723b2efec194fc3e90e58489c73fe9ee6742d51ac0cf9f1001587bec",
        "queue-record checksum domain"
    );
    assert_eq!(
        record.payload_hash, "ca230fbb029d531f1615ba8f36f9075eb915d9070ecab75f354770194536a2eb",
        "payload digest domain"
    );
}
