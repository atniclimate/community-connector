//! Record mirror acceptance: per-audience cells, the spec section 8
//! fisheries scenario, consistency with `project`, and record scoping.
//! All data is synthetic.

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use cn_model::*;
use cn_perm::*;
use cn_store::*;

const CIRCLES: [Circle; 5] = [
    Circle::Private,
    Circle::Trusted,
    Circle::Group,
    Circle::Network,
    Circle::Public,
];
const TIERS: [SensitivityTier; 4] = [
    SensitivityTier::T0,
    SensitivityTier::T1,
    SensitivityTier::T2,
    SensitivityTier::T3,
];

fn id<T>(n: u128) -> T
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    let hex = format!("{n:032x}");
    let text = format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    );
    T::from_str(&text).expect("valid deterministic id")
}

fn person(n: u128) -> PersonId {
    id(n)
}

fn group_id() -> GroupId {
    id(10)
}

fn envelope_for(who: PersonId) -> ProvenanceEnvelope {
    ProvenanceEnvelope::new(Origin::Authored, ActorRef::Human(who), who, Timestamp(1))
        .expect("valid provenance")
}

fn base_state() -> GroupState {
    let mut state = GroupState::default();
    state.group = Some(
        Group::new(
            group_id(),
            "Synthetic Group",
            TemplateId::new("synthetic").expect("template id"),
            cn_model::model_schema_version(),
            envelope_for(person(1)),
            SensitivityTier::T1,
        )
        .expect("valid group"),
    );
    state
}

fn member(state: &mut GroupState, n: u128, role: GroupRole, lifecycle: Lifecycle) {
    let mut membership = Membership::new(
        id(500 + n),
        group_id(),
        person(n),
        role,
        envelope_for(person(n)),
    );
    membership.lifecycle = lifecycle;
    state.memberships.insert(membership.id, membership);
}

fn trust(state: &mut GroupState, n: u128, grantor: PersonId, grantee: PersonId) {
    let grant = TrustGrant::new(
        id(600 + n),
        grantor,
        grantee,
        TrustScope::All,
        Timestamp(1),
        envelope_for(grantor),
    );
    state.trust_grants.insert(grant.id, grant);
}

fn entity(n: u128, owner: Option<PersonId>, presence: Circle, tier: SensitivityTier) -> Entity {
    let mut entity = Entity::new(
        id(100 + n),
        group_id(),
        KindId::new("kind").expect("kind id"),
        presence,
        envelope_for(person(1)),
        tier,
    );
    entity.owner = owner;
    entity
}

fn attr(
    visibility: Circle,
    container: SensitivityTier,
    tier: SensitivityTier,
) -> AttributeInstance {
    let mut instance = AttributeInstance::new(
        AttributeValue::Text("synthetic".to_string()),
        visibility,
        envelope_for(person(1)),
    );
    if tier > container {
        instance
            .tighten_tier(container, tier)
            .expect("tier tightens");
    }
    instance
}

fn attr_id(name: &str) -> AttrId {
    AttrId::new(name).expect("attr id")
}

fn circle_name(circle: Circle) -> &'static str {
    match circle {
        Circle::Private => "private",
        Circle::Trusted => "trusted",
        Circle::Group => "group",
        Circle::Network => "network",
        Circle::Public => "public",
    }
}

fn flags(audiences: &[Audience]) -> BTreeMap<Audience, bool> {
    Audience::ALL
        .into_iter()
        .map(|audience| (audience, audiences.contains(&audience)))
        .collect()
}

#[test]
fn cells_cover_each_audience_across_every_circle() {
    use Audience::*;
    let owner = person(1);
    let mut state = base_state();
    let mut record = entity(1, Some(owner), Circle::Public, SensitivityTier::T0);
    for circle in CIRCLES {
        record.attributes.insert(
            attr_id(circle_name(circle)),
            attr(circle, SensitivityTier::T0, SensitivityTier::T0),
        );
    }
    state.entities.insert(record.id, record);

    let mirror = record_mirror(&state, owner);
    let site = &mirror.entities[&id(101)];
    assert_eq!(
        site.presence.audiences,
        flags(&[Public, Network, Group, Trusted])
    );
    let expected: [(&str, &[Audience]); 5] = [
        ("private", &[]),
        ("trusted", &[Trusted]),
        ("group", &[Group, Trusted]),
        ("network", &[Network, Group, Trusted]),
        ("public", &[Public, Network, Group, Trusted]),
    ];
    for (name, audiences) in expected {
        let cell = &site.attributes[&attr_id(name)];
        assert_eq!(cell.audiences, flags(audiences), "cell {name}");
        assert!(!cell.tier_capped, "T0 never caps {name}");
        assert!(
            cell.ceiling_blocks.is_empty(),
            "T0 blocks nothing for {name}"
        );
        assert_eq!(cell.ceiling, Circle::Public);
    }
}

#[test]
fn tier_ceiling_blocks_are_reported_as_gaps() {
    use Audience::*;
    let owner = person(1);
    let mut state = base_state();
    let mut record = entity(1, Some(owner), Circle::Group, SensitivityTier::T2);
    record.attributes.insert(
        attr_id("note"),
        attr(Circle::Public, SensitivityTier::T2, SensitivityTier::T2),
    );
    state.entities.insert(record.id, record);

    let cell = &record_mirror(&state, owner).entities[&id(101)].attributes[&attr_id("note")];
    assert_eq!(cell.ceiling, Circle::Group);
    assert!(cell.tier_capped);
    assert_eq!(cell.ceiling_blocks, BTreeSet::from([Public, Network]));
    assert_eq!(cell.audiences, flags(&[Group, Trusted]));
}

/// Spec section 8: a T1 fishing_site with a T3 site_knowledge override and a
/// trusted-only location.
#[test]
fn fisheries_scenario_mirror() {
    use Audience::*;
    let owner = person(1);
    let mut state = base_state();
    member(&mut state, 3, GroupRole::Member, Lifecycle::Active);
    trust(&mut state, 1, owner, person(2));
    let mut site = entity(1, Some(owner), Circle::Trusted, SensitivityTier::T1);
    site.attributes.insert(
        attr_id("location"),
        attr(Circle::Trusted, SensitivityTier::T1, SensitivityTier::T1),
    );
    site.attributes.insert(
        attr_id("site_knowledge"),
        attr(Circle::Public, SensitivityTier::T1, SensitivityTier::T3),
    );
    state.entities.insert(site.id, site);

    let mirror = record_mirror(&state, owner);
    assert_eq!(mirror.owner, owner);
    let site = &mirror.entities[&id(101)];

    assert_eq!(site.presence.visibility, Circle::Trusted);
    assert_eq!(site.presence.effective_tier, SensitivityTier::T1);
    assert_eq!(site.presence.ceiling, Circle::Network);
    assert_eq!(site.presence.audiences, flags(&[Trusted]));
    assert!(!site.presence.tier_capped);

    let location = &site.attributes[&attr_id("location")];
    assert_eq!(location.visibility, Circle::Trusted);
    assert_eq!(location.effective_tier, SensitivityTier::T1);
    assert_eq!(location.ceiling, Circle::Network);
    assert_eq!(location.audiences, flags(&[Trusted]));
    assert!(!location.tier_capped);
    assert!(location.ceiling_blocks.is_empty());

    let knowledge = &site.attributes[&attr_id("site_knowledge")];
    assert_eq!(knowledge.visibility, Circle::Public);
    assert_eq!(knowledge.effective_tier, SensitivityTier::T3);
    assert_eq!(knowledge.ceiling, Circle::Private);
    assert_eq!(knowledge.audiences, flags(&[]));
    assert!(knowledge.tier_capped);
    assert_eq!(knowledge.ceiling_blocks, BTreeSet::from(Audience::ALL));

    let json = serde_json::to_value(&mirror).expect("mirror serializes");
    let attrs = &json["entities"][id::<EntityId>(101).to_string()]["attributes"];
    assert_eq!(attrs["site_knowledge"]["ceiling"], "private");
    assert_eq!(attrs["location"]["audiences"]["trusted"], true);
}

/// Every presence circle and entity tier, each with every attribute circle
/// plus a T3 override, so presence gating and ceilings are both exercised.
fn consistency_state(owner: PersonId) -> GroupState {
    let mut state = base_state();
    member(&mut state, 2, GroupRole::Member, Lifecycle::Active);
    member(&mut state, 3, GroupRole::Member, Lifecycle::Active);
    trust(&mut state, 1, owner, person(2));
    let mut n = 0;
    for presence in CIRCLES {
        for tier in TIERS {
            n += 1;
            let mut record = entity(n, Some(owner), presence, tier);
            for circle in CIRCLES {
                record
                    .attributes
                    .insert(attr_id(circle_name(circle)), attr(circle, tier, tier));
            }
            record.attributes.insert(
                attr_id("override"),
                attr(Circle::Public, tier, SensitivityTier::T3),
            );
            state.entities.insert(record.id, record);
        }
    }
    state
}

#[test]
fn mirror_flags_match_project_for_representative_viewers() {
    let owner = person(1);
    let state = consistency_state(owner);
    let mirror = record_mirror(&state, owner);
    assert_eq!(mirror.entities.len(), CIRCLES.len() * TIERS.len());
    let viewers = [
        (Audience::Public, ViewerContext::Anonymous),
        (
            Audience::Network,
            ViewerContext::Person { person: person(9) },
        ),
        (Audience::Group, ViewerContext::Person { person: person(3) }),
        (
            Audience::Trusted,
            ViewerContext::Person { person: person(2) },
        ),
    ];
    for (audience, viewer) in viewers {
        let projection = project(&state, &viewer, 1);
        let projected: BTreeMap<_, _> = projection
            .entities
            .iter()
            .map(|entity| (entity.id, &entity.attributes))
            .collect();
        for (entity_id, entry) in &mirror.entities {
            let seen = projected.get(entity_id);
            assert_eq!(
                entry.presence.audiences[&audience],
                seen.is_some(),
                "{audience:?} presence {entity_id}"
            );
            for (attr, cell) in &entry.attributes {
                let received = seen.is_some_and(|attrs| attrs.contains_key(attr));
                assert_eq!(
                    cell.audiences[&audience], received,
                    "{audience:?} {entity_id} {attr}"
                );
            }
        }
    }
}

#[test]
fn archived_and_others_records_never_appear() {
    let owner = person(1);
    let mut state = base_state();
    let mine = entity(1, Some(owner), Circle::Public, SensitivityTier::T0);
    let mut archived = entity(2, Some(owner), Circle::Public, SensitivityTier::T0);
    archived.lifecycle = Lifecycle::Archived;
    let theirs = entity(3, Some(person(2)), Circle::Public, SensitivityTier::T0);
    let unowned = entity(4, None, Circle::Public, SensitivityTier::T0);
    for record in [mine, archived, theirs, unowned] {
        state.entities.insert(record.id, record);
    }

    let ids: Vec<EntityId> = record_mirror(&state, owner).entities.into_keys().collect();
    assert_eq!(ids, vec![id(101)]);
    assert!(record_mirror(&state, person(7)).entities.is_empty());
}
