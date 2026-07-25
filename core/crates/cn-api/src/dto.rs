use std::collections::BTreeMap;

use cn_graph::PathConstraints;
use cn_model::{AttrId, AttributeValue, Circle, EntityId, KindId, SensitivityTier};
use cn_perm::{DetailProvenance, Projection};
use cn_store::{StoreReport, SubmitOutcome};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(crate) struct CoreInfo {
    pub(crate) core_version: &'static str,
    pub(crate) boundary_version: &'static str,
    pub(crate) supported_schema_majors: Vec<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct LoadReport {
    pub(crate) revision: u64,
    pub(crate) report: StoreReport,
}

#[derive(Debug, Serialize)]
pub(crate) struct SubmitReport {
    pub(crate) revision: u64,
    pub(crate) outcomes: Vec<SubmitOutcome>,
    pub(crate) report: StoreReport,
}

#[derive(Debug, Serialize)]
pub(crate) struct EntityDetail {
    pub(crate) id: EntityId,
    pub(crate) kind: KindId,
    pub(crate) owner_is_viewer: bool,
    pub(crate) tier: SensitivityTier,
    pub(crate) provenance: DetailProvenance,
    pub(crate) attributes: BTreeMap<AttrId, DetailValue>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DetailValue {
    pub(crate) value: AttributeValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) visibility: Option<Circle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tier: Option<SensitivityTier>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PathRequest {
    pub(crate) from: EntityId,
    pub(crate) to: EntityId,
    #[serde(default)]
    pub(crate) constraints: PathConstraints,
}

#[derive(Debug, Deserialize)]
pub(crate) struct NeighborhoodRequest {
    pub(crate) center: EntityId,
    pub(crate) hops: usize,
    #[serde(default)]
    pub(crate) constraints: PathConstraints,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ExportOptions {
    #[serde(default)]
    pub(crate) kinds: Option<Vec<KindId>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExportSnapshot {
    pub(crate) boundary_version: &'static str,
    pub(crate) schema_version: semver::Version,
    pub(crate) template: cn_schema::GroupTemplate,
    pub(crate) projection: Projection,
}

/// Intake record validation result (blueprint section 3): the resolved
/// kind, the authoritative template-fit report, and any kind-fallback
/// warning.
#[derive(Debug, Serialize)]
pub(crate) struct IntakeValidation {
    pub(crate) kind: KindId,
    /// Raw submission-schema findings (allowlist, consent, caps; round-1
    /// F9) - authoritative alongside the entity template-fit report.
    pub(crate) submission: cn_ingest::SubmissionFindings,
    pub(crate) report: cn_schema::ValidationReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) kind_warning: Option<String>,
}

/// Intake dedup classification result.
#[derive(Debug, Serialize)]
pub(crate) struct IntakeDedup {
    pub(crate) verdict: cn_ingest::DedupVerdict,
}

/// Near-duplicate request: the candidate payload, the other queue
/// payloads, and the template-driven name/affiliation attribute ids. The
/// graph side is NEVER supplied by the caller - it is the viewer's
/// projection, computed in-core (the no-leak property, blueprint
/// section 6).
#[derive(Debug, Deserialize)]
pub(crate) struct NearDupRequest {
    pub(crate) payload: serde_json::Value,
    #[serde(default)]
    pub(crate) queue_payloads: Vec<(String, serde_json::Value)>,
    pub(crate) name_attrs: Vec<AttrId>,
    #[serde(default)]
    pub(crate) affiliation_attrs: Vec<AttrId>,
}

/// Staged pair returned by the pure record builder: the app writes these
/// bytes create-only over FSA; it never computes a checksum itself.
#[derive(Debug, Serialize)]
pub(crate) struct StagedPair {
    pub(crate) record: cn_ingest::QueueRecord,
    pub(crate) sidecar: cn_ingest::ReviewSidecar,
}

/// Decision-message build request (D-073): identity (decision_id UUIDv7)
/// and format version are core-generated; the app supplies only the CAS
/// premise it observed and the decision content.
#[derive(Debug, Deserialize)]
pub(crate) struct DecisionRequest {
    pub(crate) record_id: String,
    pub(crate) payload_digest: String,
    pub(crate) expected_review_state: cn_ingest::ReviewState,
    pub(crate) expected_decision_generation: u64,
    pub(crate) decision: cn_ingest::DecisionType,
    pub(crate) reviewer: String,
    pub(crate) decided_at_ms: i64,
}
