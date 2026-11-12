//! Continuance, introductions, and the record mirror through cn-api, over
//! the fully synthetic relatives-demo fixture. Each test asserts a planted
//! structure documented in fixtures/README.md.

use cn_api::Api;
use serde_json::{Value, json};

const TEMPLATE: &str = include_str!("../../../../fixtures/templates/relatives-demo.template.json");
const OPS: &str = include_str!("../../../../fixtures/groups/relatives-demo.ops.jsonl");

const GROUP_ID: &str = "00000000-0000-0000-0000-000000124f80";
const GOVERNANCE: &str = "00000000-0000-0000-0000-000000127692";
const LIAISON: &str = "00000000-0000-0000-0000-000000127694";
const BRIDGE_EDGE: &str = "00000000-0000-0000-0000-000000133a2c";
const SEEKER: &str = "00000000-0000-0000-0000-000000127699";
const WILLING_INTRODUCER: &str = "00000000-0000-0000-0000-0000001276a5";
const OFFERER: &str = "00000000-0000-0000-0000-0000001276a9";
const SECOND_SEEKER: &str = "00000000-0000-0000-0000-00000012769f";
const UNWILLING_INTRODUCER: &str = "00000000-0000-0000-0000-0000001276a6";
const SECOND_OFFERER: &str = "00000000-0000-0000-0000-0000001276ac";
const TIER_CAPPED: &str = "00000000-0000-0000-0000-0000001276aa";
const ABSENT: &str = "00000000-0000-0000-0000-0000009999ff";

fn envelope(json: &str) -> Value {
    serde_json::from_str(json).expect("valid envelope json")
}

fn ok(json: &str) -> Value {
    envelope(json)
        .get("ok")
        .cloned()
        .unwrap_or_else(|| panic!("ok envelope: {json}"))
}

fn person(id: &str) -> String {
    json!({ "kind": "person", "person": id }).to_string()
}

fn anonymous() -> String {
    json!({ "kind": "anonymous" }).to_string()
}

fn load() -> Api {
    let mut api = Api::new();
    ok(&api.load_group_begin(GROUP_ID, &person(GOVERNANCE), TEMPLATE));
    ok(&api.load_ops_chunk(GROUP_ID, OPS));
    ok(&api.load_group_commit(GROUP_ID, 1_000));
    api
}

fn continuance(api: &mut Api, viewer: &str) -> Value {
    ok(&api.graph_continuance(GROUP_ID, viewer, "{}"))
}

fn introductions(api: &mut Api, viewer: &str, center: &str) -> String {
    api.suggest_introductions(GROUP_ID, viewer, &json!({ "center": center }).to_string())
}

#[test]
fn liaison_is_an_articulation_point_for_governance_and_in_its_own_view() {
    let mut api = load();
    let all = continuance(&mut api, &person(GOVERNANCE));
    assert!(all["articulation_points"].get(LIAISON).is_some());
    assert!(all["bridges"].get(BRIDGE_EDGE).is_some());
    assert!(
        !all["components"]
            .as_object()
            .expect("components")
            .is_empty()
    );

    let own = continuance(&mut api, &person(LIAISON));
    let points = own["articulation_points"].as_object().expect("points");
    assert_eq!(points.keys().collect::<Vec<_>>(), vec![LIAISON]);
    let bridges = own["bridges"].as_object().expect("bridges");
    assert!(bridges.contains_key(BRIDGE_EDGE));
    for tie in bridges.values() {
        assert!(tie["from"] == LIAISON || tie["to"] == LIAISON, "{tie}");
    }
    assert!(
        own["components"]
            .as_object()
            .expect("components")
            .is_empty()
    );

    // A member who is not the liaison does not see the liaison's position.
    let other = continuance(&mut api, &person(SEEKER));
    assert!(other["articulation_points"].get(LIAISON).is_none());
    assert!(other["bridges"].get(BRIDGE_EDGE).is_none());

    let nothing = continuance(&mut api, &anonymous());
    assert_eq!(
        nothing,
        json!({ "articulation_points": {}, "bridges": {}, "components": {} })
    );
}

fn introducer_for(suggestions: &Value, candidate: &str, introducer: &str) -> Value {
    let suggestion = suggestions
        .as_array()
        .expect("suggestions")
        .iter()
        .find(|s| s["candidate"] == candidate)
        .unwrap_or_else(|| panic!("{candidate} suggested in {suggestions}"));
    suggestion["introducers"]
        .as_array()
        .expect("introducers")
        .iter()
        .find(|i| i["entity"] == introducer)
        .cloned()
        .unwrap_or_else(|| panic!("{introducer} introduces {candidate}"))
}

#[test]
fn seekers_get_their_planted_pairs_with_willingness() {
    let mut api = load();
    let first = ok(&introductions(&mut api, &person(SEEKER), SEEKER));
    assert_eq!(
        introducer_for(&first, OFFERER, WILLING_INTRODUCER)["willing"],
        true
    );
    let second = ok(&introductions(
        &mut api,
        &person(SECOND_SEEKER),
        SECOND_SEEKER,
    ));
    assert_eq!(
        introducer_for(&second, SECOND_OFFERER, UNWILLING_INTRODUCER)["willing"],
        false
    );
    // Governance may center anyone.
    let governed = ok(&introductions(&mut api, &person(GOVERNANCE), SEEKER));
    assert_eq!(governed, first);
}

#[test]
fn a_center_the_viewer_does_not_own_is_not_found() {
    let mut api = load();
    let not_owned = introductions(&mut api, &person(SEEKER), SECOND_SEEKER);
    let absent = introductions(&mut api, &person(SEEKER), ABSENT);
    assert_eq!(envelope(&not_owned)["err"]["code"], "not_found");
    assert_eq!(not_owned, absent);
    let anonymous = introductions(&mut api, &anonymous(), SEEKER);
    assert_eq!(anonymous, absent);
}

#[test]
fn mirror_shows_the_t2_tier_cap_and_anonymous_is_empty() {
    let api = load();
    let mirror = ok(&api.record_mirror(GROUP_ID, &person(TIER_CAPPED)));
    assert_eq!(mirror["owner"], TIER_CAPPED);
    let offering = &mirror["entities"][TIER_CAPPED]["attributes"]["offering"];
    assert_eq!(offering["visibility"], "network");
    assert_eq!(offering["effective_tier"], "T2");
    assert_eq!(offering["ceiling"], "group");
    assert_eq!(offering["tier_capped"], true);

    let empty = ok(&api.record_mirror(GROUP_ID, &anonymous()));
    assert_eq!(empty, json!({ "owner": null, "entities": {} }));
}
