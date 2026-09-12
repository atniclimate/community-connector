//! Integration coverage for S-R4a: cn-api's new graph_measures JSON call,
//! exercised against the real, fully synthetic ATNI convention fixture
//! (D-094c) rather than an inline projection, per this session's own gate.

use cn_api::Api;
use serde_json::{Value, json};

const ATNI_TEMPLATE: &str =
    include_str!("../../../../fixtures/templates/atni-convention.template.json");
const ATNI_OPS: &str = include_str!("../../../../fixtures/groups/atni-convention.ops.jsonl");

const GROUP_ID: &str = "00000000-0000-0000-0000-0000000dbba0";
// A "governance" member from the fixture (active MembershipAdd), so the
// projection is non-empty (Group-visibility values require Group access).
const GOVERNANCE_VIEWER: &str = "00000000-0000-0000-0000-0000000de2bd";

fn ok(json: &str) -> Value {
    let value: Value = serde_json::from_str(json).expect("valid envelope json");
    assert!(value.get("ok").is_some() ^ value.get("err").is_some());
    value
        .get("ok")
        .cloned()
        .unwrap_or_else(|| panic!("ok envelope: {json}"))
}

fn viewer() -> String {
    json!({ "kind": "person", "person": GOVERNANCE_VIEWER }).to_string()
}

fn load_atni_api() -> Api {
    let mut api = Api::new();
    ok(&api.load_group_begin(GROUP_ID, &viewer(), ATNI_TEMPLATE));
    ok(&api.load_ops_chunk(GROUP_ID, ATNI_OPS));
    ok(&api.load_group_commit(GROUP_ID, 1_000));
    api
}

#[test]
fn graph_measures_returns_all_five_measures_over_the_atni_fixture() {
    let mut api = load_atni_api();
    let projection: Value = ok(&api.projection(GROUP_ID, &viewer()));
    let entities = projection["entities"].as_array().expect("entities array");
    assert!(entities.len() > 2, "fixture projection unexpectedly empty");
    let person_ids: Vec<&str> = entities
        .iter()
        .filter(|e| e["kind"] == "person")
        .map(|e| e["id"].as_str().expect("id"))
        .take(2)
        .collect();
    assert_eq!(
        person_ids.len(),
        2,
        "fixture must contain at least two people"
    );

    let request = json!({
        "membership_kind": "member_of",
        "jaccard_pairs": [{ "a": person_ids[0], "b": person_ids[1] }],
    })
    .to_string();
    let measures = ok(&api.graph_measures(GROUP_ID, &viewer(), &request));

    let degree = measures["degree"].as_object().expect("degree map");
    assert_eq!(degree.len(), entities.len());
    for measure in degree.values() {
        assert!(
            !measure["explanation"]
                .as_str()
                .unwrap_or_default()
                .is_empty()
        );
    }

    let betweenness = measures["betweenness"]
        .as_object()
        .expect("betweenness map");
    assert_eq!(betweenness.len(), entities.len());
    assert!(
        betweenness
            .values()
            .any(|m| m["value"].as_f64().unwrap_or(0.0) > 0.0),
        "at least one entity should sit on some shortest path in this fixture"
    );

    let eccentricity = measures["eccentricity"]
        .as_object()
        .expect("eccentricity map");
    assert_eq!(eccentricity.len(), entities.len());

    let jaccard = measures["shared_committee_jaccard"]
        .as_array()
        .expect("jaccard array");
    assert_eq!(jaccard.len(), 1);
    let pair = &jaccard[0];
    let shared = pair["shared"].as_u64().expect("shared");
    let union = pair["union"].as_u64().expect("union");
    assert!(shared <= union);
    if union > 0 {
        let expected = shared as f64 / union as f64;
        assert!((pair["value"].as_f64().unwrap() - expected).abs() < 1e-9);
    } else {
        assert_eq!(pair["value"].as_f64().unwrap(), 0.0);
    }
}

#[test]
fn graph_measures_rejects_unknown_jaccard_entity() {
    let mut api = load_atni_api();
    let request = json!({
        "membership_kind": "member_of",
        "jaccard_pairs": [{
            "a": "00000000-0000-0000-0000-000000000000",
            "b": "00000000-0000-0000-0000-000000000000",
        }],
    })
    .to_string();
    let response: Value = serde_json::from_str(&api.graph_measures(GROUP_ID, &viewer(), &request))
        .expect("valid envelope json");
    assert_eq!(response["err"]["code"], "not_found");
}
