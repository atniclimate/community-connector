use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    AttrId, AttributeInstance, Circle, EdgeId, EntityId, GroupId, KindId, Lifecycle, ModelError,
    ProvenanceEnvelope, SensitivityTier, model_schema_version,
};

/// Edge record with provenance and sensitivity tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// Edge identifier.
    pub id: EdgeId,
    /// Owning group identifier.
    pub group_id: GroupId,
    /// Edge kind identifier.
    pub kind: KindId,
    /// Source entity.
    pub from: EntityId,
    /// Target entity.
    pub to: EntityId,
    /// Whether the edge is directed.
    pub directed: bool,
    /// Optional finite edge weight.
    pub weight: Option<f64>,
    /// Edge attributes.
    pub attributes: BTreeMap<AttrId, AttributeInstance>,
    /// Edge visibility.
    pub visibility: Circle,
    /// Edge lifecycle.
    pub lifecycle: Lifecycle,
    /// Edge provenance.
    pub provenance: ProvenanceEnvelope,
    /// Edge sensitivity tier.
    pub tier: SensitivityTier,
    /// Edge schema version.
    pub schema_version: semver::Version,
}

impl Edge {
    /// Creates an active edge with no attributes and no weight.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: EdgeId,
        group_id: GroupId,
        kind: KindId,
        from: EntityId,
        to: EntityId,
        directed: bool,
        visibility: Circle,
        provenance: ProvenanceEnvelope,
        tier: SensitivityTier,
    ) -> Self {
        Self {
            id,
            group_id,
            kind,
            from,
            to,
            directed,
            weight: None,
            attributes: BTreeMap::new(),
            visibility,
            lifecycle: Lifecycle::Active,
            provenance,
            tier,
            schema_version: model_schema_version(),
        }
    }

    /// Sets a finite weight or clears it.
    pub fn set_weight(&mut self, weight: Option<f64>) -> Result<(), ModelError> {
        if weight.is_some_and(|value| !value.is_finite()) {
            Err(ModelError::NonFiniteWeight)
        } else {
            self.weight = weight;
            Ok(())
        }
    }
}
