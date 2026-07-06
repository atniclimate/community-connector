use serde::{Deserialize, Serialize};

use crate::{
    GroupId, ModelError, ProvenanceEnvelope, SensitivityTier, TemplateId, model_schema_version,
};

/// Group record with a pinned template version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    /// Group identifier.
    pub id: GroupId,
    /// Group name.
    pub name: String,
    /// Template identifier.
    pub template_id: TemplateId,
    /// Pinned template version.
    pub template_version: semver::Version,
    /// Group provenance.
    pub provenance: ProvenanceEnvelope,
    /// Group sensitivity tier.
    pub tier: SensitivityTier,
    /// Group schema version.
    pub schema_version: semver::Version,
}

impl Group {
    /// Creates a group with a non-empty name.
    pub fn new(
        id: GroupId,
        name: impl Into<String>,
        template_id: TemplateId,
        template_version: semver::Version,
        provenance: ProvenanceEnvelope,
        tier: SensitivityTier,
    ) -> Result<Self, ModelError> {
        let name = name.into();
        if name.is_empty() {
            return Err(ModelError::EmptyName);
        }
        Ok(Self {
            id,
            name,
            template_id,
            template_version,
            provenance,
            tier,
            schema_version: model_schema_version(),
        })
    }
}

/// Group membership role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupRole {
    /// Regular member role.
    Member,
    /// Governance role.
    Governance,
}
