use std::collections::BTreeMap;

use cn_graph::PathConstraints;
use cn_model::{AttrId, AttributeValue, Circle, EntityId, KindId, SensitivityTier};
use cn_perm::Projection;
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
