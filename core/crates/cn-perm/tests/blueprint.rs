#![allow(clippy::too_many_lines)]

//! I5 note: this test module is intentionally large because the cn-perm
//! blueprint obligations share one synthetic state builder and should be
//! reviewed as one acceptance suite.

use std::collections::BTreeSet;
use std::str::FromStr;

use cn_model::*;
use cn_perm::*;
use cn_store::*;
use proptest::prelude::*;

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

fn attr_id(n: u128) -> AttrId {
    AttrId::new(format!("attr{n}")).expect("attr id")
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

fn base_state() -> GroupState {
    let mut state = GroupState::default();
    state.group = Some(group());
    state
}

fn member(state: &mut GroupState, n: u128, role: GroupRole, lifecycle: Lifecycle) {
    let mut membership = Membership::new(
        membership_id(n),
        group_id(),
        person(n),
        role,
        envelope_for(person(n)),
    );
    membership.lifecycle = lifecycle;
    state.memberships.insert(membership.id, membership);
}

fn entity(n: u128, owner: Option<PersonId>, visibility: Circle, tier: SensitivityTier) -> Entity {
    let mut entity = Entity::new(
        entity_id(n),
        group_id(),
        kind_id(),
        visibility,
        envelope_for(owner.unwrap_or_else(|| person(1))),
        tier,
    );
    entity.owner = owner;
    entity
}

fn attr(value: &str, visibility: Circle) -> AttributeInstance {
    AttributeInstance::new(
        AttributeValue::Text(value.to_string()),
        visibility,
        envelope_for(person(1)),
    )
}

fn attr_with_tier(
    value: &str,
    visibility: Circle,
    container: SensitivityTier,
    tier: SensitivityTier,
) -> AttributeInstance {
    let mut instance = attr(value, visibility);
    if tier > container {
        instance
            .tighten_tier(container, tier)
            .expect("tier tightens");
    }
    instance
}

fn edge(n: u128, from: u128, to: u128, visibility: Circle, tier: SensitivityTier) -> Edge {
    Edge::new(
        edge_id(n),
        group_id(),
        KindId::new("link").expect("kind id"),
        entity_id(from),
        entity_id(to),
        false,
        visibility,
        envelope_for(person(1)),
        tier,
    )
}

fn story(n: u128, steps: Vec<u128>, visibility: Circle, tier: SensitivityTier) -> Story {
    Story::new(
        story_id(n),
        group_id(),
        format!("Synthetic Story {n}"),
        steps
            .into_iter()
            .map(|entity| StoryStep {
                entity: entity_id(entity),
                narration: format!("Synthetic step {entity}"),
            })
            .collect(),
        visibility,
        envelope_for(person(1)),
        tier,
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

fn group_create(sort: i64) -> Operation {
    op(
        sort,
        person(1),
        OpKind::GroupCreate {
            group: group(),
            template_json: template_json(),
        },
    )
}

fn template_json() -> String {
    r##"{
        "schema_version": "0.1.0",
        "template_id": "synthetic",
        "name": "Synthetic",
        "description": "Synthetic template",
        "kinds": [{
            "id": "kind",
            "label": "Kind",
            "shape": "sphere",
            "color_role": "kind-1",
            "attributes": [
                { "id": "attr0", "type": "text" },
                { "id": "attr1", "type": "text" },
                { "id": "attr2", "type": "text" },
                { "id": "location", "type": "text" },
                { "id": "site_knowledge", "type": "text" }
            ]
        }],
        "edge_kinds": [{
            "id": "link",
            "label": "Link",
            "from": ["kind"],
            "to": ["kind"],
            "directed": false,
            "weighted": "optional"
        }]
    }"##
    .to_string()
}

#[derive(Clone, Debug)]
struct AttrSpec {
    visibility: Circle,
    tier: Option<SensitivityTier>,
    lifecycle_marker: u8,
}

#[derive(Clone, Debug)]
struct EntitySpec {
    owner: Option<u8>,
    visibility: Circle,
    tier: SensitivityTier,
    lifecycle: Lifecycle,
    attrs: Vec<AttrSpec>,
}

#[derive(Clone, Debug)]
struct EdgeSpec {
    from: usize,
    to: usize,
    visibility: Circle,
    tier: SensitivityTier,
    lifecycle: Lifecycle,
}

#[derive(Clone, Debug)]
struct StorySpec {
    steps: Vec<usize>,
    visibility: Circle,
    tier: SensitivityTier,
    lifecycle: Lifecycle,
}

#[derive(Clone, Debug)]
struct GrantSpec {
    grantor: u8,
    grantee: u8,
    revoked: bool,
}

#[derive(Clone, Debug)]
struct MembershipSpec {
    person: u8,
    role: GroupRole,
    lifecycle: Lifecycle,
}

#[derive(Clone, Debug)]
struct Scenario {
    memberships: Vec<MembershipSpec>,
    grants: Vec<GrantSpec>,
    entities: Vec<EntitySpec>,
    edges: Vec<EdgeSpec>,
    stories: Vec<StorySpec>,
    viewer: Option<u8>,
}

fn circle_strategy() -> impl Strategy<Value = Circle> {
    prop_oneof![
        Just(Circle::Private),
        Just(Circle::Trusted),
        Just(Circle::Group),
        Just(Circle::Network),
        Just(Circle::Public),
    ]
}

fn tier_strategy() -> impl Strategy<Value = SensitivityTier> {
    prop_oneof![
        Just(SensitivityTier::T0),
        Just(SensitivityTier::T1),
        Just(SensitivityTier::T2),
        Just(SensitivityTier::T3),
    ]
}

fn lifecycle_strategy() -> impl Strategy<Value = Lifecycle> {
    prop_oneof![Just(Lifecycle::Active), Just(Lifecycle::Archived)]
}

/// Five viewer classes reachable by the no-leak property (P3.4): anonymous
/// (no viewer), member, facilitator, self (viewer owns generated entities),
/// and governance. Dual-role holders arise because the membership vector can
/// assign several roles to one person.
fn role_strategy() -> impl Strategy<Value = GroupRole> {
    prop_oneof![
        Just(GroupRole::Member),
        Just(GroupRole::Governance),
        Just(GroupRole::Facilitator),
    ]
}

fn scenario_strategy() -> impl Strategy<Value = Scenario> {
    let attr_spec = (
        circle_strategy(),
        prop::option::of(tier_strategy()),
        any::<u8>(),
    )
        .prop_map(|(visibility, tier, lifecycle_marker)| AttrSpec {
            visibility,
            tier,
            lifecycle_marker,
        });
    let entity_spec = (
        prop::option::of(1_u8..=5),
        circle_strategy(),
        tier_strategy(),
        lifecycle_strategy(),
        prop::collection::vec(attr_spec, 0..5),
    )
        .prop_map(|(owner, visibility, tier, lifecycle, attrs)| EntitySpec {
            owner,
            visibility,
            tier,
            lifecycle,
            attrs,
        });
    let edge_spec = (
        0_usize..20,
        0_usize..20,
        circle_strategy(),
        tier_strategy(),
        lifecycle_strategy(),
    )
        .prop_map(|(from, to, visibility, tier, lifecycle)| EdgeSpec {
            from,
            to,
            visibility,
            tier,
            lifecycle,
        });
    let story_spec = (
        prop::collection::vec(0_usize..20, 0..5),
        circle_strategy(),
        tier_strategy(),
        lifecycle_strategy(),
    )
        .prop_map(|(steps, visibility, tier, lifecycle)| StorySpec {
            steps,
            visibility,
            tier,
            lifecycle,
        });
    (
        prop::collection::vec((1_u8..=5, role_strategy(), lifecycle_strategy()), 0..10),
        prop::collection::vec((1_u8..=5, 1_u8..=5, any::<bool>()), 0..10),
        prop::collection::vec(entity_spec, 0..20),
        prop::collection::vec(edge_spec, 0..20),
        prop::collection::vec(story_spec, 0..10),
        prop::option::of(1_u8..=5),
    )
        .prop_map(
            |(memberships, grants, entities, edges, stories, viewer)| Scenario {
                memberships: memberships
                    .into_iter()
                    .map(|(person, role, lifecycle)| MembershipSpec {
                        person,
                        role,
                        lifecycle,
                    })
                    .collect(),
                grants: grants
                    .into_iter()
                    .map(|(grantor, grantee, revoked)| GrantSpec {
                        grantor,
                        grantee,
                        revoked,
                    })
                    .collect(),
                entities,
                edges,
                stories,
                viewer,
            },
        )
}

fn scenario_state(scenario: &Scenario) -> GroupState {
    let mut state = base_state();
    for (n, membership) in scenario.memberships.iter().enumerate() {
        let mut record = Membership::new(
            membership_id(n as u128 + 1),
            group_id(),
            person(membership.person.into()),
            membership.role,
            envelope_for(person(membership.person.into())),
        );
        record.lifecycle = membership.lifecycle;
        state.memberships.insert(record.id, record);
    }
    for (n, grant) in scenario.grants.iter().enumerate() {
        let mut record = TrustGrant::new(
            grant_id(n as u128 + 1),
            person(grant.grantor.into()),
            person(grant.grantee.into()),
            TrustScope::All,
            ts(1),
            envelope_for(person(grant.grantor.into())),
        );
        if grant.revoked {
            record.revoke(ts(2)).expect("valid revoke");
        }
        state.trust_grants.insert(record.id, record);
    }
    add_scenario_entities(&mut state, scenario);
    add_scenario_edges(&mut state, scenario);
    add_scenario_stories(&mut state, scenario);
    state
}

fn add_scenario_entities(state: &mut GroupState, scenario: &Scenario) {
    for (n, spec) in scenario.entities.iter().enumerate() {
        let owner = spec.owner.map(|owner| person(owner.into()));
        let mut record = entity(n as u128 + 1, owner, spec.visibility, spec.tier);
        record.lifecycle = spec.lifecycle;
        for (a, attr_spec) in spec.attrs.iter().enumerate() {
            let tier = attr_spec.tier.unwrap_or(spec.tier);
            let instance = attr_with_tier(
                &format!("Synthetic value {n}-{a}-{}", attr_spec.lifecycle_marker),
                attr_spec.visibility,
                spec.tier,
                tier,
            );
            record.attributes.insert(attr_id(a as u128), instance);
        }
        state.entities.insert(record.id, record);
    }
}

fn add_scenario_edges(state: &mut GroupState, scenario: &Scenario) {
    let len = scenario.entities.len();
    if len == 0 {
        return;
    }
    for (n, spec) in scenario.edges.iter().enumerate() {
        let mut record = edge(
            n as u128 + 1,
            (spec.from % len) as u128 + 1,
            (spec.to % len) as u128 + 1,
            spec.visibility,
            spec.tier,
        );
        record.lifecycle = spec.lifecycle;
        state.edges.insert(record.id, record);
    }
}

fn add_scenario_stories(state: &mut GroupState, scenario: &Scenario) {
    let len = scenario.entities.len();
    if len == 0 {
        return;
    }
    for (n, spec) in scenario.stories.iter().enumerate() {
        let steps = spec
            .steps
            .iter()
            .map(|idx| (idx % len) as u128 + 1)
            .collect();
        let mut record = story(n as u128 + 1, steps, spec.visibility, spec.tier);
        record.lifecycle = spec.lifecycle;
        state.stories.insert(record.id, record);
    }
}

fn scenario_viewer(scenario: &Scenario) -> ViewerContext {
    scenario
        .viewer
        .map(|viewer| ViewerContext::Person {
            person: person(viewer.into()),
        })
        .unwrap_or(ViewerContext::Anonymous)
}

proptest! {
    #[test]
    fn property_projection_sound_and_complete(scenario in scenario_strategy()) {
        let state = scenario_state(&scenario);
        let viewer = scenario_viewer(&scenario);
        let projection = project(&state, &viewer, 7);
        let projected_entities: BTreeSet<_> = projection.entities.iter().map(|entity| entity.id).collect();

        for projected in &projection.entities {
            let raw = state.entities.get(&projected.id).expect("projected entity exists");
            prop_assert!(raw.lifecycle == Lifecycle::Active);
            prop_assert!(cn_perm::projection::entity_visible(&state, &viewer, raw));
            for attr in projected.attributes.keys() {
                let instance = raw.attributes.get(attr).expect("projected attr exists");
                prop_assert!(cn_perm::projection::attr_visible(
                    &state,
                    &viewer,
                    raw.owner.as_ref(),
                    raw.tier,
                    instance
                ));
            }
            for (attr, instance) in &raw.attributes {
                let expected = cn_perm::projection::attr_visible(
                    &state,
                    &viewer,
                    raw.owner.as_ref(),
                    raw.tier,
                    instance,
                );
                prop_assert_eq!(projected.attributes.contains_key(attr), expected);
            }
        }

        for raw in state.entities.values().filter(|entity| entity.lifecycle == Lifecycle::Active) {
            let expected = cn_perm::projection::entity_visible(&state, &viewer, raw);
            prop_assert_eq!(projected_entities.contains(&raw.id), expected);
        }
        for edge in &projection.edges {
            prop_assert!(projected_entities.contains(&edge.from));
            prop_assert!(projected_entities.contains(&edge.to));
        }
        for story in &projection.stories {
            for step in &story.steps {
                prop_assert!(projected_entities.contains(&step.entity));
            }
        }

        // Read property extension (P3.4 + 2026-07-17 adversarial round):
        // governance-only report depth never appears for any non-governance
        // viewer class (anonymous, member, facilitator, self, dual-role).
        // A non-governance viewer gets zeroed counts, no invisible-subject
        // warnings, and at most their own finding-stripped quarantine stubs.
        let sample_report = StoreReport {
            quarantined: vec![QuarantineEntry {
                op_id: op_id(999),
                responsible_human: person(1),
                subject: None,
                reason: QuarantineReason::MissingTarget,
                findings: vec![cn_schema::Finding {
                    code: cn_schema::FindingCode::UnsupportedSchemaVersion,
                    message: "synthetic governance-only depth".to_string(),
                    path: "/synthetic".to_string(),
                }],
            }],
            warnings: vec![StoreFinding {
                code: "Synthetic".to_string(),
                message: "synthetic invisible-subject warning".to_string(),
                subject: None,
            }],
            applied: 3,
            deduped: 2,
        };
        let redacted = redact_report(&state, &viewer, &sample_report);
        let viewer_person = match &viewer {
            ViewerContext::Person { person } => Some(*person),
            ViewerContext::Anonymous => None,
        };
        let viewer_is_governance =
            viewer_person.is_some_and(|person| cn_perm::viewer::is_governance(&state, person));
        if viewer_is_governance {
            prop_assert_eq!(&redacted, &sample_report);
        } else {
            prop_assert_eq!(redacted.applied, 0);
            prop_assert_eq!(redacted.deduped, 0);
            prop_assert!(redacted.warnings.is_empty());
            for entry in &redacted.quarantined {
                prop_assert!(entry.findings.is_empty());
                prop_assert_eq!(Some(entry.responsible_human), viewer_person);
            }
        }
    }
}

#[test]
fn tier_ceiling_cells() {
    let owner = person(1);
    let member_person = person(2);
    let non_member = person(3);
    let mut state = base_state();
    member(&mut state, 2, GroupRole::Member, Lifecycle::Active);
    let mut record = entity(1, Some(owner), Circle::Public, SensitivityTier::T0);
    record.attributes.insert(
        AttrId::new("t3_public").expect("attr id"),
        attr_with_tier(
            "Synthetic T3",
            Circle::Public,
            SensitivityTier::T0,
            SensitivityTier::T3,
        ),
    );
    record.attributes.insert(
        AttrId::new("t2_public").expect("attr id"),
        attr_with_tier(
            "Synthetic T2",
            Circle::Public,
            SensitivityTier::T0,
            SensitivityTier::T2,
        ),
    );
    record.attributes.insert(
        AttrId::new("t1_public").expect("attr id"),
        attr_with_tier(
            "Synthetic T1",
            Circle::Public,
            SensitivityTier::T0,
            SensitivityTier::T1,
        ),
    );
    record
        .attributes
        .insert(attr_id(0), attr("Synthetic T0", Circle::Public));
    state.entities.insert(record.id, record);

    let owner_projection = project(&state, &ViewerContext::Person { person: owner }, 1);
    let member_projection = project(
        &state,
        &ViewerContext::Person {
            person: member_person,
        },
        1,
    );
    let network_projection = project(&state, &ViewerContext::Person { person: non_member }, 1);
    let anon_projection = project(&state, &ViewerContext::Anonymous, 1);
    assert!(
        owner_projection.entities[0]
            .attributes
            .contains_key(&AttrId::new("t3_public").expect("attr id"))
    );
    assert!(
        !member_projection.entities[0]
            .attributes
            .contains_key(&AttrId::new("t3_public").expect("attr id"))
    );
    assert!(
        member_projection.entities[0]
            .attributes
            .contains_key(&AttrId::new("t2_public").expect("attr id"))
    );
    assert!(
        !network_projection.entities[0]
            .attributes
            .contains_key(&AttrId::new("t2_public").expect("attr id"))
    );
    assert!(
        network_projection.entities[0]
            .attributes
            .contains_key(&AttrId::new("t1_public").expect("attr id"))
    );
    assert!(
        !anon_projection.entities[0]
            .attributes
            .contains_key(&AttrId::new("t1_public").expect("attr id"))
    );
    assert!(
        anon_projection.entities[0]
            .attributes
            .contains_key(&attr_id(0))
    );
}

#[test]
fn export_projection_excludes_owner_visible_t3_override() {
    let owner = person(1);
    let mut state = base_state();
    let mut record = entity(1, Some(owner), Circle::Public, SensitivityTier::T0);
    let t3_attr = AttrId::new("t3_public").expect("attr id");
    record.attributes.insert(
        t3_attr.clone(),
        attr_with_tier(
            "Synthetic T3 export gated",
            Circle::Public,
            SensitivityTier::T0,
            SensitivityTier::T3,
        ),
    );
    state.entities.insert(record.id, record);

    let viewer = ViewerContext::Person { person: owner };
    let display_projection = project(&state, &viewer, 1);
    let export = export_projection(&state, &viewer, 1);

    assert!(
        display_projection.entities[0]
            .attributes
            .contains_key(&t3_attr)
    );
    assert!(!export.entities[0].attributes.contains_key(&t3_attr));
}

#[test]
fn fisheries_scenario_matches_spec() {
    let owner = person(1);
    let trusted = person(2);
    let member_person = person(3);
    let mut state = base_state();
    member(&mut state, 3, GroupRole::Member, Lifecycle::Active);
    let grant = TrustGrant::new(
        grant_id(1),
        owner,
        trusted,
        TrustScope::All,
        ts(1),
        envelope_for(owner),
    );
    state.trust_grants.insert(grant.id, grant);
    let mut site = entity(1, Some(owner), Circle::Trusted, SensitivityTier::T1);
    site.attributes.insert(
        AttrId::new("location").expect("attr id"),
        attr("Synthetic location", Circle::Trusted),
    );
    site.attributes.insert(
        AttrId::new("site_knowledge").expect("attr id"),
        attr_with_tier(
            "Synthetic knowledge",
            Circle::Public,
            SensitivityTier::T1,
            SensitivityTier::T3,
        ),
    );
    state.entities.insert(site.id, site);

    assert_eq!(
        project(&state, &ViewerContext::Person { person: owner }, 1).entities[0]
            .attributes
            .len(),
        2
    );
    let trusted_projection = project(&state, &ViewerContext::Person { person: trusted }, 1);
    assert!(
        trusted_projection.entities[0]
            .attributes
            .contains_key(&AttrId::new("location").expect("attr id"))
    );
    assert!(
        !trusted_projection.entities[0]
            .attributes
            .contains_key(&AttrId::new("site_knowledge").expect("attr id"))
    );
    assert!(
        project(
            &state,
            &ViewerContext::Person {
                person: member_person
            },
            1
        )
        .entities
        .is_empty()
    );
    assert!(
        project(&state, &ViewerContext::Anonymous, 1)
            .entities
            .is_empty()
    );
}

#[test]
fn revoked_grant_changes_projection_after_fold() {
    let owner = person(1);
    let trusted = person(2);
    let mut site = entity(1, Some(owner), Circle::Trusted, SensitivityTier::T1);
    site.attributes
        .insert(attr_id(0), attr("Synthetic trusted", Circle::Trusted));
    let grant = TrustGrant::new(
        grant_id(1),
        owner,
        trusted,
        TrustScope::All,
        ts(2),
        envelope_for(owner),
    );
    let before_ops = vec![
        group_create(0),
        op(
            1,
            owner,
            OpKind::EntityCreate {
                entity: site.clone(),
            },
        ),
        op(
            2,
            owner,
            OpKind::TrustGrantCreate {
                grant: grant.clone(),
            },
        ),
    ];
    let (before, _) = fold(before_ops);
    assert_eq!(
        project(&before, &ViewerContext::Person { person: trusted }, 1)
            .entities
            .len(),
        1
    );

    let after_ops = vec![
        group_create(0),
        op(1, owner, OpKind::EntityCreate { entity: site }),
        op(2, owner, OpKind::TrustGrantCreate { grant }),
        op(
            3,
            owner,
            OpKind::TrustGrantRevoke {
                grant: grant_id(1),
                at: ts(3),
            },
        ),
    ];
    let (after, _) = fold(after_ops);
    assert!(
        project(&after, &ViewerContext::Person { person: trusted }, 2)
            .entities
            .is_empty()
    );
}

#[test]
fn archived_member_loses_group_access() {
    let mut state = base_state();
    member(&mut state, 2, GroupRole::Member, Lifecycle::Archived);
    state.entities.insert(
        entity_id(1),
        entity(1, None, Circle::Group, SensitivityTier::T1),
    );
    assert!(
        project(&state, &ViewerContext::Person { person: person(2) }, 1)
            .entities
            .is_empty()
    );
}

#[test]
fn story_elision_order_and_empty_story_rules() {
    let owner = person(1);
    let mut state = base_state();
    state.entities.insert(
        entity_id(1),
        entity(1, None, Circle::Public, SensitivityTier::T0),
    );
    state.entities.insert(
        entity_id(2),
        entity(2, Some(owner), Circle::Private, SensitivityTier::T0),
    );
    state.entities.insert(
        entity_id(3),
        entity(3, None, Circle::Public, SensitivityTier::T0),
    );
    state.stories.insert(
        story_id(1),
        story(1, vec![1, 2, 3], Circle::Public, SensitivityTier::T0),
    );
    state.stories.insert(
        story_id(2),
        story(2, vec![1], Circle::Private, SensitivityTier::T0),
    );
    state.stories.insert(
        story_id(3),
        story(3, vec![2], Circle::Public, SensitivityTier::T0),
    );
    let projection = project(&state, &ViewerContext::Anonymous, 1);
    let visible = projection
        .stories
        .iter()
        .find(|story| story.id == story_id(1))
        .expect("story");
    assert_eq!(
        visible
            .steps
            .iter()
            .map(|step| step.entity)
            .collect::<Vec<_>>(),
        vec![entity_id(1), entity_id(3)]
    );
    assert!(
        !projection
            .stories
            .iter()
            .any(|story| story.id == story_id(2))
    );
    let empty = projection
        .stories
        .iter()
        .find(|story| story.id == story_id(3))
        .expect("empty visible story");
    assert!(empty.steps.is_empty());
}

#[test]
fn redact_report_obeys_no_counts_rule() {
    let mut state = base_state();
    member(&mut state, 2, GroupRole::Member, Lifecycle::Active);
    member(&mut state, 3, GroupRole::Governance, Lifecycle::Active);
    state.entities.insert(
        entity_id(1),
        entity(1, None, Circle::Group, SensitivityTier::T1),
    );
    let report = StoreReport {
        quarantined: vec![QuarantineEntry {
            op_id: op_id(1),
            responsible_human: person(4),
            subject: Some(SubjectRef::Entity(entity_id(99))),
            reason: QuarantineReason::MissingTarget,
            findings: Vec::new(),
        }],
        warnings: vec![
            StoreFinding {
                code: "Visible".to_string(),
                message: format!("finding for {}", entity_id(1)),
                subject: Some(SubjectRef::Entity(entity_id(1))),
            },
            StoreFinding {
                code: "Hidden".to_string(),
                message: format!("finding for {}", entity_id(99)),
                subject: Some(SubjectRef::Entity(entity_id(99))),
            },
        ],
        applied: 7,
        deduped: 3,
    };
    let own = redact_report(
        &state,
        &ViewerContext::Person { person: person(4) },
        &report,
    );
    assert_eq!(own.quarantined[0].reason, QuarantineReason::NotFound);
    assert!(own.warnings.is_empty());
    assert_eq!(own.applied, 0);

    let governance = redact_report(
        &state,
        &ViewerContext::Person { person: person(3) },
        &report,
    );
    assert_eq!(governance, report);

    let member_report = redact_report(
        &state,
        &ViewerContext::Person { person: person(2) },
        &report,
    );
    assert_eq!(member_report.warnings.len(), 1);
    assert!(member_report.quarantined.is_empty());

    let anonymous = redact_report(&state, &ViewerContext::Anonymous, &report);
    assert!(anonymous.warnings.is_empty());
    assert!(anonymous.quarantined.is_empty());
}

#[test]
fn report_redaction_ignores_hidden_message_text_that_contains_visible_kind() {
    let mut state = base_state();
    member(&mut state, 2, GroupRole::Member, Lifecycle::Active);
    state.entities.insert(
        entity_id(1),
        entity(1, None, Circle::Group, SensitivityTier::T1),
    );
    state.entities.insert(
        entity_id(2),
        entity(2, Some(person(1)), Circle::Private, SensitivityTier::T1),
    );
    let report = StoreReport {
        quarantined: Vec::new(),
        warnings: vec![StoreFinding {
            code: "Hidden".to_string(),
            message: format!("hidden finding mentions visible kind {}", kind_id()),
            subject: Some(SubjectRef::Entity(entity_id(2))),
        }],
        applied: 0,
        deduped: 0,
    };

    let redacted = redact_report(
        &state,
        &ViewerContext::Person { person: person(2) },
        &report,
    );

    assert!(redacted.warnings.is_empty());
}

#[test]
fn submitter_sees_own_hidden_quarantine_scrubbed_to_not_found() {
    let mut state = base_state();
    member(&mut state, 2, GroupRole::Member, Lifecycle::Active);
    state.entities.insert(
        entity_id(1),
        entity(1, Some(person(1)), Circle::Private, SensitivityTier::T1),
    );
    let report = StoreReport {
        quarantined: vec![QuarantineEntry {
            op_id: op_id(7),
            responsible_human: person(2),
            subject: Some(SubjectRef::Entity(entity_id(1))),
            reason: QuarantineReason::FailedValidation,
            findings: vec![cn_schema::Finding {
                code: cn_schema::FindingCode::RequiredAttrMissing,
                path: format!("/entities/{}", entity_id(1)),
                message: format!("hidden {}", entity_id(1)),
            }],
        }],
        warnings: Vec::new(),
        applied: 0,
        deduped: 0,
    };

    let redacted = redact_report(
        &state,
        &ViewerContext::Person { person: person(2) },
        &report,
    );

    assert_eq!(redacted.quarantined.len(), 1);
    assert_eq!(redacted.quarantined[0].reason, QuarantineReason::NotFound);
    assert!(redacted.quarantined[0].findings.is_empty());
}

#[test]
fn authorizer_rules_cover_positive_and_negative_cases() {
    let authz = PermAuthorizer;
    let mut state = base_state();
    member(&mut state, 1, GroupRole::Governance, Lifecycle::Active);
    member(&mut state, 2, GroupRole::Member, Lifecycle::Active);
    state.entities.insert(
        entity_id(1),
        entity(1, Some(person(2)), Circle::Group, SensitivityTier::T1),
    );
    state.entities.insert(
        entity_id(2),
        entity(2, None, Circle::Private, SensitivityTier::T1),
    );
    state.entities.insert(
        entity_id(3),
        entity(3, None, Circle::Group, SensitivityTier::T1),
    );
    state.edges.insert(
        edge_id(1),
        edge(1, 1, 3, Circle::Group, SensitivityTier::T1),
    );
    state.stories.insert(
        story_id(1),
        story(1, vec![1], Circle::Group, SensitivityTier::T1),
    );
    state
        .entities
        .get_mut(&entity_id(1))
        .expect("entity")
        .attributes
        .insert(attr_id(0), attr("Synthetic", Circle::Group));
    let grant = TrustGrant::new(
        grant_id(1),
        person(2),
        person(3),
        TrustScope::All,
        ts(1),
        envelope_for(person(2)),
    );
    state.trust_grants.insert(grant.id, grant);

    assert!(
        authz
            .authorize(&GroupState::default(), &group_create(1))
            .is_ok()
    );
    assert_code(authz.authorize(&state, &group_create(1)), "group_exists");
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    2,
                    person(2),
                    OpKind::EntityCreate {
                        entity: entity(4, Some(person(2)), Circle::Group, SensitivityTier::T1)
                    }
                )
            )
            .is_ok()
    );
    assert_code(
        authz.authorize(
            &state,
            &op(
                3,
                person(3),
                OpKind::EntityCreate {
                    entity: entity(5, Some(person(2)), Circle::Group, SensitivityTier::T1),
                },
            ),
        ),
        "not_owner_or_governance",
    );
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    4,
                    person(2),
                    OpKind::EdgeCreate {
                        edge: edge(2, 1, 3, Circle::Group, SensitivityTier::T1)
                    }
                )
            )
            .is_ok()
    );
    assert_code(
        authz.authorize(
            &state,
            &op(
                5,
                person(3),
                OpKind::StoryCreate {
                    story: story(2, vec![1], Circle::Group, SensitivityTier::T1),
                },
            ),
        ),
        "not_member",
    );
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    6,
                    person(2),
                    OpKind::AttributeSet {
                        entity: entity_id(1),
                        attr: attr_id(1),
                        instance: attr("Synthetic", Circle::Group)
                    }
                )
            )
            .is_ok()
    );
    assert_code(
        authz.authorize(
            &state,
            &op(
                7,
                person(3),
                OpKind::AttributeRemove {
                    entity: entity_id(1),
                    attr: attr_id(0),
                },
            ),
        ),
        "not_owner_or_governance",
    );
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    8,
                    person(2),
                    OpKind::EntityLifecycleSet {
                        entity: entity_id(1),
                        lifecycle: Lifecycle::Archived
                    }
                )
            )
            .is_ok()
    );
    assert_code(
        authz.authorize(
            &state,
            &op(
                9,
                person(2),
                OpKind::StoryLifecycleSet {
                    story: story_id(1),
                    lifecycle: Lifecycle::Archived,
                },
            ),
        ),
        "not_governance",
    );
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    10,
                    person(2),
                    OpKind::VisibilitySet {
                        target: VisibilityTarget::EntityPresence(entity_id(1)),
                        visibility: Circle::Trusted
                    }
                )
            )
            .is_ok()
    );
    assert_code(
        authz.authorize(
            &state,
            &op(
                11,
                person(2),
                OpKind::VisibilitySet {
                    target: VisibilityTarget::Edge(edge_id(1)),
                    visibility: Circle::Trusted,
                },
            ),
        ),
        "not_governance",
    );
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    12,
                    person(1),
                    OpKind::TierSet {
                        target: TierTarget::Entity(entity_id(1)),
                        tier: SensitivityTier::T0
                    }
                )
            )
            .is_ok()
    );
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    13,
                    person(2),
                    OpKind::TierSet {
                        target: TierTarget::Entity(entity_id(1)),
                        tier: SensitivityTier::T2
                    }
                )
            )
            .is_ok()
    );
    assert_code(
        authz.authorize(
            &state,
            &op(
                14,
                person(2),
                OpKind::TierSet {
                    target: TierTarget::Entity(entity_id(1)),
                    tier: SensitivityTier::T0,
                },
            ),
        ),
        "tier_not_tightened",
    );
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    15,
                    person(1),
                    OpKind::MembershipAdd {
                        membership: Membership::new(
                            membership_id(20),
                            group_id(),
                            person(3),
                            GroupRole::Member,
                            envelope_for(person(1))
                        )
                    }
                )
            )
            .is_ok()
    );
    let mut bootstrap = base_state();
    let bootstrap_membership = Membership::new(
        membership_id(21),
        group_id(),
        person(1),
        GroupRole::Governance,
        envelope_for(person(1)),
    );
    assert!(
        authz
            .authorize(
                &bootstrap,
                &op(
                    16,
                    person(1),
                    OpKind::MembershipAdd {
                        membership: bootstrap_membership
                    }
                )
            )
            .is_ok()
    );
    bootstrap.group = Some(group());
    assert_code(
        authz.authorize(
            &state,
            &op(
                17,
                person(2),
                OpKind::MembershipLifecycleSet {
                    membership: membership_id(1),
                    lifecycle: Lifecycle::Archived,
                },
            ),
        ),
        "not_governance",
    );
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    18,
                    person(2),
                    OpKind::TrustGrantCreate {
                        grant: TrustGrant::new(
                            grant_id(2),
                            person(2),
                            person(3),
                            TrustScope::All,
                            ts(1),
                            envelope_for(person(2))
                        )
                    }
                )
            )
            .is_ok()
    );
    assert_code(
        authz.authorize(
            &state,
            &op(
                19,
                person(3),
                OpKind::TrustGrantRevoke {
                    grant: grant_id(1),
                    at: ts(2),
                },
            ),
        ),
        "grantor_mismatch",
    );
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    20,
                    person(2),
                    OpKind::CustodyAppend {
                        target: CustodyTarget::Entity(entity_id(1)),
                        event: custody(person(2))
                    }
                )
            )
            .is_ok()
    );
    assert_code(
        authz.authorize(
            &state,
            &op(
                21,
                person(2),
                OpKind::CustodyAppend {
                    target: CustodyTarget::Entity(entity_id(2)),
                    event: custody(person(2)),
                },
            ),
        ),
        "target_hidden",
    );
    assert!(
        authz
            .authorize(
                &state,
                &op(
                    22,
                    person(1),
                    OpKind::CustodyAppend {
                        target: CustodyTarget::Entity(entity_id(2)),
                        event: custody(person(1))
                    }
                )
            )
            .is_ok()
    );
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

fn assert_code(result: Result<(), AuthzDenial>, code: &str) {
    let denial = result.expect_err("operation denied");
    assert_eq!(denial.code, code);
}

#[test]
fn fingerprint_distinguishes_member_facilitator_and_dual_role() {
    let viewer = ViewerContext::Person { person: person(2) };

    let mut as_member = base_state();
    member(&mut as_member, 2, GroupRole::Member, Lifecycle::Active);
    let member_fp = viewer_fingerprint(&as_member, &viewer);

    let mut as_facilitator = base_state();
    member(
        &mut as_facilitator,
        2,
        GroupRole::Facilitator,
        Lifecycle::Active,
    );
    let facilitator_fp = viewer_fingerprint(&as_facilitator, &viewer);

    let mut as_dual = base_state();
    member(&mut as_dual, 2, GroupRole::Facilitator, Lifecycle::Active);
    let mut second = Membership::new(
        membership_id(90),
        group_id(),
        person(2),
        GroupRole::Member,
        envelope_for(person(2)),
    );
    second.lifecycle = Lifecycle::Active;
    as_dual.memberships.insert(second.id, second);
    let dual_fp = viewer_fingerprint(&as_dual, &viewer);

    assert_ne!(member_fp, facilitator_fp);
    assert_ne!(member_fp, dual_fp);
    assert_ne!(facilitator_fp, dual_fp);
}

#[test]
fn cache_key_survives_fingerprint_collision() {
    // CRC32 cross-role collision pair engineered by the 2026-07-17
    // adversarial round (Codex session 019f7389-9e32-78a2-94ee-7a1b2827f3d3):
    // in one group state, this member and this facilitator collide at the
    // hashed 32-bit viewer_fingerprint for ANY shared template-version
    // suffix. Caches must therefore key on viewer_cache_key (the canonical
    // context), which is collision-free by construction.
    let member_person: PersonId =
        PersonId::from_str("8a6a5a71-4e38-f6e4-e021-29edf7ed1c20").expect("person id");
    let facilitator_person: PersonId =
        PersonId::from_str("56db767d-724e-8fb1-923c-a06813fa87d8").expect("person id");
    let mut state = base_state();
    let member_membership = Membership::new(
        membership_id(91),
        group_id(),
        member_person,
        GroupRole::Member,
        envelope_for(member_person),
    );
    state
        .memberships
        .insert(member_membership.id, member_membership);
    let facilitator_membership = Membership::new(
        membership_id(92),
        group_id(),
        facilitator_person,
        GroupRole::Facilitator,
        envelope_for(facilitator_person),
    );
    state
        .memberships
        .insert(facilitator_membership.id, facilitator_membership);

    let member_viewer = ViewerContext::Person {
        person: member_person,
    };
    let facilitator_viewer = ViewerContext::Person {
        person: facilitator_person,
    };
    assert_eq!(
        viewer_fingerprint(&state, &member_viewer),
        viewer_fingerprint(&state, &facilitator_viewer),
        "collision pair no longer collides - the fingerprint input format \
         moved; engineer a new pair before trusting this test"
    );
    assert_ne!(
        viewer_cache_key(&state, &member_viewer),
        viewer_cache_key(&state, &facilitator_viewer),
        "cache keys must never collide across distinct authorization contexts"
    );
}

#[test]
fn facilitator_reads_at_member_level_and_report_is_not_governance() {
    let mut state = base_state();
    member(&mut state, 2, GroupRole::Member, Lifecycle::Active);
    member(&mut state, 5, GroupRole::Facilitator, Lifecycle::Active);
    state.entities.insert(
        entity_id(1),
        entity(1, None, Circle::Group, SensitivityTier::T1),
    );
    state.entities.insert(
        entity_id(2),
        entity(2, Some(person(1)), Circle::Private, SensitivityTier::T1),
    );

    let member_projection = project(&state, &ViewerContext::Person { person: person(2) }, 1);
    let facilitator_projection = project(&state, &ViewerContext::Person { person: person(5) }, 1);
    assert_eq!(
        member_projection
            .entities
            .iter()
            .map(|entity| entity.id)
            .collect::<Vec<_>>(),
        facilitator_projection
            .entities
            .iter()
            .map(|entity| entity.id)
            .collect::<Vec<_>>(),
    );
    assert_eq!(facilitator_projection.entities.len(), 1);

    let report = StoreReport {
        quarantined: vec![QuarantineEntry {
            op_id: op_id(1),
            responsible_human: person(4),
            subject: Some(SubjectRef::Entity(entity_id(2))),
            reason: QuarantineReason::MissingTarget,
            findings: Vec::new(),
        }],
        warnings: vec![StoreFinding {
            code: "Hidden".to_string(),
            message: format!("finding for {}", entity_id(2)),
            subject: Some(SubjectRef::Entity(entity_id(2))),
        }],
        applied: 7,
        deduped: 3,
    };
    let facilitator_view = redact_report(
        &state,
        &ViewerContext::Person { person: person(5) },
        &report,
    );
    assert_ne!(facilitator_view, report);
    assert!(facilitator_view.quarantined.is_empty());
    assert!(facilitator_view.warnings.is_empty());
    assert_eq!(facilitator_view.applied, 0);
    assert_eq!(facilitator_view.deduped, 0);
}

#[test]
fn fingerprint_changes_only_for_relevant_viewer_state() {
    let viewer = ViewerContext::Person { person: person(2) };
    let mut state = base_state();
    let first = viewer_fingerprint(&state, &viewer);
    let grant = TrustGrant::new(
        grant_id(1),
        person(1),
        person(2),
        TrustScope::All,
        ts(1),
        envelope_for(person(1)),
    );
    state.trust_grants.insert(grant.id, grant);
    let with_grant = viewer_fingerprint(&state, &viewer);
    assert_ne!(first, with_grant);
    state
        .trust_grants
        .get_mut(&grant_id(1))
        .expect("grant")
        .revoke(ts(2))
        .expect("revoke");
    assert_ne!(with_grant, viewer_fingerprint(&state, &viewer));
    member(&mut state, 2, GroupRole::Member, Lifecycle::Active);
    let with_role = viewer_fingerprint(&state, &viewer);
    assert_ne!(first, with_role);
    let mut changed_version = cn_model::model_schema_version();
    changed_version.minor = 2;
    state.group.as_mut().expect("group").template_version = changed_version;
    assert_ne!(with_role, viewer_fingerprint(&state, &viewer));
    let before_other = viewer_fingerprint(&state, &viewer);
    let other_grant = TrustGrant::new(
        grant_id(2),
        person(1),
        person(3),
        TrustScope::All,
        ts(3),
        envelope_for(person(1)),
    );
    state.trust_grants.insert(other_grant.id, other_grant);
    assert_eq!(before_other, viewer_fingerprint(&state, &viewer));
}
