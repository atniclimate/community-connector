use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    AttrId, AttributeInstance, Circle, EntityId, GroupId, KindId, Lifecycle, PersonId,
    ProvenanceEnvelope, SensitivityTier, model_schema_version,
};

/// Entity record with provenance and sensitivity tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    /// Entity identifier.
    pub id: EntityId,
    /// Owning group identifier.
    pub group_id: GroupId,
    /// Entity kind identifier.
    pub kind: KindId,
    /// Entity attributes.
    pub attributes: BTreeMap<AttrId, AttributeInstance>,
    /// Optional owner person.
    pub owner: Option<PersonId>,
    /// Presence visibility.
    pub presence_visibility: Circle,
    /// Entity lifecycle.
    pub lifecycle: Lifecycle,
    /// Entity provenance.
    pub provenance: ProvenanceEnvelope,
    /// Entity sensitivity tier.
    pub tier: SensitivityTier,
    /// Entity schema version.
    pub schema_version: semver::Version,
}

impl Entity {
    /// Creates an active entity with no attributes.
    pub fn new(
        id: EntityId,
        group_id: GroupId,
        kind: KindId,
        presence_visibility: Circle,
        provenance: ProvenanceEnvelope,
        tier: SensitivityTier,
    ) -> Self {
        Self {
            id,
            group_id,
            kind,
            attributes: BTreeMap::new(),
            owner: None,
            presence_visibility,
            lifecycle: Lifecycle::Active,
            provenance,
            tier,
            schema_version: model_schema_version(),
        }
    }
}
