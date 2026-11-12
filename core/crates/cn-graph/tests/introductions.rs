//! Introduction suggestions on planted shapes. Synthetic data only (I1).

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use cn_graph::*;
use cn_model::*;
use cn_perm::{ProjectedEdge, ProjectedEntity, Projection};

fn uuid<T: FromStr>(n: u128) -> T
where
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

fn node(n: u128) -> EntityId {
    uuid(100 + n)
}

fn kind(value: &str) -> KindId {
    KindId::new(value).expect("valid kind")
}

fn attr(value: &str) -> AttrId {
    AttrId::new(value).expect("valid attr")
}

fn tags(values: &[&str]) -> AttributeValue {
    AttributeValue::Tags(values.iter().map(|v| v.to_string()).collect())
}

/// A tie numbered by its position, so ids are stable across reorderings.
fn tie(n: usize, from: u128, to: u128, kind_name: &str) -> ProjectedEdge {
    ProjectedEdge {
        id: uuid(10_000 + n as u128),
        kind: kind(kind_name),
        from: node(from),
        to: node(to),
        directed: false,
        weight: None,
        attributes: BTreeMap::new(),
    }
}

struct Fixture {
    entities: BTreeMap<u128, ProjectedEntity>,
    edges: Vec<ProjectedEdge>,
}

impl Fixture {
    /// People 1..=n tied by "knows" pairs.
    fn new(n: u128, pairs: &[(u128, u128)]) -> Self {
        let entities = (1..=n)
            .map(|i| {
                let e = ProjectedEntity {
                    id: node(i),
                    kind: kind("person"),
                    owner_is_viewer: false,
                    attributes: BTreeMap::new(),
                };
                (i, e)
            })
            .collect();
        let mut f = Self {
            entities,
            edges: Vec::new(),
        };
        for &(a, b) in pairs {
            f.tie(a, b, "knows");
        }
        f
    }

    fn tie(&mut self, a: u128, b: u128, kind_name: &str) -> &mut Self {
        let n = self.edges.len() + 1;
        self.edges.push(tie(n, a, b, kind_name));
        self
    }

    fn set(&mut self, n: u128, name: &str, value: AttributeValue) -> &mut Self {
        let e = self.entities.get_mut(&n).expect("entity in fixture");
        e.attributes.insert(attr(name), value);
        self
    }

    fn projection(&self) -> Projection {
        Projection {
            group_id: uuid(10),
            viewer_fingerprint: "synthetic-viewer".to_string(),
            revision: 1,
            entities: self.entities.values().cloned().collect(),
            edges: self.edges.clone(),
            stories: Vec::new(),
        }
    }

    fn run(&self, req: &IntroductionRequest) -> Vec<IntroductionSuggestion> {
        let p = self.projection();
        suggest_introductions(&p, &GraphIndex::build(&p), req).expect("known center")
    }
}

fn candidates(out: &[IntroductionSuggestion]) -> Vec<EntityId> {
    out.iter().map(|s| s.candidate).collect()
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

#[test]
fn planted_triads_rank_by_adamic_adar_and_skip_known_and_far() {
    // 4 shares 2 and 3 with the center; 5 shares only 2; 6 is already tied
    // to the center; 7 is three ties away.
    let f = Fixture::new(
        7,
        &[
            (1, 2),
            (1, 3),
            (2, 4),
            (3, 4),
            (2, 5),
            (1, 6),
            (2, 6),
            (5, 7),
        ],
    );
    let out = f.run(&IntroductionRequest::new(node(1)));
    assert_eq!(candidates(&out), vec![node(4), node(5)]);
    // degree(2) = 4 (1, 4, 5, 6), degree(3) = 2.
    assert!(approx(out[0].score, 1.0 / 4f64.ln() + 1.0 / 2f64.ln()));
    assert!(approx(out[1].score, 1.0 / 4f64.ln()));
    let via: Vec<EntityId> = out[0].introducers.iter().map(|i| i.entity).collect();
    assert_eq!(via, vec![node(2), node(3)]);
    assert_eq!(out[0].shared_tags, SharedTags::default());
    assert_eq!(
        out[0].explanation,
        "2 mutual relatives, 0 willing to introduce"
    );
}

/// Candidate 5 shares introducer 2 (degree 2); 6 and 7 share introducer 4
/// (degree 3), so without tags the order is 5, 6, 7.
fn complement_fixture() -> Fixture {
    Fixture::new(7, &[(1, 2), (2, 5), (1, 4), (4, 6), (4, 7)])
}

#[test]
fn complementarity_raises_rank_in_either_direction() {
    let mut f = complement_fixture();
    let req = IntroductionRequest::new(node(1));
    assert_eq!(candidates(&f.run(&req)), vec![node(5), node(6), node(7)]);

    f.set(1, "seeking", tags(&["grant_writing", "data"]))
        .set(1, "offering", AttributeValue::Enum("mentoring".into()))
        .set(6, "offering", tags(&["grant_writing", "data", "maps"]))
        .set(7, "seeking", tags(&["mentoring"]))
        // Hidden-looking noise: offering stored under another attribute.
        .set(5, "skills", tags(&["grant_writing"]));
    let out = f.run(&req);
    assert_eq!(candidates(&out), vec![node(6), node(7), node(5)]);
    let base = 1.0 / 3f64.ln();
    assert!(approx(out[0].score, base + COMPLEMENTARITY_BONUS));
    assert!(approx(out[1].score, base + COMPLEMENTARITY_BONUS));
    assert!(approx(out[2].score, 1.0 / 2f64.ln()));
    assert_eq!(
        out[0].shared_tags.center_seeks,
        vec!["data", "grant_writing"]
    );
    assert!(out[0].shared_tags.candidate_seeks.is_empty());
    assert_eq!(out[1].shared_tags.candidate_seeks, vec!["mentoring"]);
    assert_eq!(
        out[0].explanation,
        "1 mutual relative, 0 willing to introduce; they offer what you seek: data, grant_writing"
    );

    // Both directions still add the bonus once.
    f.set(6, "seeking", tags(&["mentoring"]));
    let out = f.run(&req);
    assert_eq!(out[0].candidate, node(6));
    assert!(approx(out[0].score, base + COMPLEMENTARITY_BONUS));

    // Custom attribute ids are honored.
    let custom = IntroductionRequest {
        seeking_attr: Some(attr("skills")),
        offering_attr: Some(attr("skills")),
        ..IntroductionRequest::new(node(1))
    };
    f.set(1, "skills", tags(&["grant_writing"]));
    let out = f.run(&custom);
    assert_eq!(out[0].candidate, node(5));
    assert_eq!(out[0].shared_tags.center_seeks, vec!["grant_writing"]);
}

#[test]
fn willing_comes_only_from_visible_willing_values() {
    // Introducers 2..=7 all tie the center to candidate 8.
    let pairs: Vec<(u128, u128)> = (2..=7).flat_map(|n| [(1, n), (n, 8)]).collect();
    let mut f = Fixture::new(8, &pairs);
    f.set(
        2,
        "willing_to_introduce",
        AttributeValue::Enum("yes".into()),
    )
    .set(
        3,
        "willing_to_introduce",
        AttributeValue::Text("open".into()),
    )
    .set(4, "willing_to_introduce", AttributeValue::Enum("no".into()))
    // 5 has no visible value (hidden equals absent).
    .set(6, "display_name", AttributeValue::Text("yes".into()))
    .set(7, "willing_to_introduce", tags(&["open", "later"]));
    let willing = |out: &[IntroductionSuggestion]| -> BTreeSet<EntityId> {
        out[0]
            .introducers
            .iter()
            .filter(|i| i.willing)
            .map(|i| i.entity)
            .collect()
    };
    let out = f.run(&IntroductionRequest::new(node(1)));
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].introducers.len(), 6);
    assert_eq!(willing(&out), BTreeSet::from([node(2), node(3), node(7)]));
    assert_eq!(
        out[0].explanation,
        "6 mutual relatives, 3 willing to introduce"
    );

    let custom = IntroductionRequest {
        willing_attr: Some(attr("display_name")),
        willing_values: Some(vec!["yes".into()]),
        ..IntroductionRequest::new(node(1))
    };
    assert_eq!(willing(&f.run(&custom)), BTreeSet::from([node(6)]));
}

#[test]
fn kind_filters_limit_candidates_and_counted_ties() {
    // Center 1 is a member of org 2 and knows person 3. Person 4 is also a
    // member of 2; person 5 and org 6 know 3.
    let mut f = Fixture::new(6, &[(1, 3), (3, 5), (3, 6)]);
    for n in [2, 6] {
        f.entities.get_mut(&n).expect("entity").kind = kind("organization");
    }
    f.tie(1, 2, "member_of").tie(4, 2, "member_of");
    let all = f.run(&IntroductionRequest::new(node(1)));
    assert_eq!(candidates(&all), vec![node(4), node(5), node(6)]);

    let people = IntroductionRequest {
        candidate_kinds: Some(vec![kind("person")]),
        ..IntroductionRequest::new(node(1))
    };
    assert_eq!(candidates(&f.run(&people)), vec![node(4), node(5)]);

    let via_knows = IntroductionRequest {
        via_edge_kinds: Some(vec![kind("knows")]),
        ..people.clone()
    };
    assert_eq!(candidates(&f.run(&via_knows)), vec![node(5)]);

    let via_member = IntroductionRequest {
        via_edge_kinds: Some(vec![kind("member_of")]),
        candidate_kinds: None,
        ..people
    };
    assert_eq!(candidates(&f.run(&via_member)), vec![node(4)]);

    // A direct tie of a kind not counted still means they already know
    // each other.
    f.tie(1, 4, "coauthored");
    assert_eq!(candidates(&f.run(&via_member)), Vec::<EntityId>::new());
}

#[test]
fn results_do_not_depend_on_insertion_order() {
    let mut f = Fixture::new(
        9,
        &[
            (1, 2),
            (1, 3),
            (2, 4),
            (3, 4),
            (2, 5),
            (3, 6),
            (6, 7),
            (2, 8),
            (3, 9),
        ],
    );
    f.set(1, "seeking", tags(&["data"]))
        .set(8, "offering", tags(&["data"]))
        .set(
            3,
            "willing_to_introduce",
            AttributeValue::Enum("yes".into()),
        );
    let req = IntroductionRequest::new(node(1));
    let forward = f.run(&req);
    let p = f.projection();
    let mut reversed = p.clone();
    reversed.entities.reverse();
    reversed.edges.rotate_left(4);
    reversed.edges.reverse();
    let backward = suggest_introductions(&reversed, &GraphIndex::build(&reversed), &req)
        .expect("known center");
    assert_eq!(forward, backward);
    assert_eq!(format!("{forward:?}"), format!("{backward:?}"));
    assert_eq!(forward[0].candidate, node(8), "bonus lifts 8 to the top");
}

#[test]
fn hidden_center_answers_like_absent_and_hidden_entities_change_nothing() {
    // The viewer's projection omits person 9 and their ties entirely.
    let visible = Fixture::new(6, &[(1, 2), (2, 3), (1, 4), (4, 5)]);
    let p = visible.projection();
    let idx = GraphIndex::build(&p);
    let never = suggest_introductions(&p, &idx, &IntroductionRequest::new(node(99)));
    let hidden = suggest_introductions(&p, &idx, &IntroductionRequest::new(node(9)));
    assert_eq!(never, Err(GraphError::NotFound));
    assert_eq!(hidden, never);

    // Were 9 visible (owner's view), it would be an introducer and a
    // candidate, so the fixture is not vacuous.
    let mut owner = Fixture::new(9, &[(1, 2), (2, 3), (1, 4), (4, 5), (1, 9), (9, 6)]);
    owner.entities.remove(&7);
    owner.entities.remove(&8);
    assert_ne!(
        owner.run(&IntroductionRequest::new(node(1))),
        visible.run(&IntroductionRequest::new(node(1)))
    );
}

#[test]
fn limit_defaults_clamps_and_truncates() {
    // Center 1, hub 2, and 60 candidates 3..=62 through the hub.
    let pairs: Vec<(u128, u128)> = std::iter::once((1, 2))
        .chain((3..=62).map(|n| (2, n)))
        .collect();
    let f = Fixture::new(62, &pairs);
    let with = |limit| IntroductionRequest {
        limit,
        ..IntroductionRequest::new(node(1))
    };
    assert_eq!(f.run(&with(None)).len(), DEFAULT_INTRODUCTION_LIMIT);
    assert!(f.run(&with(Some(0))).is_empty());
    assert_eq!(f.run(&with(Some(3))).len(), 3);
    assert_eq!(f.run(&with(Some(50))).len(), MAX_INTRODUCTION_LIMIT);
    let all = f.run(&with(Some(500)));
    assert_eq!(all.len(), MAX_INTRODUCTION_LIMIT);
    // Equal scores fall back to candidate id order.
    let expected: Vec<EntityId> = (3..=52).map(node).collect();
    assert_eq!(candidates(&all), expected);
}
