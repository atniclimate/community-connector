use serde::{Deserialize, Serialize};

use crate::{
    GroupId, ModelError, PersonId, ProvenanceEnvelope, Timestamp, TrustGrantId,
    model_schema_version,
};

/// Scope for a trust grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustScope {
    /// Trust applies to all groups.
    All,
    /// Trust applies to one group.
    Group(GroupId),
}

/// Owner-managed and audited trust grant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustGrant {
    /// Trust grant identifier.
    pub id: TrustGrantId,
    /// Granting person.
    pub grantor: PersonId,
    /// Trusted person.
    pub grantee: PersonId,
    /// Trust scope.
    pub scope: TrustScope,
    /// Grant timestamp.
    pub granted_at: Timestamp,
    /// Optional revocation timestamp.
    pub revoked_at: Option<Timestamp>,
    /// Trust grant provenance.
    pub provenance: ProvenanceEnvelope,
    /// Trust grant schema version.
    pub schema_version: semver::Version,
}

impl TrustGrant {
    /// Creates an active trust grant.
    pub fn new(
        id: TrustGrantId,
        grantor: PersonId,
        grantee: PersonId,
        scope: TrustScope,
        granted_at: Timestamp,
        provenance: ProvenanceEnvelope,
    ) -> Self {
        Self {
            id,
            grantor,
            grantee,
            scope,
            granted_at,
            revoked_at: None,
            provenance,
            schema_version: model_schema_version(),
        }
    }

    /// Revokes this grant once.
    pub fn revoke(&mut self, at: Timestamp) -> Result<(), ModelError> {
        if self.revoked_at.is_some() {
            return Err(ModelError::AlreadyRevoked);
        }
        if at < self.granted_at {
            return Err(ModelError::RevokedBeforeGranted);
        }
        self.revoked_at = Some(at);
        Ok(())
    }

    /// Returns true when this grant is not revoked.
    pub fn is_active(&self) -> bool {
        self.revoked_at.is_none()
    }
}
