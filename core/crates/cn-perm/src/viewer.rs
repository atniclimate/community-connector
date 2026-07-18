use cn_model::{Circle, GroupRole, PersonId, TrustScope};
use cn_store::GroupState;
use crc32fast::Hasher;
use serde::{Deserialize, Serialize};

/// Viewer context used to resolve read-time permissions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ViewerContext {
    Anonymous,
    Person { person: PersonId },
}

/// Highest circle this viewer reaches for a value owned by `owner` in this
/// group (spec section 2; ADR-001 D5). In v0, Network means any authenticated
/// person, pending the future network ADR.
pub fn admissible_circle(
    state: &GroupState,
    viewer: &ViewerContext,
    owner: Option<&PersonId>,
) -> Circle {
    let ViewerContext::Person { person } = viewer else {
        return Circle::Public;
    };
    if owner.is_some_and(|owner| owner == person) {
        return Circle::Private;
    }
    if owner.is_some_and(|owner| active_trusts_owner(state, owner, person)) {
        return Circle::Trusted;
    }
    if active_roles_for(state, person).next().is_some() {
        return Circle::Group;
    }
    Circle::Network
}

/// Returns true when the viewer has active governance membership in the group
/// (spec section 3; ADR-002 A-B6).
pub fn is_governance(state: &GroupState, person: PersonId) -> bool {
    active_roles_for(state, &person).any(|role| role == GroupRole::Governance)
}

/// Returns true when the viewer has an active facilitator or governance
/// membership in the group (facilitator blueprint section 2; D-028). Grants
/// write authority only; it is never consulted on a read path.
pub fn is_facilitator_or_governance(state: &GroupState, person: PersonId) -> bool {
    active_roles_for(state, &person)
        .any(|role| matches!(role, GroupRole::Facilitator | GroupRole::Governance))
}

/// Returns true when the viewer has any active membership in the group
/// (spec section 2; ADR-001 D5).
pub fn is_active_member(state: &GroupState, person: PersonId) -> bool {
    active_roles_for(state, &person).next().is_some()
}

/// Returns true when the viewer's active roles include the Member role
/// specifically (facilitator blueprint section 4: the member custody rule
/// dominates for dual-role holders).
pub(crate) fn has_member_role(state: &GroupState, person: PersonId) -> bool {
    active_roles_for(state, &person).any(|role| role == GroupRole::Member)
}

/// Returns the projection cache fingerprint (ADR-003 D3 and round-1
/// fingerprint amendment).
pub fn viewer_fingerprint(state: &GroupState, viewer: &ViewerContext) -> String {
    let canonical = fingerprint_input(state, viewer);
    let mut hasher = Hasher::new();
    hasher.update(canonical.as_bytes());
    format!("{:08x}", hasher.finalize())
}

/// Returns the collision-free in-memory cache key: the canonical fingerprint
/// input itself rather than its 32-bit hash. Distinct viewer authorization
/// contexts can collide at the hashed `viewer_fingerprint` (it is a CRC32;
/// the 2026-07-17 adversarial round produced a concrete member/facilitator
/// collision pair), so per-viewer caches must never key on the hash alone.
/// This string contains person and grant ids: it is for in-memory keying
/// only and must never be serialized or exported. Exports keep carrying the
/// hashed `viewer_fingerprint`.
pub fn viewer_cache_key(state: &GroupState, viewer: &ViewerContext) -> String {
    fingerprint_input(state, viewer)
}

fn fingerprint_input(state: &GroupState, viewer: &ViewerContext) -> String {
    match viewer {
        ViewerContext::Anonymous => format!("anon:::{}", template_version(state)),
        ViewerContext::Person { person } => {
            let grants = active_grant_ids(state, person).join(",");
            let roles = active_role_names(state, person).join(",");
            format!(
                "person:{person}:{grants}:{roles}:{}",
                template_version(state)
            )
        }
    }
}

fn active_grant_ids(state: &GroupState, grantee: &PersonId) -> Vec<String> {
    let Some(group) = &state.group else {
        return Vec::new();
    };
    state
        .trust_grants
        .values()
        .filter(|grant| {
            grant.grantee == *grantee
                && grant.is_active()
                && scope_matches_group(grant.scope, group.id)
        })
        .map(|grant| grant.id.to_string())
        .collect()
}

fn active_role_names(state: &GroupState, person: &PersonId) -> Vec<String> {
    let mut roles: Vec<_> = active_roles_for(state, person)
        .map(|role| match role {
            GroupRole::Member => "member".to_string(),
            GroupRole::Governance => "governance".to_string(),
            GroupRole::Facilitator => "facilitator".to_string(),
        })
        .collect();
    roles.sort();
    roles
}

fn active_roles_for<'a>(
    state: &'a GroupState,
    person: &'a PersonId,
) -> impl Iterator<Item = GroupRole> + 'a {
    let group_id = state.group.as_ref().map(|group| group.id);
    state.memberships.values().filter_map(move |membership| {
        (Some(membership.group_id) == group_id
            && membership.person == *person
            && membership.lifecycle == cn_model::Lifecycle::Active)
            .then_some(membership.role)
    })
}

fn active_trusts_owner(state: &GroupState, owner: &PersonId, viewer: &PersonId) -> bool {
    let Some(group) = &state.group else {
        return false;
    };
    state.trust_grants.values().any(|grant| {
        grant.grantor == *owner
            && grant.grantee == *viewer
            && grant.is_active()
            && scope_matches_group(grant.scope, group.id)
    })
}

fn scope_matches_group(scope: TrustScope, group_id: cn_model::GroupId) -> bool {
    match scope {
        TrustScope::All => true,
        TrustScope::Group(id) => id == group_id,
    }
}

fn template_version(state: &GroupState) -> String {
    match &state.group {
        Some(group) => group.template_version.to_string(),
        None => "none".to_string(),
    }
}
