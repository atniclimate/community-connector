#![allow(clippy::too_many_lines)]

//! Exhaustive authz tests for the role x op-kind authority matrix
//! (docs/design/authority-matrix.md; facilitator blueprint P3.2).
//!
//! I5 note: this module is intentionally large because every matrix cell gets
//! an explicit assertion against one shared synthetic state, including the
//! three custody role-combinations (facilitator-only, member-only, dual-role).
//!
//! Cast (all synthetic):
//! - person(1) governance (also group creator)
//! - person(2) member
//! - person(3) facilitator only (owns entity 6)
//! - person(4) no membership (the "anonymous" write column)
//! - person(5) member who owns entity 2
//! - person(6) dual-role: facilitator and member

use std::str::FromStr;

use cn_model::*;
use cn_perm::authz::authorize_op;
use cn_store::*;

fn ts(ms: i64) -> Timestamp {
    Timestamp(ms)
}

fn uuid_string(n: u128) -> String {
    let hex = format!("{n:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

fn id<T>(n: u128) -> T
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    T::from_str(&uuid_string(n)).expect("valid deterministic id")
}

fn person(n: u128) -> PersonId {
    id(n)
}

fn group_id() -> GroupId {
    id(10)
}

fn entity_id(n: u128) -> EntityId {
    id(100 + n)
}

fn edge_id(n: u128) -> EdgeId {
    id(200 + n)
}

fn story_id(n: u128) -> StoryId {
    id(300 + n)
}

fn op_id(n: u128) -> OpId {
    id(400 + n)
}

fn membership_id(n: u128) -> MembershipId {
    id(500 + n)
}

fn grant_id(n: u128) -> TrustGrantId {
    id(600 + n)
}

fn attr_id(name: &str) -> AttrId {
    AttrId::new(name).expect("attr id")
}

fn kind_id() -> KindId {
    KindId::new("kind").expect("kind id")
}

fn envelope_for(who: PersonId) -> ProvenanceEnvelope {
    ProvenanceEnvelope::new(Origin::Authored, ActorRef::Human(who), who, ts(1))
        .expect("valid provenance")
}

fn group() -> Group {
    Group::new(
        group_id(),
        "Synthetic Group",
        TemplateId::new("synthetic").expect("template id"),
        cn_model::model_schema_version(),
        envelope_for(person(1)),
        SensitivityTier::T1,
    )
    .expect("valid group")
}

fn add_member(state: &mut GroupState, membership: u128, who: u128, role: GroupRole) {
    let record = Membership::new(
        membership_id(membership),
        group_id(),
        person(who),
        role,
        envelope_for(person(who)),
    );
    state.memberships.insert(record.id, record);
}

fn entity_by(n: u128, owner: Option<PersonId>, by: PersonId, visibility: Circle) -> Entity {
    let mut record = Entity::new(
        entity_id(n),
        group_id(),
        kind_id(),
        visibility,
        envelope_for(by),
        SensitivityTier::T1,
    );
    record.owner = owner;
    record
}

fn attr_by(value: &str, by: PersonId, visibility: Circle) -> AttributeInstance {
    AttributeInstance::new(
        AttributeValue::Text(value.to_string()),
        visibility,
        envelope_for(by),
    )
}

fn edge_by(n: u128, from: u128, to: u128, by: PersonId) -> Edge {
    Edge::new(
        edge_id(n),
        group_id(),
        KindId::new("link").expect("kind id"),
        entity_id(from),
        entity_id(to),
        false,
        Circle::Group,
        envelope_for(by),
        SensitivityTier::T1,
    )
}

fn story_by(n: u128, by: PersonId) -> Story {
    Story::new(
        story_id(n),
        group_id(),
        format!("Synthetic Story {n}"),
        vec![StoryStep {
            entity: entity_id(1),
            narration: "Synthetic step".to_string(),
        }],
        Circle::Group,
        envelope_for(by),
        SensitivityTier::T1,
    )
}

/// Private story - hidden from every non-governance viewer, including the
/// facilitator (target-aware StoryUpdate cells).
fn story_private(n: u128, by: PersonId) -> Story {
    Story::new(
        story_id(n),
        group_id(),
        format!("Synthetic Private Story {n}"),
        vec![StoryStep {
            entity: entity_id(1),
            narration: "Synthetic step".to_string(),
        }],
        Circle::Private,
        envelope_for(by),
        SensitivityTier::T1,
    )
}

fn op(sort: i64, submitter: PersonId, kind: OpKind) -> Operation {
    let actor = ActorRef::Human(submitter);
    let op_id = op_id(sort as u128);
    Operation {
        op_id,
        group_id: group_id(),
        actor: actor.clone(),
        responsible_human: submitter,
        recorded_at: ts(sort),
        sort_key: SortKey::new(
            Hlc {
                wall_ms: sort,
                counter: 0,
            },
            &actor,
            op_id,
        ),
        template_version: cn_model::model_schema_version(),
        kind,
        schema_version: cn_model::model_schema_version(),
    }
}

/// Shared matrix state; see the module doc for the cast.
fn matrix_state() -> GroupState {
    let mut state = GroupState::default();
    state.group = Some(group());
    add_member(&mut state, 1, 1, GroupRole::Governance);
    add_member(&mut state, 2, 2, GroupRole::Member);
    add_member(&mut state, 3, 3, GroupRole::Facilitator);
    add_member(&mut state, 5, 5, GroupRole::Member);
    add_member(&mut state, 6, 6, GroupRole::Facilitator);
    add_member(&mut state, 7, 6, GroupRole::Member);

    // e1: unowned group-visible record created by governance.
    state
        .entities
        .insert(entity_id(1), entity_by(1, None, person(1), Circle::Group));
    // e2: owned by member person(5).
    state.entities.insert(
        entity_id(2),
        entity_by(2, Some(person(5)), person(5), Circle::Group),
    );
    // e3: unowned but private - hidden from every non-governance submitter.
    state
        .entities
        .insert(entity_id(3), entity_by(3, None, person(1), Circle::Private));
    // e4: unowned, created by the facilitator person(3); carries one
    // facilitator-entered attribute and one member-entered attribute.
    let mut e4 = entity_by(4, None, person(3), Circle::Group);
    e4.attributes.insert(
        attr_id("fac_attr"),
        attr_by("Synthetic facilitator entry", person(3), Circle::Group),
    );
    e4.attributes.insert(
        attr_id("mem_attr"),
        attr_by("Synthetic member entry", person(2), Circle::Group),
    );
    state.entities.insert(entity_id(4), e4);
    // e5: created by the facilitator but private, so hidden even from them.
    state
        .entities
        .insert(entity_id(5), entity_by(5, None, person(3), Circle::Private));
    // e6: owned by the facilitator person(3) - owner tier/visibility cells.
    state.entities.insert(
        entity_id(6),
        entity_by(6, Some(person(3)), person(3), Circle::Group),
    );

    // g1: member-created edge; g2: facilitator-created edge.
    state.edges.insert(edge_id(1), edge_by(1, 1, 4, person(2)));
    state.edges.insert(edge_id(2), edge_by(2, 4, 1, person(3)));

    // s1: member-created story; s2: facilitator-created story.
    state.stories.insert(story_id(1), story_by(1, person(2)));
    state.stories.insert(story_id(2), story_by(2, person(3)));

    // Trust grant from member person(2) to facilitator person(3).
    let grant = TrustGrant::new(
        grant_id(1),
        person(2),
        person(3),
        TrustScope::All,
        ts(1),
        envelope_for(person(2)),
    );
    state.trust_grants.insert(grant.id, grant);
    state
}

fn assert_allowed(state: &GroupState, sort: i64, submitter: PersonId, kind: OpKind) {
    let result = authorize_op(state, &op(sort, submitter, kind));
    assert!(result.is_ok(), "expected allow, got {result:?}");
}

fn assert_denied(state: &GroupState, sort: i64, submitter: PersonId, kind: OpKind, code: &str) {
    let denial = authorize_op(state, &op(sort, submitter, kind)).expect_err("operation denied");
    assert_eq!(denial.code, code);
}

fn custody(who: PersonId) -> CustodyEvent {
    CustodyEvent {
        id: id(900),
        action: CustodyAction::Corrected,
        at: ts(5),
        actor: ActorRef::Human(who),
        note: Some("Synthetic custody".to_string()),
    }
}

fn custody_append(target: CustodyTarget, who: PersonId) -> OpKind {
    OpKind::CustodyAppend {
        target,
        event: custody(who),
    }
}

#[test]
fn group_create_cells() {
    // Bootstrap rule: anyone may create the first group; second create denied.
    let empty = GroupState::default();
    let create = || OpKind::GroupCreate {
        group: group(),
        template_json: "{}".to_string(),
    };
    assert_allowed(&empty, 1, person(4), create());
    assert_denied(&matrix_state(), 2, person(1), create(), "group_exists");
}

#[test]
fn entity_create_unowned_cells() {
    let state = matrix_state();
    let create = |n: u128, by: u128| OpKind::EntityCreate {
        entity: entity_by(n, None, person(by), Circle::Group),
    };
    assert_denied(&state, 1, person(4), create(30, 4), "not_member");
    assert_allowed(&state, 2, person(2), create(31, 2));
    assert_allowed(&state, 3, person(3), create(32, 3));
    assert_allowed(&state, 4, person(6), create(33, 6));
    assert_allowed(&state, 5, person(1), create(34, 1));
}

#[test]
fn entity_create_owned_by_other_cells() {
    let state = matrix_state();
    let create = |n: u128, by: u128| OpKind::EntityCreate {
        entity: entity_by(n, Some(person(5)), person(by), Circle::Group),
    };
    assert_denied(
        &state,
        1,
        person(4),
        create(30, 5),
        "not_owner_or_governance",
    );
    assert_denied(
        &state,
        2,
        person(2),
        create(31, 5),
        "not_owner_or_governance",
    );
    // Facilitator does NOT bypass owner controls.
    assert_denied(
        &state,
        3,
        person(3),
        create(32, 5),
        "not_owner_or_governance",
    );
    assert_denied(
        &state,
        4,
        person(6),
        create(33, 5),
        "not_owner_or_governance",
    );
    assert_allowed(&state, 5, person(1), create(34, 5));
    // Owner creating their own record is allowed.
    assert_allowed(&state, 6, person(5), create(35, 5));
}

#[test]
fn edge_create_cells() {
    let state = matrix_state();
    let create = |n: u128, by: u128| OpKind::EdgeCreate {
        edge: edge_by(n, 1, 4, person(by)),
    };
    assert_denied(&state, 1, person(4), create(30, 4), "not_member");
    assert_allowed(&state, 2, person(2), create(31, 2));
    assert_allowed(&state, 3, person(3), create(32, 3));
    assert_allowed(&state, 4, person(1), create(33, 1));
}

#[test]
fn story_create_cells() {
    // StoryCreate stays member-open (existing rule, documented as such).
    let state = matrix_state();
    let create = |n: u128, by: u128| OpKind::StoryCreate {
        story: story_by(n, person(by)),
    };
    assert_denied(&state, 1, person(4), create(30, 4), "not_member");
    assert_allowed(&state, 2, person(2), create(31, 2));
    assert_allowed(&state, 3, person(3), create(32, 3));
    assert_allowed(&state, 4, person(1), create(33, 1));
}

#[test]
fn attribute_mutation_unowned_target_cells() {
    let state = matrix_state();
    let set = |by: u128| OpKind::AttributeSet {
        entity: entity_id(1),
        attr: attr_id("cell"),
        instance: attr_by("Synthetic cell", person(by), Circle::Group),
    };
    assert_denied(&state, 1, person(4), set(4), "not_member");
    assert_allowed(&state, 2, person(2), set(2));
    assert_allowed(&state, 3, person(3), set(3));
    assert_allowed(&state, 4, person(1), set(1));
    let remove = OpKind::AttributeRemove {
        entity: entity_id(1),
        attr: attr_id("cell"),
    };
    assert_denied(&state, 5, person(4), remove.clone(), "not_member");
    assert_allowed(&state, 6, person(3), remove);
}

#[test]
fn attribute_mutation_owned_by_other_cells() {
    let state = matrix_state();
    let set = |by: u128| OpKind::AttributeSet {
        entity: entity_id(2),
        attr: attr_id("cell"),
        instance: attr_by("Synthetic cell", person(by), Circle::Group),
    };
    assert_denied(&state, 1, person(4), set(4), "not_owner_or_governance");
    assert_denied(&state, 2, person(2), set(2), "not_owner_or_governance");
    // Facilitator does NOT bypass owner controls.
    assert_denied(&state, 3, person(3), set(3), "not_owner_or_governance");
    assert_allowed(&state, 4, person(1), set(1));
    assert_allowed(&state, 5, person(5), set(5));
    assert_denied(
        &state,
        6,
        person(3),
        OpKind::AttributeSet {
            entity: entity_id(99),
            attr: attr_id("cell"),
            instance: attr_by("Synthetic cell", person(3), Circle::Group),
        },
        "target_missing",
    );
}

#[test]
fn entity_lifecycle_cells() {
    let state = matrix_state();
    let archive = |n: u128| OpKind::EntityLifecycleSet {
        entity: entity_id(n),
        lifecycle: Lifecycle::Archived,
    };
    // Unowned target: archival stays governance-only.
    assert_denied(&state, 1, person(4), archive(1), "not_governance");
    assert_denied(&state, 2, person(2), archive(1), "not_governance");
    assert_denied(&state, 3, person(3), archive(1), "not_governance");
    assert_denied(&state, 4, person(6), archive(1), "not_governance");
    assert_allowed(&state, 5, person(1), archive(1));
    // Owned target: owner or governance only.
    assert_allowed(&state, 6, person(5), archive(2));
    assert_denied(&state, 7, person(3), archive(2), "not_owner_or_governance");
}

#[test]
fn edge_and_story_lifecycle_cells() {
    let state = matrix_state();
    let edge_archive = OpKind::EdgeLifecycleSet {
        edge: edge_id(1),
        lifecycle: Lifecycle::Archived,
    };
    let story_archive = OpKind::StoryLifecycleSet {
        story: story_id(1),
        lifecycle: Lifecycle::Archived,
    };
    assert_denied(&state, 1, person(4), edge_archive.clone(), "not_governance");
    assert_denied(&state, 2, person(2), edge_archive.clone(), "not_governance");
    assert_denied(&state, 3, person(3), edge_archive.clone(), "not_governance");
    assert_allowed(&state, 4, person(1), edge_archive);
    assert_denied(
        &state,
        5,
        person(2),
        story_archive.clone(),
        "not_governance",
    );
    // Facilitator can author stories but cannot archive them.
    assert_denied(
        &state,
        6,
        person(3),
        story_archive.clone(),
        "not_governance",
    );
    assert_allowed(&state, 7, person(1), story_archive);
}

#[test]
fn visibility_cells() {
    let state = matrix_state();
    let owned = |visibility: Circle| OpKind::VisibilitySet {
        target: VisibilityTarget::EntityPresence(entity_id(2)),
        visibility,
    };
    // Owned target: strictly owner-only - even governance is denied.
    assert_denied(&state, 1, person(4), owned(Circle::Public), "not_owner");
    assert_denied(&state, 2, person(2), owned(Circle::Public), "not_owner");
    // Facilitator can NEVER loosen another's visibility.
    assert_denied(&state, 3, person(3), owned(Circle::Public), "not_owner");
    assert_denied(&state, 4, person(6), owned(Circle::Public), "not_owner");
    assert_denied(&state, 5, person(1), owned(Circle::Public), "not_owner");
    assert_allowed(&state, 6, person(5), owned(Circle::Trusted));
    // Facilitator as owner of their own record keeps owner authority.
    assert_allowed(
        &state,
        7,
        person(3),
        OpKind::VisibilitySet {
            target: VisibilityTarget::EntityPresence(entity_id(6)),
            visibility: Circle::Trusted,
        },
    );
    // Unowned targets (edge/story/unowned entity) stay governance-only.
    let edge_target = |visibility: Circle| OpKind::VisibilitySet {
        target: VisibilityTarget::Edge(edge_id(1)),
        visibility,
    };
    assert_denied(
        &state,
        8,
        person(2),
        edge_target(Circle::Trusted),
        "not_governance",
    );
    assert_denied(
        &state,
        9,
        person(3),
        edge_target(Circle::Trusted),
        "not_governance",
    );
    assert_allowed(&state, 10, person(1), edge_target(Circle::Trusted));
    assert_denied(
        &state,
        11,
        person(3),
        OpKind::VisibilitySet {
            target: VisibilityTarget::EntityPresence(entity_id(1)),
            visibility: Circle::Trusted,
        },
        "not_governance",
    );
}

#[test]
fn tier_cells() {
    let state = matrix_state();
    let entity_tier = |n: u128, tier: SensitivityTier| OpKind::TierSet {
        target: TierTarget::Entity(entity_id(n)),
        tier,
    };
    // Governance: any direction (tier authority is ATNI Climate via
    // governance, D-034).
    assert_allowed(&state, 1, person(1), entity_tier(2, SensitivityTier::T0));
    // Owner: tighten-only.
    assert_allowed(&state, 2, person(5), entity_tier(2, SensitivityTier::T2));
    assert_denied(
        &state,
        3,
        person(5),
        entity_tier(2, SensitivityTier::T0),
        "tier_not_tightened",
    );
    assert_denied(
        &state,
        4,
        person(5),
        entity_tier(2, SensitivityTier::T1),
        "tier_not_tightened",
    );
    // Non-owner member and facilitator: denied; facilitator cannot lower a
    // tier on anyone else's record.
    assert_denied(
        &state,
        5,
        person(2),
        entity_tier(2, SensitivityTier::T2),
        "not_owner",
    );
    assert_denied(
        &state,
        6,
        person(3),
        entity_tier(2, SensitivityTier::T2),
        "not_owner",
    );
    assert_denied(
        &state,
        7,
        person(3),
        entity_tier(2, SensitivityTier::T0),
        "not_owner",
    );
    assert_denied(
        &state,
        8,
        person(4),
        entity_tier(2, SensitivityTier::T2),
        "not_owner",
    );
    // Facilitator as owner: tighten-only, same as any owner.
    assert_allowed(&state, 9, person(3), entity_tier(6, SensitivityTier::T2));
    assert_denied(
        &state,
        10,
        person(3),
        entity_tier(6, SensitivityTier::T0),
        "tier_not_tightened",
    );
}

#[test]
fn edge_weight_cells() {
    let state = matrix_state();
    let weight = OpKind::EdgeWeightSet {
        edge: edge_id(1),
        weight: Some(2.0),
    };
    assert_denied(&state, 1, person(4), weight.clone(), "not_member");
    assert_allowed(&state, 2, person(2), weight.clone());
    assert_allowed(&state, 3, person(3), weight.clone());
    assert_allowed(&state, 4, person(1), weight);
}

#[test]
fn membership_add_cells() {
    let state = matrix_state();
    let add = |membership: u128, who: u128, role: GroupRole, by: PersonId| OpKind::MembershipAdd {
        membership: Membership::new(
            membership_id(membership),
            group_id(),
            person(who),
            role,
            envelope_for(by),
        ),
    };
    assert_allowed(
        &state,
        1,
        person(1),
        add(30, 4, GroupRole::Member, person(1)),
    );
    assert_denied(
        &state,
        2,
        person(2),
        add(31, 4, GroupRole::Member, person(2)),
        "bootstrap_denied",
    );
    // Facilitator CANNOT govern membership or grant roles - not even to
    // themselves.
    assert_denied(
        &state,
        3,
        person(3),
        add(32, 4, GroupRole::Member, person(3)),
        "bootstrap_denied",
    );
    assert_denied(
        &state,
        4,
        person(3),
        add(33, 3, GroupRole::Member, person(3)),
        "bootstrap_denied",
    );
    assert_denied(
        &state,
        5,
        person(3),
        add(34, 4, GroupRole::Facilitator, person(3)),
        "bootstrap_denied",
    );
    // First-member bootstrap by the group creator stays intact.
    let mut fresh = GroupState::default();
    fresh.group = Some(group());
    assert_allowed(
        &fresh,
        6,
        person(1),
        add(35, 1, GroupRole::Governance, person(1)),
    );
}

#[test]
fn membership_lifecycle_cells() {
    let state = matrix_state();
    let archive = |membership: u128| OpKind::MembershipLifecycleSet {
        membership: membership_id(membership),
        lifecycle: Lifecycle::Archived,
    };
    assert_denied(&state, 1, person(4), archive(2), "not_governance");
    assert_denied(&state, 2, person(2), archive(2), "not_governance");
    assert_denied(&state, 3, person(3), archive(2), "not_governance");
    assert_allowed(&state, 4, person(1), archive(2));
    assert_denied(&state, 5, person(1), archive(99), "target_missing");
}

#[test]
fn trust_grant_cells() {
    let state = matrix_state();
    // Grantor-only rule is role-independent and unchanged: a facilitator may
    // grant trust as themselves and may not act for another grantor.
    let create = |grantor: u128, grantee: u128| OpKind::TrustGrantCreate {
        grant: TrustGrant::new(
            grant_id(30),
            person(grantor),
            person(grantee),
            TrustScope::All,
            ts(2),
            envelope_for(person(grantor)),
        ),
    };
    assert_allowed(&state, 1, person(3), create(3, 2));
    assert_allowed(&state, 2, person(4), create(4, 2));
    assert_denied(&state, 3, person(3), create(2, 3), "grantor_mismatch");
    let revoke = OpKind::TrustGrantRevoke {
        grant: grant_id(1),
        at: ts(3),
    };
    assert_allowed(&state, 4, person(2), revoke.clone());
    assert_denied(&state, 5, person(3), revoke, "grantor_mismatch");
}

#[test]
fn story_update_cells() {
    // The one loosening: require_governance -> is_facilitator_or_governance
    // (D-037 in-app authoring authority).
    let state = matrix_state();
    let update = || OpKind::StoryUpdate {
        story: story_id(1),
        title: Some("Synthetic updated title".to_string()),
        steps: None,
    };
    assert_denied(
        &state,
        1,
        person(4),
        update(),
        "not_facilitator_or_governance",
    );
    assert_denied(
        &state,
        2,
        person(2),
        update(),
        "not_facilitator_or_governance",
    );
    assert_allowed(&state, 3, person(3), update());
    assert_allowed(&state, 4, person(6), update());
    assert_allowed(&state, 5, person(1), update());

    // Adversarial-round tightening (2026-07-17): StoryUpdate is target-aware
    // for facilitators. Facilitator write reach never exceeds facilitator
    // read reach; governance stays unrestricted; the role check precedes the
    // target check so non-facilitators learn nothing about a story.
    let mut with_hidden = matrix_state();
    with_hidden
        .stories
        .insert(story_id(3), story_private(3, person(1)));
    let update_hidden = || OpKind::StoryUpdate {
        story: story_id(3),
        title: Some("Synthetic hidden update".to_string()),
        steps: None,
    };
    assert_denied(&with_hidden, 6, person(3), update_hidden(), "target_hidden");
    assert_allowed(&with_hidden, 7, person(1), update_hidden());
    let update_missing = || OpKind::StoryUpdate {
        story: story_id(9),
        title: Some("Synthetic missing update".to_string()),
        steps: None,
    };
    assert_denied(&state, 8, person(3), update_missing(), "target_missing");
    assert_allowed(&state, 9, person(1), update_missing());
    assert_denied(
        &state,
        10,
        person(2),
        update_missing(),
        "not_facilitator_or_governance",
    );
}

#[test]
fn custody_governance_cells() {
    let state = matrix_state();
    // Governance: any target, including hidden ones.
    assert_allowed(
        &state,
        1,
        person(1),
        custody_append(CustodyTarget::Entity(entity_id(1)), person(1)),
    );
    assert_allowed(
        &state,
        2,
        person(1),
        custody_append(CustodyTarget::Entity(entity_id(3)), person(1)),
    );
}

#[test]
fn custody_member_only_cells() {
    let state = matrix_state();
    // Member: any visible target, including facilitator-created records.
    assert_allowed(
        &state,
        1,
        person(2),
        custody_append(CustodyTarget::Entity(entity_id(1)), person(2)),
    );
    assert_allowed(
        &state,
        2,
        person(2),
        custody_append(CustodyTarget::Entity(entity_id(4)), person(2)),
    );
    assert_allowed(
        &state,
        3,
        person(2),
        custody_append(CustodyTarget::Group, person(2)),
    );
    assert_denied(
        &state,
        4,
        person(2),
        custody_append(CustodyTarget::Entity(entity_id(3)), person(2)),
        "target_hidden",
    );
    assert_denied(
        &state,
        5,
        person(2),
        custody_append(CustodyTarget::Entity(entity_id(99)), person(2)),
        "target_missing",
    );
    assert_denied(
        &state,
        6,
        person(4),
        custody_append(CustodyTarget::Entity(entity_id(1)), person(4)),
        "not_member",
    );
}

#[test]
fn custody_facilitator_only_cells() {
    let state = matrix_state();
    // Facilitator-only: custody follows their own entry work.
    assert_allowed(
        &state,
        1,
        person(3),
        custody_append(CustodyTarget::Entity(entity_id(4)), person(3)),
    );
    assert_denied(
        &state,
        2,
        person(3),
        custody_append(CustodyTarget::Entity(entity_id(1)), person(3)),
        "custody_not_own_record",
    );
    // Visibility still gates their own created records.
    assert_denied(
        &state,
        3,
        person(3),
        custody_append(CustodyTarget::Entity(entity_id(5)), person(3)),
        "target_hidden",
    );
    assert_denied(
        &state,
        4,
        person(3),
        custody_append(CustodyTarget::Entity(entity_id(99)), person(3)),
        "target_missing",
    );
    // Attribute targets resolve to the attribute instance's own envelope: the
    // facilitator reaches their own entry, not a member's entry on the same
    // entity.
    assert_allowed(
        &state,
        5,
        person(3),
        custody_append(
            CustodyTarget::Attribute(entity_id(4), attr_id("fac_attr")),
            person(3),
        ),
    );
    assert_denied(
        &state,
        6,
        person(3),
        custody_append(
            CustodyTarget::Attribute(entity_id(4), attr_id("mem_attr")),
            person(3),
        ),
        "custody_not_own_record",
    );
    // Edge and story targets follow the same created-records rule.
    assert_allowed(
        &state,
        7,
        person(3),
        custody_append(CustodyTarget::Edge(edge_id(2)), person(3)),
    );
    assert_denied(
        &state,
        8,
        person(3),
        custody_append(CustodyTarget::Edge(edge_id(1)), person(3)),
        "custody_not_own_record",
    );
    assert_allowed(
        &state,
        9,
        person(3),
        custody_append(CustodyTarget::Story(story_id(2)), person(3)),
    );
    assert_denied(
        &state,
        10,
        person(3),
        custody_append(CustodyTarget::Story(story_id(1)), person(3)),
        "custody_not_own_record",
    );
    // The group record was created by governance, not the facilitator.
    assert_denied(
        &state,
        11,
        person(3),
        custody_append(CustodyTarget::Group, person(3)),
        "custody_not_own_record",
    );
}

#[test]
fn custody_dual_role_cells() {
    let state = matrix_state();
    // Facilitator who is ALSO a member keeps member reach - the role never
    // subtracts.
    assert_allowed(
        &state,
        1,
        person(6),
        custody_append(CustodyTarget::Entity(entity_id(1)), person(6)),
    );
    assert_allowed(
        &state,
        2,
        person(6),
        custody_append(CustodyTarget::Group, person(6)),
    );
    assert_denied(
        &state,
        3,
        person(6),
        custody_append(CustodyTarget::Entity(entity_id(3)), person(6)),
        "target_hidden",
    );
}
