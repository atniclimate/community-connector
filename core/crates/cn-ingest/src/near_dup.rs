//! Near-duplicate candidate surfacing (D-056.4, ADR-005 D4).
//!
//! No submitter auth means the same person can submit twice with different
//! UUIDs, so the review UI surfaces candidates - with their reasons - for
//! the facilitator to approve / reject / merge BY HAND; tooling never
//! auto-merges. The graph side of the comparison is a `cn_perm::Projection`
//! computed for the FACILITATOR viewer, whose attributes are already
//! visibility-filtered: a candidate can never reference anything the
//! facilitator cannot already see (the no-leak property, blueprint
//! section 6). Never score against raw `GroupState`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use cn_model::{AttrId, AttributeValue, EntityId};
use cn_perm::Projection;

/// Where a candidate was found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateSource {
    /// An entity already in the facilitator's projected graph.
    GraphEntity(EntityId),
    /// Another payload waiting in the queue.
    QueueRecord(String),
}

/// One near-duplicate candidate with transparent reasons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NearDupCandidate {
    pub source: CandidateSource,
    /// Human-readable match reasons (always non-empty; D-056.4 transparency).
    pub reasons: Vec<String>,
}

/// Mirrors cn-perm's private D-057 `one_line` rule: split on control
/// characters plus U+2028/U+2029, drop empty fragments, join with single
/// spaces. Characterized by the same cases as cn-perm's tests.
pub fn one_line(text: &str) -> String {
    text.split(|c: char| c.is_control() || c == '\u{2028}' || c == '\u{2029}')
        .filter(|fragment| !fragment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Conservative name normalization for matching: one-line rule, lowercase,
/// whitespace collapsed.
pub fn normalize_name(text: &str) -> String {
    one_line(text)
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn name_tokens(normalized: &str) -> BTreeSet<String> {
    normalized.split(' ').map(|t| t.to_string()).collect()
}

/// Extracts candidate name strings from a payload's `fields` object for
/// the supplied name attribute ids.
fn payload_names(payload: &Value, name_attrs: &[AttrId]) -> Vec<String> {
    let Some(fields) = payload.get("fields").and_then(Value::as_object) else {
        return Vec::new();
    };
    name_attrs
        .iter()
        .filter_map(|attr| fields.get(attr.as_str()))
        .filter_map(Value::as_str)
        .map(normalize_name)
        .filter(|name| !name.is_empty())
        .collect()
}

fn attribute_texts(value: &AttributeValue) -> Vec<String> {
    match value {
        AttributeValue::Text(text) | AttributeValue::Enum(text) => vec![normalize_name(text)],
        AttributeValue::Tags(tags) => tags.iter().map(|t| normalize_name(t)).collect(),
        _ => Vec::new(),
    }
}

/// Conservative match: exact normalized equality, or full containment of
/// the shorter token set in the longer (catches "Jo Doe" vs "Jo A. Doe")
/// when the shorter side has at least two tokens.
fn name_match(a: &str, b: &str) -> Option<String> {
    if a.is_empty() || b.is_empty() {
        return None;
    }
    if a == b {
        return Some(format!("exact normalized name match: \"{a}\""));
    }
    let (ta, tb) = (name_tokens(a), name_tokens(b));
    let (shorter, longer) = if ta.len() <= tb.len() {
        (&ta, &tb)
    } else {
        (&tb, &ta)
    };
    if shorter.len() >= 2 && shorter.is_subset(longer) {
        return Some(format!("name token containment: \"{a}\" ~ \"{b}\""));
    }
    None
}

/// Surfaces near-duplicate candidates for one payload against the
/// facilitator-projected graph and the other queue payloads.
///
/// `name_attrs`: attribute ids that carry names (template-driven, e.g.
/// `display_name`). `affiliation_attrs`: ids whose overlap strengthens a
/// name match with an extra reason (never sufficient alone).
pub fn near_duplicates(
    payload: &Value,
    queue_payloads: &[(String, Value)],
    projection: &Projection,
    name_attrs: &[AttrId],
    affiliation_attrs: &[AttrId],
) -> Vec<NearDupCandidate> {
    let names = payload_names(payload, name_attrs);
    if names.is_empty() {
        return Vec::new();
    }
    let affiliations: BTreeSet<String> = payload
        .get("fields")
        .and_then(Value::as_object)
        .map(|fields| {
            affiliation_attrs
                .iter()
                .filter_map(|attr| fields.get(attr.as_str()))
                .flat_map(|value| match value {
                    Value::String(s) => vec![normalize_name(s)],
                    Value::Array(items) => items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(normalize_name)
                        .collect(),
                    _ => Vec::new(),
                })
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let mut candidates = Vec::new();

    for entity in &projection.entities {
        let mut reasons = Vec::new();
        for attr in name_attrs {
            if let Some(value) = entity.attributes.get(attr) {
                for entity_name in attribute_texts(value) {
                    for name in &names {
                        if let Some(reason) = name_match(name, &entity_name) {
                            reasons.push(reason);
                        }
                    }
                }
            }
        }
        if !reasons.is_empty() && !affiliations.is_empty() {
            for attr in affiliation_attrs {
                if let Some(value) = entity.attributes.get(attr) {
                    for text in attribute_texts(value) {
                        if affiliations.contains(&text) {
                            reasons.push(format!("shared affiliation: \"{text}\""));
                        }
                    }
                }
            }
        }
        if !reasons.is_empty() {
            reasons.dedup();
            candidates.push(NearDupCandidate {
                source: CandidateSource::GraphEntity(entity.id),
                reasons,
            });
        }
    }

    for (record_id, other) in queue_payloads {
        let other_names = payload_names(other, name_attrs);
        let mut reasons = Vec::new();
        for name in &names {
            for other_name in &other_names {
                if let Some(reason) = name_match(name, other_name) {
                    reasons.push(reason);
                }
            }
        }
        if !reasons.is_empty() {
            reasons.dedup();
            candidates.push(NearDupCandidate {
                source: CandidateSource::QueueRecord(record_id.clone()),
                reasons,
            });
        }
    }

    candidates
}
