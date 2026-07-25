//! Approval planning (ADR-005 D4/D5): turns an approved queue record into
//! an ops batch with preassigned ids, the pre-link batch digest, and the
//! intake provenance block built INTO the modeled values.
//!
//! Invoked natively by `cn intake apply` on admitting an approve decision.
//! Attributes ride inside the `EntityCreate` entity (fold validates
//! required attributes at create time, so a bare create followed by
//! `AttributeSet` ops would quarantine - a deliberate deviation from the
//! blueprint's earlier op-list sketch, recorded here).

use serde_json::Value;

use cn_model::{
    ActorRef, AttributeInstance, AttributeValue, Entity, EntityId, GroupId, INTAKE_BLOCK_VERSION,
    IntakeProvenance, KindId, Origin, PersonId, ProvenanceEnvelope, SensitivityTier, Timestamp,
};
use cn_schema::{AttrType, GroupTemplate, ValidationReport, validate_entity};
use cn_store::{Hlc, OpKind, Operation, SortKey};

use crate::record::{ApprovalPlanRef, QueueRecord, SubmissionSource};
use crate::version::{IngestError, canonical_digest};

/// Software-agent actor identity, stable regardless of final packaging
/// (ADR-005 D5).
pub const INTAKE_ACTOR_ID: &str = "cn-intake/0.1.0";

/// The pilot form's default entity kind (blueprint sections 2-4). The
/// single source for both the CLI's `--kind` default and the read-only
/// facade's kind resolution.
pub const DEFAULT_PILOT_KIND: &str = "person";

/// Resolves the entity kind for a record: a payload-carried `kind` field
/// wins, else the caller's default. Returns an accompanying warning when a
/// payload kind was invalid and the default was used - template validation
/// and the seam preflight stay authoritative either way, so a wrong kind
/// surfaces loudly downstream, never as a silent write.
pub fn resolve_kind(record: &QueueRecord, default_kind: &KindId) -> (KindId, Option<String>) {
    match record.payload.get("kind").and_then(Value::as_str) {
        Some(raw) => match KindId::new(raw) {
            Ok(kind) => (kind, None),
            Err(_) => (
                default_kind.clone(),
                Some(format!(
                    "record {}: payload kind '{raw}' is not a valid kind id; using \
                     default '{default_kind}'",
                    record.record_id
                )),
            ),
        },
        None => (default_kind.clone(), None),
    }
}

/// Submission-schema findings (blueprint section 2; round-1 F9): the
/// typed report over the RAW payload - allowlist, versions, consent
/// shape, length caps, control characters - which template-fit
/// validation of the built entity cannot see.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct SubmissionFindings {
    /// Rejecting findings: an approve of a record carrying any of these
    /// must not reach the durable seam (validation preflight).
    pub errors: Vec<String>,
    /// Non-rejecting findings, surfaced for review.
    pub warnings: Vec<String>,
}

/// Payload text caps (hostile-content guardrails; ADR-005 D4).
pub const PAYLOAD_TEXT_MAX: usize = 2_000;
const KNOWN_TOP_LEVEL: &[&str] = &[
    "submission_version",
    "submission_id",
    "form_version",
    "kind",
    "consent",
    "captured_at",
    "fields",
];

fn has_disallowed_controls(text: &str) -> bool {
    text.chars()
        .any(|c| (c.is_control() && c != '\n' && c != '\t') || c == '\u{2028}' || c == '\u{2029}')
}

fn check_text(findings: &mut SubmissionFindings, what: &str, text: &str) {
    if text.len() > PAYLOAD_TEXT_MAX {
        findings
            .errors
            .push(format!("{what}: exceeds {PAYLOAD_TEXT_MAX} bytes"));
    }
    if has_disallowed_controls(text) {
        findings
            .errors
            .push(format!("{what}: contains disallowed control characters"));
    }
}

/// Validates the RAW submission payload against the submission schema
/// (allowlist, versions, consent, caps, control characters). Pure; the
/// caller decides what staging state the findings produce. An approve of
/// a record with `errors` must never reach the durable seam.
pub fn validate_submission(payload: &Value, template: &GroupTemplate) -> SubmissionFindings {
    let mut findings = SubmissionFindings::default();
    let Some(object) = payload.as_object() else {
        findings
            .errors
            .push("payload is not a JSON object".to_string());
        return findings;
    };

    for key in object.keys() {
        if !KNOWN_TOP_LEVEL.contains(&key.as_str()) {
            findings
                .errors
                .push(format!("unknown top-level field '{key}'"));
        }
    }
    match object.get("submission_version").and_then(Value::as_str) {
        None => findings
            .errors
            .push("submission_version missing or not a string".to_string()),
        Some(raw) => match semver::Version::parse(raw) {
            Err(_) => findings
                .errors
                .push(format!("submission_version '{raw}' is not semver")),
            Ok(version) if version.major != 0 => findings
                .errors
                .push(format!("unknown submission_version major {version}")),
            Ok(_) => {}
        },
    }
    match object.get("submission_id").and_then(Value::as_str) {
        None => findings
            .errors
            .push("submission_id missing or not a string".to_string()),
        Some(id) if id.trim().is_empty() => {
            findings.errors.push("submission_id is empty".to_string());
        }
        Some(id) => check_text(&mut findings, "submission_id", id),
    }
    if object.get("form_version").and_then(Value::as_str).is_none() {
        findings
            .errors
            .push("form_version missing or not a string".to_string());
    }
    match object.get("consent") {
        None => findings.errors.push("consent block missing".to_string()),
        Some(consent) => {
            if consent.get("consent_affirmed").and_then(Value::as_bool) != Some(true) {
                findings
                    .errors
                    .push("consent_affirmed is not true - nothing proceeds without the affirmation (D-030)".to_string());
            }
            if consent
                .get("consent_text_digest")
                .and_then(Value::as_str)
                .is_none_or(|digest| digest.is_empty())
            {
                findings
                    .errors
                    .push("consent_text_digest missing or empty".to_string());
            }
            if consent
                .get("consent_affirmed_at")
                .and_then(Value::as_i64)
                .is_none()
            {
                findings
                    .errors
                    .push("consent_affirmed_at missing or not an integer".to_string());
            }
        }
    }
    if object.get("captured_at").and_then(Value::as_i64).is_none() {
        findings
            .warnings
            .push("captured_at missing or not an integer".to_string());
    }

    // Field allowlist against the template (payload kind rule, D-070.5).
    let default_kind = KindId::new(DEFAULT_PILOT_KIND).expect("valid const");
    let kind_id = object
        .get("kind")
        .and_then(Value::as_str)
        .and_then(|raw| KindId::new(raw).ok())
        .unwrap_or(default_kind);
    let kind_def = template.kinds.iter().find(|k| k.id == kind_id);
    match (kind_def, object.get("fields")) {
        (_, None) => findings.errors.push("fields object missing".to_string()),
        (None, _) => findings
            .errors
            .push(format!("kind '{kind_id}' is not in the group template")),
        (Some(kind_def), Some(fields)) => match fields.as_object() {
            None => findings
                .errors
                .push("fields is not a JSON object".to_string()),
            Some(fields) => {
                for (field, value) in fields {
                    let Some(_def) = kind_def.attributes.iter().find(|a| a.id.as_str() == field)
                    else {
                        findings.errors.push(format!(
                            "field '{field}' is not a template attribute of '{kind_id}'"
                        ));
                        continue;
                    };
                    match value {
                        Value::String(text) => check_text(&mut findings, field, text),
                        Value::Array(items) => {
                            for (index, item) in items.iter().enumerate() {
                                if let Value::String(text) = item {
                                    check_text(&mut findings, &format!("{field}[{index}]"), text);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        },
    }
    findings
}

/// Verifies a persisted approval plan's cross-field integrity before
/// recovery trusts it (round-1 F6): lengths agree, op ids are unique and
/// equal the ops' own ids, every per-op digest matches its op's bytes,
/// and the pre-link batch digest recomputes from the ops with every
/// intake `batch_digest` blanked. Returns the parsed ops on success.
pub fn verify_persisted_plan(plan: &ApprovalPlanRef) -> Result<Vec<Operation>, IngestError> {
    if plan.ops.is_empty()
        || plan.ops.len() != plan.per_op_digests.len()
        || plan.ops.len() != plan.op_ids.len()
    {
        return Err(IngestError::Serialize(format!(
            "persisted plan is malformed: {} ops, {} digests, {} ids",
            plan.ops.len(),
            plan.per_op_digests.len(),
            plan.op_ids.len()
        )));
    }
    let ops: Vec<Operation> = plan
        .ops
        .iter()
        .map(|value| serde_json::from_value(value.clone()))
        .collect::<Result<_, _>>()
        .map_err(|err| IngestError::Serialize(format!("persisted plan ops do not parse: {err}")))?;
    let mut seen = std::collections::BTreeSet::new();
    for (index, op) in ops.iter().enumerate() {
        if plan.op_ids[index] != op.op_id.to_string() {
            return Err(IngestError::ChecksumMismatch {
                what: format!("plan op id {} disagrees with its op", plan.op_ids[index]),
            });
        }
        if !seen.insert(op.op_id) {
            return Err(IngestError::ChecksumMismatch {
                what: format!("plan op id {} is duplicated", op.op_id),
            });
        }
        if canonical_digest(op)? != plan.per_op_digests[index] {
            return Err(IngestError::ChecksumMismatch {
                what: format!("plan digest for op {}", op.op_id),
            });
        }
    }
    // Recompute the pre-link projection: canonical ops with every intake
    // batch_digest EMPTY (ADR-005 D5 non-circular definition).
    let mut prelink = ops.clone();
    for op in &mut prelink {
        if let OpKind::EntityCreate { entity } = &mut op.kind {
            if let Some(block) = entity.provenance.intake().cloned() {
                let mut cleared = block;
                cleared.batch_digest = String::new();
                entity.provenance.set_intake(cleared);
            }
            for instance in entity.attributes.values_mut() {
                if let Some(block) = instance.provenance.intake().cloned() {
                    let mut cleared = block;
                    cleared.batch_digest = String::new();
                    instance.provenance.set_intake(cleared);
                }
            }
        }
    }
    if canonical_digest(&prelink)? != plan.batch_digest {
        return Err(IngestError::ChecksumMismatch {
            what: "plan batch_digest does not recompute from its ops".to_string(),
        });
    }
    Ok(ops)
}

/// The authoritative template-fit validation for a queue record, computed
/// by building EXACTLY the entity `plan_approval` would build (throwaway
/// ids and plan context, discarded ops): the review UI's report can never
/// diverge from the apply-time report (I2). Verifies version and checksum
/// first via `record.verify()` inside the plan path.
pub fn validate_record(
    record: &QueueRecord,
    template: &GroupTemplate,
    kind: &KindId,
) -> Result<ValidationReport, IngestError> {
    let nil = "00000000-0000-0000-0000-000000000000";
    let context = PlanContext {
        group_id: nil
            .parse()
            .map_err(|_| IngestError::Serialize("nil group id".to_string()))?,
        facilitator: nil
            .parse()
            .map_err(|_| IngestError::Serialize("nil person id".to_string()))?,
        kind: kind.clone(),
        now_ms: 0,
        template_version: semver::Version::new(0, 0, 0),
    };
    let mut n: u128 = 0;
    let plan = plan_approval(record, template, &context, &mut || {
        n += 1;
        uuid::Uuid::from_u128(n)
    })?;
    Ok(plan.validation)
}

/// Everything the plan needs besides the record and template.
#[derive(Debug, Clone)]
pub struct PlanContext {
    pub group_id: GroupId,
    /// The reviewing facilitator: responsible_human on every op (D-056.2).
    pub facilitator: PersonId,
    /// The template kind this submission creates (pilot form: person).
    pub kind: KindId,
    /// Approval wall-clock milliseconds (drives Hlc and recorded_at).
    pub now_ms: i64,
    /// The group's template version for the op envelope.
    pub template_version: semver::Version,
}

/// The generated plan: ops ready for the durable seam, the sidecar plan
/// reference, and the authoritative validation report.
#[derive(Debug, Clone)]
pub struct ApprovalPlan {
    pub ops: Vec<Operation>,
    pub plan_ref: ApprovalPlanRef,
    pub validation: ValidationReport,
}

fn attr_value_from_json(attr_type: &AttrType, value: &Value) -> Option<AttributeValue> {
    match (attr_type, value) {
        (AttrType::Text, Value::String(s)) => Some(AttributeValue::Text(s.clone())),
        (AttrType::Enum, Value::String(s)) => Some(AttributeValue::Enum(s.clone())),
        (AttrType::Number, Value::Number(n)) => n.as_f64().map(AttributeValue::Number),
        (AttrType::Tags, Value::Array(items)) => {
            let tags: Option<std::collections::BTreeSet<String>> = items
                .iter()
                .map(|item| item.as_str().map(str::to_string))
                .collect();
            tags.map(AttributeValue::Tags)
        }
        // Date/Geo/Link/Media are not in the pilot remote field set; a
        // template requiring them surfaces through validate_entity below.
        _ => None,
    }
}

/// Builds the approval plan for one approved record: one unowned,
/// facilitator-created entity of `context.kind`, attributes mapped from
/// the payload's `fields` per the template, T1 tier (D-034), intake
/// provenance block on the entity AND every attribute instance (D5).
///
/// `id_gen` supplies UUIDs (production: `uuid::Uuid::now_v7`; tests:
/// deterministic) - ids are generated HERE, once; recovery reuses the
/// persisted plan and never regenerates them.
pub fn plan_approval(
    record: &QueueRecord,
    template: &GroupTemplate,
    context: &PlanContext,
    id_gen: &mut dyn FnMut() -> uuid::Uuid,
) -> Result<ApprovalPlan, IngestError> {
    record.verify()?;

    let actor = ActorRef::Agent {
        agent_id: INTAKE_ACTOR_ID.to_string(),
    };
    let now = Timestamp(context.now_ms);

    // The intake block, pre-link: batch_digest empty until the projection
    // digest is computed (ADR-005 D5 non-circular definition).
    let consent = record.payload.get("consent");
    let intake_block = |batch_digest: String| IntakeProvenance {
        intake_block_version: semver::Version::parse(INTAKE_BLOCK_VERSION)
            .map_err(|err| IngestError::Serialize(err.to_string()))
            .expect("valid const"),
        record_id: record.record_id.clone(),
        receipt_id: match &record.source {
            SubmissionSource::Remote { receipt_id, .. } => Some(receipt_id.clone()),
            SubmissionSource::InApp {} => None,
        },
        submission_id: record
            .payload
            .get("submission_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        form_version: record
            .payload
            .get("form_version")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        consent_text_digest: consent
            .and_then(|c| c.get("consent_text_digest"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        consent_affirmed: consent
            .and_then(|c| c.get("consent_affirmed"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        consent_affirmed_at: Timestamp(
            consent
                .and_then(|c| c.get("consent_affirmed_at"))
                .and_then(Value::as_i64)
                .unwrap_or(0),
        ),
        payload_digest: record.payload_hash.clone(),
        batch_digest,
    };

    let envelope = || -> Result<ProvenanceEnvelope, IngestError> {
        let mut env = ProvenanceEnvelope::new(
            Origin::Ingested {
                source: INTAKE_ACTOR_ID.to_string(),
            },
            actor.clone(),
            context.facilitator,
            now,
        )
        .map_err(|err| IngestError::Serialize(err.to_string()))?;
        env.set_intake(intake_block(String::new()));
        Ok(env)
    };

    // Build the entity: unowned, facilitator-created, T1 (D-034/D-056.2).
    let entity_id: EntityId = id_gen().to_string().parse().map_err(|_| {
        IngestError::Serialize("generated entity id was not a valid uuid".to_string())
    })?;
    let mut entity = Entity::new(
        entity_id,
        context.group_id,
        context.kind.clone(),
        cn_model::Circle::Group,
        envelope()?,
        SensitivityTier::T1,
    );

    let kind_def = template.kinds.iter().find(|k| k.id == context.kind);
    if let (Some(kind_def), Some(fields)) = (
        kind_def,
        record.payload.get("fields").and_then(Value::as_object),
    ) {
        for def in &kind_def.attributes {
            let Some(raw) = fields.get(def.id.as_str()) else {
                continue;
            };
            let Some(value) = attr_value_from_json(&def.attr_type, raw) else {
                continue; // type mismatch surfaces via validate_entity
            };
            let instance = AttributeInstance::new(value, def.default_visibility, envelope()?);
            entity.attributes.insert(def.id.clone(), instance);
        }
    }

    // Authoritative validation (I2: the core owns trust decisions).
    let validation = validate_entity(template, &entity);

    // One EntityCreate op carrying the populated entity.
    let op_id: cn_model::OpId = id_gen()
        .to_string()
        .parse()
        .map_err(|_| IngestError::Serialize("generated op id was not a valid uuid".to_string()))?;
    let mut ops = vec![Operation {
        op_id,
        group_id: context.group_id,
        actor: actor.clone(),
        responsible_human: context.facilitator,
        recorded_at: now,
        sort_key: SortKey::new(
            Hlc {
                wall_ms: context.now_ms,
                counter: 0,
            },
            &actor,
            op_id,
        ),
        template_version: context.template_version.clone(),
        kind: OpKind::EntityCreate { entity },
        schema_version: cn_model::model_schema_version(),
    }];

    // Pre-link batch digest: canonical bytes with every intake.batch_digest
    // EMPTY (exactly the state the ops are in right now).
    let batch_digest = canonical_digest(&ops)?;

    // Populate the digest into every intake block, then compute FINAL
    // per-op digests for the durable seam.
    for op in &mut ops {
        if let OpKind::EntityCreate { entity } = &mut op.kind {
            let mut refreshed = envelope()?;
            refreshed.set_intake(intake_block(batch_digest.clone()));
            entity.provenance = refreshed;
            for instance in entity.attributes.values_mut() {
                let mut attr_env = envelope()?;
                attr_env.set_intake(intake_block(batch_digest.clone()));
                instance.provenance = attr_env;
            }
        }
    }
    let per_op_digests: Vec<String> = ops.iter().map(canonical_digest).collect::<Result<_, _>>()?;

    let plan_ref = ApprovalPlanRef {
        op_ids: ops.iter().map(|op| op.op_id.to_string()).collect(),
        per_op_digests,
        batch_digest,
        ops: ops
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()
            .map_err(|err| IngestError::Serialize(err.to_string()))?,
        extras: std::collections::BTreeMap::new(),
    };
    Ok(ApprovalPlan {
        ops,
        plan_ref,
        validation,
    })
}
