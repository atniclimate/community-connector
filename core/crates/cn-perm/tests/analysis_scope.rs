//! Analysis scope for continuance and introduction results. Synthetic data.

use std::str::FromStr;

use cn_model::*;
use cn_perm::*;
use cn_store::*;

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

fn envelope_for(who: PersonId) -> ProvenanceEnvelope {
    ProvenanceEnvelope::new(Origin::Authored, ActorRef::Human(who), who, Timestamp(1))
        .expect("valid provenance")
}

fn member(state: &mut GroupState, slot: u128, n: u128, role: GroupRole, lifecycle: Lifecycle) {
    let mut membership = Membership::new(
        id(500 + slot),
        id(10),
        person(n),
        role,
        envelope_for(person(n)),
    );
    membership.lifecycle = lifecycle;
    state.memberships.insert(membership.id, membership);
}

fn state() -> GroupState {
    let mut state = GroupState::default();
    state.group = Some(
        Group::new(
            id(10),
            "Synthetic Group",
            TemplateId::new("synthetic").expect("template id"),
            cn_model::model_schema_version(),
            envelope_for(person(1)),
            SensitivityTier::T1,
        )
        .expect("valid group"),
    );
    member(&mut state, 1, 1, GroupRole::Governance, Lifecycle::Active);
    member(&mut state, 2, 2, GroupRole::Member, Lifecycle::Active);
    member(&mut state, 3, 3, GroupRole::Facilitator, Lifecycle::Active);
    member(&mut state, 4, 4, GroupRole::Governance, Lifecycle::Archived);
    member(&mut state, 5, 5, GroupRole::Member, Lifecycle::Active);
    member(&mut state, 6, 5, GroupRole::Governance, Lifecycle::Active);
    state
}

#[test]
fn anonymous_gets_nothing() {
    assert_eq!(
        analysis_scope(&state(), &ViewerContext::Anonymous),
        AnalysisScope::Nothing
    );
}

#[test]
fn active_governance_gets_all() {
    let state = state();
    for n in [1, 5] {
        let viewer = ViewerContext::Person { person: person(n) };
        assert_eq!(
            analysis_scope(&state, &viewer),
            AnalysisScope::All,
            "person {n}"
        );
    }
}

#[test]
fn everyone_else_gets_own_records() {
    let state = state();
    // member, facilitator, archived governance, non-member
    for n in [2, 3, 4, 9] {
        let viewer = ViewerContext::Person { person: person(n) };
        assert_eq!(
            analysis_scope(&state, &viewer),
            AnalysisScope::OwnRecords(person(n)),
            "person {n}"
        );
    }
}
