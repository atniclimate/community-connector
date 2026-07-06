use cn_model::{
    CustodyEvent, Entity, Membership, MembershipId, PersonId, SensitivityTier, TrustGrant,
    TrustGrantId,
};
use cn_store::{
    Authorizer, AuthzDenial, CustodyTarget, GroupState, OpKind, Operation, TierTarget,
    VisibilityTarget,
};

use crate::projection::{edge_visible, entity_visible, story_visible, visible_entity_attributes};
use crate::viewer::{ViewerContext, is_active_member, is_governance};

/// cn-store authorizer implementing ADR-002 A-B6.
pub struct PermAuthorizer;

impl Authorizer for PermAuthorizer {
    fn authorize(&self, state: &GroupState, op: &Operation) -> Result<(), AuthzDenial> {
        authorize_op(state, op)
    }
}

/// Stable denial codes returned by `PermAuthorizer`:
/// `group_exists`, `group_missing`, `not_member`, `not_owner_or_governance`,
/// `not_governance`, `not_owner`, `tier_not_tightened`, `bootstrap_denied`,
/// `grantor_mismatch`, `target_missing`, and `target_hidden`.
pub fn authorize_op(state: &GroupState, op: &Operation) -> Result<(), AuthzDenial> {
    let submitter = op.responsible_human;
    match &op.kind {
        OpKind::GroupCreate { .. } => authorize_group_create(state),
        OpKind::EntityCreate { entity } => authorize_entity_create(state, submitter, entity),
        OpKind::EdgeCreate { edge } => authorize_unowned_create(state, submitter, edge),
        OpKind::StoryCreate { story } => authorize_unowned_create(state, submitter, story),
        OpKind::AttributeSet { entity, .. } | OpKind::AttributeRemove { entity, .. } => {
            authorize_entity_mutation(state, submitter, *entity)
        }
        OpKind::EntityLifecycleSet { entity, .. } => {
            authorize_entity_lifecycle(state, submitter, *entity)
        }
        OpKind::EdgeLifecycleSet { .. } | OpKind::StoryLifecycleSet { .. } => {
            require_governance(state, submitter)
        }
        OpKind::VisibilitySet { target, .. } => authorize_visibility(state, submitter, target),
        OpKind::TierSet { target, tier } => authorize_tier(state, submitter, target, *tier),
        OpKind::EdgeWeightSet { .. } => require_member(state, submitter),
        OpKind::MembershipAdd { membership } => authorize_membership_add(state, op, membership),
        OpKind::MembershipLifecycleSet { membership, .. } => {
            authorize_membership_lifecycle(state, submitter, *membership)
        }
        OpKind::TrustGrantCreate { grant } => authorize_grant_create(submitter, grant),
        OpKind::TrustGrantRevoke { grant, .. } => authorize_grant_revoke(state, submitter, *grant),
        OpKind::CustodyAppend { target, event } => {
            authorize_custody(state, submitter, target, event)
        }
        OpKind::StoryUpdate { .. } => require_governance(state, submitter),
    }
}

fn authorize_group_create(state: &GroupState) -> Result<(), AuthzDenial> {
    if state.group.is_none() {
        Ok(())
    } else {
        Err(denial("group_exists", "group already exists"))
    }
}

fn authorize_entity_create(
    state: &GroupState,
    submitter: PersonId,
    entity: &Entity,
) -> Result<(), AuthzDenial> {
    match entity.owner {
        Some(owner) if owner == submitter || is_governance(state, submitter) => Ok(()),
        Some(_) => Err(denial(
            "not_owner_or_governance",
            "owned entity create requires owner or governance",
        )),
        None => require_member(state, submitter),
    }
}

fn authorize_unowned_create<T>(
    state: &GroupState,
    submitter: PersonId,
    _record: &T,
) -> Result<(), AuthzDenial> {
    require_member(state, submitter)
}

fn authorize_entity_mutation(
    state: &GroupState,
    submitter: PersonId,
    entity: cn_model::EntityId,
) -> Result<(), AuthzDenial> {
    let record = state
        .entities
        .get(&entity)
        .ok_or_else(|| denial("target_missing", "entity target missing"))?;
    authorize_owner_or_member(state, submitter, record.owner)
}

fn authorize_entity_lifecycle(
    state: &GroupState,
    submitter: PersonId,
    entity: cn_model::EntityId,
) -> Result<(), AuthzDenial> {
    let record = state
        .entities
        .get(&entity)
        .ok_or_else(|| denial("target_missing", "entity target missing"))?;
    match record.owner {
        Some(owner) if owner == submitter || is_governance(state, submitter) => Ok(()),
        Some(_) => Err(denial(
            "not_owner_or_governance",
            "lifecycle requires owner or governance",
        )),
        None => require_governance(state, submitter),
    }
}

fn authorize_owner_or_member(
    state: &GroupState,
    submitter: PersonId,
    owner: Option<PersonId>,
) -> Result<(), AuthzDenial> {
    match owner {
        Some(owner) if owner == submitter || is_governance(state, submitter) => Ok(()),
        Some(_) => Err(denial(
            "not_owner_or_governance",
            "owned record requires owner or governance",
        )),
        None => require_member(state, submitter),
    }
}

fn authorize_visibility(
    state: &GroupState,
    submitter: PersonId,
    target: &VisibilityTarget,
) -> Result<(), AuthzDenial> {
    let owner = visibility_owner(state, target)?;
    match owner {
        Some(owner) if owner == submitter => Ok(()),
        Some(_) => Err(denial("not_owner", "visibility requires owner")),
        None => require_governance(state, submitter),
    }
}

fn visibility_owner(
    state: &GroupState,
    target: &VisibilityTarget,
) -> Result<Option<PersonId>, AuthzDenial> {
    match target {
        VisibilityTarget::EntityPresence(id) | VisibilityTarget::Attribute(id, _) => state
            .entities
            .get(id)
            .map(|entity| entity.owner)
            .ok_or_else(|| denial("target_missing", "visibility target missing")),
        VisibilityTarget::Edge(id) => state
            .edges
            .contains_key(id)
            .then_some(None)
            .ok_or_else(|| denial("target_missing", "visibility target missing")),
        VisibilityTarget::Story(id) => state
            .stories
            .contains_key(id)
            .then_some(None)
            .ok_or_else(|| denial("target_missing", "visibility target missing")),
    }
}

fn authorize_tier(
    state: &GroupState,
    submitter: PersonId,
    target: &TierTarget,
    tier: SensitivityTier,
) -> Result<(), AuthzDenial> {
    if is_governance(state, submitter) {
        return Ok(());
    }
    let (owner, current) = tier_owner_and_current(state, target)?;
    if owner != Some(submitter) {
        return Err(denial("not_owner", "tier change requires owner"));
    }
    if tier > current {
        Ok(())
    } else {
        Err(denial(
            "tier_not_tightened",
            "owner tier changes must be strictly more restrictive",
        ))
    }
}

fn tier_owner_and_current(
    state: &GroupState,
    target: &TierTarget,
) -> Result<(Option<PersonId>, SensitivityTier), AuthzDenial> {
    match target {
        TierTarget::Entity(id) => state
            .entities
            .get(id)
            .map(|entity| (entity.owner, entity.tier))
            .ok_or_else(|| denial("target_missing", "tier target missing")),
        TierTarget::Attribute(id, attr) => attr_tier_owner_current(state, *id, attr),
        TierTarget::Edge(id) => state
            .edges
            .get(id)
            .map(|edge| (None, edge.tier))
            .ok_or_else(|| denial("target_missing", "tier target missing")),
        TierTarget::Story(id) => state
            .stories
            .get(id)
            .map(|story| (None, story.tier))
            .ok_or_else(|| denial("target_missing", "tier target missing")),
    }
}

fn attr_tier_owner_current(
    state: &GroupState,
    id: cn_model::EntityId,
    attr: &cn_model::AttrId,
) -> Result<(Option<PersonId>, SensitivityTier), AuthzDenial> {
    let entity = state
        .entities
        .get(&id)
        .ok_or_else(|| denial("target_missing", "tier target missing"))?;
    let instance = entity
        .attributes
        .get(attr)
        .ok_or_else(|| denial("target_missing", "tier target missing"))?;
    Ok((entity.owner, instance.effective_tier(entity.tier)))
}

fn authorize_membership_add(
    state: &GroupState,
    op: &Operation,
    membership: &Membership,
) -> Result<(), AuthzDenial> {
    if is_governance(state, op.responsible_human) {
        return Ok(());
    }
    if first_membership_bootstrap(state, op.responsible_human, membership) {
        return Ok(());
    }
    Err(denial(
        "bootstrap_denied",
        "membership add requires governance or first-member bootstrap",
    ))
}

fn first_membership_bootstrap(
    state: &GroupState,
    submitter: PersonId,
    membership: &Membership,
) -> bool {
    let Some(group) = &state.group else {
        return false;
    };
    state.memberships.is_empty()
        && membership.person == submitter
        && group.provenance.responsible_human() == submitter
        && membership.group_id == group.id
}

fn authorize_membership_lifecycle(
    state: &GroupState,
    submitter: PersonId,
    membership: MembershipId,
) -> Result<(), AuthzDenial> {
    state
        .memberships
        .contains_key(&membership)
        .then_some(())
        .ok_or_else(|| denial("target_missing", "membership target missing"))?;
    require_governance(state, submitter)
}

fn authorize_grant_create(submitter: PersonId, grant: &TrustGrant) -> Result<(), AuthzDenial> {
    if grant.grantor == submitter {
        Ok(())
    } else {
        Err(denial(
            "grantor_mismatch",
            "trust grant create requires grantor submitter",
        ))
    }
}

fn authorize_grant_revoke(
    state: &GroupState,
    submitter: PersonId,
    grant: TrustGrantId,
) -> Result<(), AuthzDenial> {
    let grant = state
        .trust_grants
        .get(&grant)
        .ok_or_else(|| denial("target_missing", "trust grant target missing"))?;
    if grant.grantor == submitter {
        Ok(())
    } else {
        Err(denial(
            "grantor_mismatch",
            "trust grant revoke requires grantor submitter",
        ))
    }
}

fn authorize_custody(
    state: &GroupState,
    submitter: PersonId,
    target: &CustodyTarget,
    _event: &CustodyEvent,
) -> Result<(), AuthzDenial> {
    if is_governance(state, submitter) {
        return Ok(());
    }
    require_member(state, submitter)?;
    if custody_target_visible(state, submitter, target)? {
        Ok(())
    } else {
        Err(denial("target_hidden", "custody target is not visible"))
    }
}

fn custody_target_visible(
    state: &GroupState,
    submitter: PersonId,
    target: &CustodyTarget,
) -> Result<bool, AuthzDenial> {
    let viewer = ViewerContext::Person { person: submitter };
    match target {
        CustodyTarget::Entity(id) => state
            .entities
            .get(id)
            .map(|entity| entity_visible(state, &viewer, entity))
            .ok_or_else(|| denial("target_missing", "custody target missing")),
        CustodyTarget::Attribute(id, attr) => custody_attribute_visible(state, &viewer, *id, attr),
        CustodyTarget::Edge(id) => state
            .edges
            .get(id)
            .map(|edge| edge_visible(state, &viewer, edge))
            .ok_or_else(|| denial("target_missing", "custody target missing")),
        CustodyTarget::Story(id) => state
            .stories
            .get(id)
            .map(|story| story_visible(state, &viewer, story))
            .ok_or_else(|| denial("target_missing", "custody target missing")),
        CustodyTarget::Group => Ok(state.group.is_some()),
    }
}

fn custody_attribute_visible(
    state: &GroupState,
    viewer: &ViewerContext,
    id: cn_model::EntityId,
    attr: &cn_model::AttrId,
) -> Result<bool, AuthzDenial> {
    let entity = state
        .entities
        .get(&id)
        .ok_or_else(|| denial("target_missing", "custody target missing"))?;
    Ok(visible_entity_attributes(state, viewer, entity).contains_key(attr))
}

fn require_member(state: &GroupState, submitter: PersonId) -> Result<(), AuthzDenial> {
    if is_active_member(state, submitter) {
        Ok(())
    } else {
        Err(denial("not_member", "operation requires active membership"))
    }
}

fn require_governance(state: &GroupState, submitter: PersonId) -> Result<(), AuthzDenial> {
    if is_governance(state, submitter) {
        Ok(())
    } else {
        Err(denial(
            "not_governance",
            "operation requires governance role",
        ))
    }
}

fn denial(code: &str, message: &str) -> AuthzDenial {
    AuthzDenial {
        code: code.to_string(),
        message: message.to_string(),
    }
}
