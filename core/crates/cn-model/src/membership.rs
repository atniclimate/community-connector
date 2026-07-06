use serde::{Deserialize, Serialize};

use crate::{
    GroupId, GroupRole, Lifecycle, MembershipId, PersonId, ProvenanceEnvelope, model_schema_version,
};

/// Core group membership primitive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Membership {
    /// Membership identifier.
    pub id: MembershipId,
    /// Group identifier.
    pub group_id: GroupId,
    /// Person identifier.
    pub person: PersonId,
    /// Membership role.
    pub role: GroupRole,
    /// Membership lifecycle.
    pub lifecycle: Lifecycle,
    /// Membership provenance.
    pub provenance: ProvenanceEnvelope,
    /// Membership schema version.
    pub schema_version: semver::Version,
}

impl Membership {
    /// Creates an active membership.
    pub fn new(
        id: MembershipId,
        group_id: GroupId,
        person: PersonId,
        role: GroupRole,
        provenance: ProvenanceEnvelope,
    ) -> Self {
        Self {
            id,
            group_id,
            person,
            role,
            lifecycle: Lifecycle::Active,
            provenance,
            schema_version: model_schema_version(),
        }
    }
}
