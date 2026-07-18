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
    /// Pilot facilitator: entry/import and story authoring authority
    /// without governance powers (D-028, integration plan 6.3).
    Facilitator,
}

#[cfg(test)]
mod tests {
    use crate::{
        ActorRef, Membership, MembershipId, Origin, PersonId, ProvenanceEnvelope, Timestamp,
    };
    use std::str::FromStr;

    use super::GroupRole;

    fn person() -> PersonId {
        PersonId::from_str("00000000-0000-0000-0000-000000000001").expect("person id")
    }

    fn membership(role: GroupRole) -> Membership {
        let who = person();
        let provenance =
            ProvenanceEnvelope::new(Origin::Authored, ActorRef::Human(who), who, Timestamp(1))
                .expect("valid provenance");
        Membership::new(
            MembershipId::from_str("00000000-0000-0000-0000-000000000002").expect("membership id"),
            crate::GroupId::from_str("00000000-0000-0000-0000-000000000003").expect("group id"),
            who,
            role,
            provenance,
        )
    }

    #[test]
    fn facilitator_membership_round_trips_as_snake_case() {
        let original = membership(GroupRole::Facilitator);
        let json = serde_json::to_string(&original).expect("serialize membership");
        assert!(
            json.contains("\"role\":\"facilitator\""),
            "facilitator must be snake_case on the wire: {json}"
        );
        let decoded: Membership = serde_json::from_str(&json).expect("deserialize membership");
        assert_eq!(decoded, original);
    }

    #[test]
    fn unknown_future_role_is_rejected_loudly() {
        let json = serde_json::to_string(&membership(GroupRole::Facilitator))
            .expect("serialize membership")
            .replace("\"role\":\"facilitator\"", "\"role\":\"coordinator\"");
        let err = serde_json::from_str::<Membership>(&json)
            .expect_err("unknown role must not deserialize");
        assert!(
            err.to_string().contains("coordinator"),
            "rejection must name the unknown role: {err}"
        );
    }
}
