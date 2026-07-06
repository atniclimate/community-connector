use serde::{Deserialize, Serialize};

use crate::{
    Circle, EntityId, GroupId, Lifecycle, ProvenanceEnvelope, SensitivityTier, StoryId,
    model_schema_version,
};

/// Narrative story over graph entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Story {
    /// Story identifier.
    pub id: StoryId,
    /// Owning group identifier.
    pub group_id: GroupId,
    /// Story title.
    pub title: String,
    /// Ordered story steps.
    pub steps: Vec<StoryStep>,
    /// Story visibility.
    pub visibility: Circle,
    /// Story lifecycle.
    pub lifecycle: Lifecycle,
    /// Story provenance.
    pub provenance: ProvenanceEnvelope,
    /// Story sensitivity tier.
    pub tier: SensitivityTier,
    /// Story schema version.
    pub schema_version: semver::Version,
}

impl Story {
    /// Creates an active story with required provenance and tier.
    pub fn new(
        id: StoryId,
        group_id: GroupId,
        title: impl Into<String>,
        steps: Vec<StoryStep>,
        visibility: Circle,
        provenance: ProvenanceEnvelope,
        tier: SensitivityTier,
    ) -> Self {
        Self {
            id,
            group_id,
            title: title.into(),
            steps,
            visibility,
            lifecycle: Lifecycle::Active,
            provenance,
            tier,
            schema_version: model_schema_version(),
        }
    }
}

/// Single narrated story step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoryStep {
    /// Referenced entity.
    pub entity: EntityId,
    /// Human-readable narration.
    pub narration: String,
}
