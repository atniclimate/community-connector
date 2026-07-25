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
    };
    Ok(ApprovalPlan {
        ops,
        plan_ref,
        validation,
    })
}
