//! Introduction suggestions: entities two ties from the center, ranked by the
//! mutual relatives who could introduce them (Adamic-Adar) and by whether one
//! side offers what the other seeks. Ties are read as undirected, as
//! `betweenness` does. Attributes are read only from the viewer's
//! `Projection` (the `GraphIndex` drops them), so hidden values never count.

use std::collections::{BTreeMap, BTreeSet};

use cn_model::{AttrId, AttributeValue, EntityId, KindId};
use cn_perm::{ProjectedEntity, Projection};
use serde::{Deserialize, Serialize};

use crate::index::{GraphIndex, all_neighbors, require_entity};
use crate::measures::degrees;
use crate::query::GraphError;

pub const DEFAULT_INTRODUCTION_LIMIT: usize = 10;
/// Larger requested limits are clamped to this.
pub const MAX_INTRODUCTION_LIMIT: usize = 50;

/// Added once to a candidate's score when the center's visible seeking tags
/// meet the candidate's visible offering tags, or the reverse. One mutual
/// relative of degree 2 is worth 1/ln 2 (about 1.44) and of degree 3 about
/// 0.91, so a complementary match lifts a candidate past roughly one extra
/// well-placed mutual relative, but not past several.
pub const COMPLEMENTARITY_BONUS: f64 = 1.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntroductionRequest {
    pub center: EntityId,
    /// Default `DEFAULT_INTRODUCTION_LIMIT`, clamped to `MAX_INTRODUCTION_LIMIT`.
    pub limit: Option<usize>,
    /// Only candidates of these kinds; any kind when `None`.
    pub candidate_kinds: Option<Vec<KindId>>,
    /// Only ties of these kinds count toward common neighbors; any when `None`.
    pub via_edge_kinds: Option<Vec<KindId>>,
    /// Default `seeking`.
    pub seeking_attr: Option<AttrId>,
    /// Default `offering`.
    pub offering_attr: Option<AttrId>,
    /// Default `willing_to_introduce`.
    pub willing_attr: Option<AttrId>,
    /// Default `yes` and `open`.
    pub willing_values: Option<Vec<String>>,
}

impl IntroductionRequest {
    /// A request for `center` with every option at its default.
    pub fn new(center: EntityId) -> Self {
        Self {
            center,
            limit: None,
            candidate_kinds: None,
            via_edge_kinds: None,
            seeking_attr: None,
            offering_attr: None,
            willing_attr: None,
            willing_values: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Introducer {
    pub entity: EntityId,
    /// True only when a visible value of the willing attribute is one of the
    /// willing values.
    pub willing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SharedTags {
    /// Tags the center seeks and the candidate offers.
    pub center_seeks: Vec<String>,
    /// Tags the candidate seeks and the center offers.
    pub candidate_seeks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntroductionSuggestion {
    pub candidate: EntityId,
    pub score: f64,
    /// Common neighbors, in id order.
    pub introducers: Vec<Introducer>,
    pub shared_tags: SharedTags,
    pub explanation: String,
}

/// Suggests introductions for `req.center`, best first (score descending,
/// then candidate id). A hidden or unknown center is `NotFound`.
pub fn suggest_introductions(
    p: &Projection,
    idx: &GraphIndex,
    req: &IntroductionRequest,
) -> Result<Vec<IntroductionSuggestion>, GraphError> {
    require_entity(idx, req.center)?;
    let limit = req
        .limit
        .unwrap_or(DEFAULT_INTRODUCTION_LIMIT)
        .min(MAX_INTRODUCTION_LIMIT);
    let entities: BTreeMap<EntityId, &ProjectedEntity> =
        p.entities.iter().map(|e| (e.id, e)).collect();
    let via = req.via_edge_kinds.as_deref();
    let near = ties(idx, req.center, via);
    let adjacent = ties(idx, req.center, None);
    let mut common: BTreeMap<EntityId, BTreeSet<EntityId>> = BTreeMap::new();
    for &n in &near {
        for c in ties(idx, n, via) {
            let kind_ok = req
                .candidate_kinds
                .as_ref()
                .is_none_or(|kinds| entities.get(&c).is_some_and(|e| kinds.contains(&e.kind)));
            if c != req.center && !adjacent.contains(&c) && kind_ok {
                common.entry(c).or_default().insert(n);
            }
        }
    }
    let ctx = Scoring::new(req, &entities, idx);
    let mut out: Vec<_> = common
        .into_iter()
        .map(|(candidate, via)| ctx.suggestion(candidate, &via))
        .collect();
    out.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.candidate.cmp(&b.candidate))
    });
    out.truncate(limit);
    Ok(out)
}

/// Distinct visible neighbors of `node` over ties of `kinds`, excluding
/// self-loops.
fn ties(idx: &GraphIndex, node: EntityId, kinds: Option<&[KindId]>) -> BTreeSet<EntityId> {
    all_neighbors(idx, node)
        .filter(|adj| kinds.is_none_or(|k| k.contains(&adj.kind)))
        .map(|adj| adj.to)
        .filter(|&to| to != node && idx.entities.contains(&to))
        .collect()
}

struct Scoring<'a> {
    center: EntityId,
    entities: &'a BTreeMap<EntityId, &'a ProjectedEntity>,
    degrees: BTreeMap<EntityId, usize>,
    seeking: AttrId,
    offering: AttrId,
    willing: AttrId,
    willing_values: Vec<String>,
}

impl<'a> Scoring<'a> {
    fn new(
        req: &IntroductionRequest,
        entities: &'a BTreeMap<EntityId, &'a ProjectedEntity>,
        idx: &GraphIndex,
    ) -> Self {
        let pick = |given: &Option<AttrId>, default: &str| {
            given
                .clone()
                .unwrap_or_else(|| AttrId::new(default).expect("default attribute id is a slug"))
        };
        Self {
            center: req.center,
            entities,
            degrees: degrees(idx),
            seeking: pick(&req.seeking_attr, "seeking"),
            offering: pick(&req.offering_attr, "offering"),
            willing: pick(&req.willing_attr, "willing_to_introduce"),
            willing_values: req
                .willing_values
                .clone()
                .unwrap_or_else(|| vec!["yes".to_string(), "open".to_string()]),
        }
    }

    fn suggestion(&self, candidate: EntityId, via: &BTreeSet<EntityId>) -> IntroductionSuggestion {
        let mut score: f64 = via
            .iter()
            .map(|n| {
                // A common neighbor has at least two ties; max(2) keeps
                // ln(1) = 0 from ever dividing.
                let degree = self.degrees.get(n).copied().unwrap_or(0).max(2);
                1.0 / (degree as f64).ln()
            })
            .sum();
        let shared_tags = SharedTags {
            center_seeks: self.overlap(self.center, &self.seeking, candidate, &self.offering),
            candidate_seeks: self.overlap(candidate, &self.seeking, self.center, &self.offering),
        };
        if !shared_tags.center_seeks.is_empty() || !shared_tags.candidate_seeks.is_empty() {
            score += COMPLEMENTARITY_BONUS;
        }
        let introducers: Vec<Introducer> = via
            .iter()
            .map(|&entity| Introducer {
                entity,
                willing: self
                    .values(entity, &self.willing)
                    .iter()
                    .any(|v| self.willing_values.iter().any(|w| w == v)),
            })
            .collect();
        let explanation = explain(&introducers, &shared_tags);
        IntroductionSuggestion {
            candidate,
            score,
            introducers,
            shared_tags,
            explanation,
        }
    }

    fn overlap(&self, a: EntityId, a_attr: &AttrId, b: EntityId, b_attr: &AttrId) -> Vec<String> {
        let theirs = self.values(b, b_attr);
        self.values(a, a_attr)
            .intersection(&theirs)
            .map(|tag| (*tag).to_string())
            .collect()
    }

    /// Visible string values of `attr` on `entity`: each tag, or the single
    /// text or enum value.
    fn values(&self, entity: EntityId, attr: &AttrId) -> BTreeSet<&'a str> {
        match self
            .entities
            .get(&entity)
            .and_then(|e| e.attributes.get(attr))
        {
            Some(AttributeValue::Tags(tags)) => tags.iter().map(String::as_str).collect(),
            Some(AttributeValue::Text(v) | AttributeValue::Enum(v)) => BTreeSet::from([v.as_str()]),
            _ => BTreeSet::new(),
        }
    }
}

fn explain(introducers: &[Introducer], tags: &SharedTags) -> String {
    let n = introducers.len();
    let willing = introducers.iter().filter(|i| i.willing).count();
    let mut text = format!(
        "{n} mutual relative{}, {willing} willing to introduce",
        if n == 1 { "" } else { "s" }
    );
    if !tags.center_seeks.is_empty() {
        text.push_str(&format!(
            "; they offer what you seek: {}",
            tags.center_seeks.join(", ")
        ));
    }
    if !tags.candidate_seeks.is_empty() {
        text.push_str(&format!(
            "; they seek what you offer: {}",
            tags.candidate_seeks.join(", ")
        ));
    }
    text
}
