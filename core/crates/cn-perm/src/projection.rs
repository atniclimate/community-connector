use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use cn_model::{
    ActorRef, AttrId, AttributeInstance, AttributeValue, Circle, CustodyEvent, Edge, EdgeId,
    Entity, EntityId, GroupId, KindId, Lifecycle, Origin, SensitivityTier, Story, StoryId,
};
use cn_store::GroupState;
use serde::Serialize;

use crate::rules::value_visible;
use crate::viewer::{ViewerContext, admissible_circle, viewer_fingerprint};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Projection {
    pub group_id: GroupId,
    pub viewer_fingerprint: String,
    pub revision: u64,
    pub entities: Vec<ProjectedEntity>,
    pub edges: Vec<ProjectedEdge>,
    pub stories: Vec<ProjectedStory>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectedEntity {
    pub id: EntityId,
    pub kind: KindId,
    pub owner_is_viewer: bool,
    pub attributes: BTreeMap<AttrId, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectedEdge {
    pub id: EdgeId,
    pub kind: KindId,
    pub from: EntityId,
    pub to: EntityId,
    pub directed: bool,
    pub weight: Option<f64>,
    pub attributes: BTreeMap<AttrId, AttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectedStory {
    pub id: StoryId,
    pub title: String,
    pub steps: Vec<ProjectedStoryStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectedStoryStep {
    pub entity: EntityId,
    pub narration: String,
}

/// Permission-shaped metadata for an entity detail response (D-032/D-033).
/// `tier` is the effective tier of what this viewer actually sees: the
/// maximum of the entity tier and the effective tiers of exactly the
/// projected attribute set. Computing over raw attributes instead would
/// either under-report (entity tier alone) or reveal the existence of
/// hidden higher-tier attributes.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EntityDetailMetadata {
    pub tier: SensitivityTier,
    pub provenance: DetailProvenance,
    pub attributes: BTreeMap<AttrId, DetailAttribute>,
}

/// One attribute of the viewer's projected detail: the value plus the
/// owner-only disclosure of its visibility and effective tier. The
/// disclosure decision is made here, at the permission boundary (I2);
/// carriers serialize these fields without reshaping them.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DetailAttribute {
    pub value: AttributeValue,
    pub visibility: Option<Circle>,
    pub tier: Option<SensitivityTier>,
}

/// The one-line provenance summary visible to every projected viewer, with
/// the full custody chain present only for governance viewers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DetailProvenance {
    pub line: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custody: Option<Vec<CustodyEvent>>,
}

/// Supplies the complete entity-detail projection at the permission
/// boundary: effective tier over the projected attribute set, the
/// owner-only attribute settings disclosure, and provenance with
/// governance-only custody depth (D-032/D-033; I2).
///
/// Deliberate cost note (2026-07-24 adversarial round): this re-applies the
/// projection predicate over the raw attribute map and clones each visible
/// value, an O(attributes) pass per single-entity detail request. Accepted:
/// detail is a one-entity interaction path, not a bulk path, and recomputing
/// from raw state under the same mutable API call keeps this function the
/// sole authority instead of trusting a caller-supplied projected set.
pub fn entity_detail_metadata(
    state: &GroupState,
    viewer: &ViewerContext,
    entity: &Entity,
) -> EntityDetailMetadata {
    let provenance = &entity.provenance;
    let custody = match viewer {
        ViewerContext::Person { person } if crate::viewer::is_governance(state, *person) => {
            Some(provenance.custody().to_vec())
        }
        _ => None,
    };
    let disclose_settings = owner_is_viewer(viewer, &entity.owner);
    let mut tier = entity.tier;
    let mut attributes = BTreeMap::new();
    for (attr, instance) in &entity.attributes {
        if !attr_visible(state, viewer, entity.owner.as_ref(), entity.tier, instance) {
            continue;
        }
        let effective = instance.effective_tier(entity.tier);
        tier = SensitivityTier::most_restrictive(tier, effective);
        attributes.insert(
            attr.clone(),
            DetailAttribute {
                value: instance.value.clone(),
                visibility: disclose_settings.then_some(instance.visibility),
                tier: disclose_settings.then_some(effective),
            },
        );
    }
    EntityDetailMetadata {
        tier,
        provenance: DetailProvenance {
            line: one_line(&format!(
                "Added by {} from {} (timestamp {})",
                actor_summary(provenance.recorded_by()),
                origin_summary(provenance.origin()),
                provenance.recorded_at().0,
            )),
            custody,
        },
        attributes,
    }
}

/// Collapses control characters and Unicode line/paragraph separators
/// (U+2028/U+2029) so the D-033 summary is structurally a single line even
/// when an agent id or ingested source carries line-breaking text.
fn one_line(text: &str) -> String {
    text.split(|c: char| c.is_control() || c == '\u{2028}' || c == '\u{2029}')
        .filter(|fragment| !fragment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn actor_summary(actor: &ActorRef) -> String {
    match actor {
        ActorRef::Human(person) => format!("person {person}"),
        ActorRef::Agent { agent_id } => format!("agent {agent_id}"),
    }
}

fn origin_summary(origin: &Origin) -> String {
    match origin {
        Origin::SelfReported => "self-reported".to_string(),
        Origin::Ingested { source } => source.to_string(),
        Origin::Derived { inputs } => format!("{} operation inputs", inputs.len()),
        Origin::Authored => "authored input".to_string(),
    }
}

/// Builds the viewer projection (spec sections 5 and 6; ADR-001 A-B1;
/// ADR-003 D1 and round-2 entity detail amendment).
pub fn project(state: &GroupState, viewer: &ViewerContext, revision: u64) -> Projection {
    let entities = project_entities(state, viewer);
    let entity_ids = entities.iter().map(|entity| entity.id).collect();
    Projection {
        group_id: projection_group_id(state),
        viewer_fingerprint: viewer_fingerprint(state, viewer),
        revision,
        edges: project_edges(state, viewer, &entity_ids),
        stories: project_stories(state, viewer, &entity_ids),
        entities,
    }
}

/// Builds an export projection. This applies the normal viewer projection and
/// the export gate from ADR-001 A-B2 / ADR-002 A-B8: values whose effective
/// tier is T3 are excluded even when the viewer is the owner.
pub fn export_projection(state: &GroupState, viewer: &ViewerContext, revision: u64) -> Projection {
    let mut projection = project(state, viewer, revision);
    remove_export_t3_entities(state, &mut projection);
    remove_export_t3_edges(state, &mut projection);
    retain_referenced_content(&mut projection);
    projection
}

/// Projects active entities only (spec section 5 object rules; blueprint
/// projection rule 1).
fn project_entities(state: &GroupState, viewer: &ViewerContext) -> Vec<ProjectedEntity> {
    state
        .entities
        .values()
        .filter(|entity| entity.lifecycle == Lifecycle::Active)
        .filter(|entity| entity_visible(state, viewer, entity))
        .map(|entity| project_entity(state, viewer, entity))
        .collect()
}

/// Projects entity presence when the presence rule passes (spec section 5;
/// blueprint projection rule 2).
pub fn entity_visible(state: &GroupState, viewer: &ViewerContext, entity: &Entity) -> bool {
    visible_for_owner(
        state,
        viewer,
        entity.owner.as_ref(),
        entity.presence_visibility,
        entity.tier,
    )
}

fn project_entity(state: &GroupState, viewer: &ViewerContext, entity: &Entity) -> ProjectedEntity {
    ProjectedEntity {
        id: entity.id,
        kind: entity.kind.clone(),
        owner_is_viewer: owner_is_viewer(viewer, &entity.owner),
        attributes: visible_entity_attributes(state, viewer, entity),
    }
}

/// Includes only attributes whose own visibility and effective tier pass
/// (spec section 5; ADR-001 A-B3; blueprint projection rule 3).
pub fn visible_entity_attributes(
    state: &GroupState,
    viewer: &ViewerContext,
    entity: &Entity,
) -> BTreeMap<AttrId, AttributeValue> {
    entity
        .attributes
        .iter()
        .filter(|(_, instance)| {
            attr_visible(state, viewer, entity.owner.as_ref(), entity.tier, instance)
        })
        .map(|(attr, instance)| (attr.clone(), instance.value.clone()))
        .collect()
}

/// Projects edges only when both endpoints are projected and the edge's own
/// rule passes (spec section 5; ADR-001 D6; blueprint projection rules 4 and
/// 6).
fn project_edges(
    state: &GroupState,
    viewer: &ViewerContext,
    entity_ids: &BTreeSet<EntityId>,
) -> Vec<ProjectedEdge> {
    state
        .edges
        .values()
        .filter(|edge| edge.lifecycle == Lifecycle::Active)
        .filter(|edge| entity_ids.contains(&edge.from) && entity_ids.contains(&edge.to))
        .filter(|edge| edge_visible(state, viewer, edge))
        .map(|edge| project_edge(state, viewer, edge))
        .collect()
}

/// Evaluates the edge's own circle and tier (spec section 5; ADR-001 D6).
pub fn edge_visible(state: &GroupState, viewer: &ViewerContext, edge: &Edge) -> bool {
    visible_for_owner(state, viewer, None, edge.visibility, edge.tier)
}

fn project_edge(state: &GroupState, viewer: &ViewerContext, edge: &Edge) -> ProjectedEdge {
    ProjectedEdge {
        id: edge.id,
        kind: edge.kind.clone(),
        from: edge.from,
        to: edge.to,
        directed: edge.directed,
        weight: edge.weight,
        attributes: visible_edge_attributes(state, viewer, edge),
    }
}

/// Filters edge attributes like entity attributes with edge tier as container
/// (spec section 5; ADR-001 D6; blueprint projection rule 4).
pub fn visible_edge_attributes(
    state: &GroupState,
    viewer: &ViewerContext,
    edge: &Edge,
) -> BTreeMap<AttrId, AttributeValue> {
    edge.attributes
        .iter()
        .filter(|(_, instance)| attr_visible(state, viewer, None, edge.tier, instance))
        .map(|(attr, instance)| (attr.clone(), instance.value.clone()))
        .collect()
}

/// Projects stories through their own rule and silently elides invisible steps
/// (spec section 5; ADR-001 A-B4; blueprint projection rules 5 and 6).
fn project_stories(
    state: &GroupState,
    viewer: &ViewerContext,
    entity_ids: &BTreeSet<EntityId>,
) -> Vec<ProjectedStory> {
    state
        .stories
        .values()
        .filter(|story| story.lifecycle == Lifecycle::Active)
        .filter(|story| story_visible(state, viewer, story))
        .map(|story| project_story(story, entity_ids))
        .collect()
}

/// Evaluates a story's whole-story circle and tier (spec section 5; ADR-001
/// A-B4).
pub fn story_visible(state: &GroupState, viewer: &ViewerContext, story: &Story) -> bool {
    visible_for_owner(state, viewer, None, story.visibility, story.tier)
}

fn project_story(story: &Story, entity_ids: &BTreeSet<EntityId>) -> ProjectedStory {
    let steps = story
        .steps
        .iter()
        .filter(|step| entity_ids.contains(&step.entity))
        .map(|step| ProjectedStoryStep {
            entity: step.entity,
            narration: step.narration.clone(),
        })
        .collect();
    ProjectedStory {
        id: story.id,
        title: story.title.clone(),
        steps,
    }
}

/// Projection omits provenance envelopes and tiers by construction (blueprint
/// projection rule 7; ADR-003 round-2 entity detail amendment).
pub fn attr_visible(
    state: &GroupState,
    viewer: &ViewerContext,
    owner: Option<&cn_model::PersonId>,
    container_tier: SensitivityTier,
    instance: &AttributeInstance,
) -> bool {
    visible_for_owner(
        state,
        viewer,
        owner,
        instance.visibility,
        instance.effective_tier(container_tier),
    )
}

pub fn visible_for_owner(
    state: &GroupState,
    viewer: &ViewerContext,
    owner: Option<&cn_model::PersonId>,
    visibility: Circle,
    tier: SensitivityTier,
) -> bool {
    value_visible(admissible_circle(state, viewer, owner), visibility, tier)
}

fn owner_is_viewer(viewer: &ViewerContext, owner: &Option<cn_model::PersonId>) -> bool {
    match (viewer, owner) {
        (ViewerContext::Person { person }, Some(owner)) => person == owner,
        _ => false,
    }
}

fn projection_group_id(state: &GroupState) -> GroupId {
    if let Some(group) = &state.group {
        return group.id;
    }
    match GroupId::from_str("00000000-0000-0000-0000-000000000000") {
        Ok(id) => id,
        Err(_) => GroupId::new(cn_model::Timestamp(0)),
    }
}

fn remove_export_t3_entities(state: &GroupState, projection: &mut Projection) {
    projection.entities = projection
        .entities
        .drain(..)
        .filter_map(|entity| export_entity(state, entity))
        .collect();
}

fn export_entity(state: &GroupState, mut entity: ProjectedEntity) -> Option<ProjectedEntity> {
    let raw = state.entities.get(&entity.id)?;
    if raw.tier == SensitivityTier::T3 {
        return None;
    }
    entity.attributes.retain(|attr, _| {
        raw.attributes
            .get(attr)
            .is_some_and(|instance| instance.effective_tier(raw.tier) != SensitivityTier::T3)
    });
    Some(entity)
}

fn remove_export_t3_edges(state: &GroupState, projection: &mut Projection) {
    projection.edges = projection
        .edges
        .drain(..)
        .filter_map(|edge| export_edge(state, edge))
        .collect();
}

fn export_edge(state: &GroupState, mut edge: ProjectedEdge) -> Option<ProjectedEdge> {
    let raw = state.edges.get(&edge.id)?;
    if raw.tier == SensitivityTier::T3 {
        return None;
    }
    edge.attributes.retain(|attr, _| {
        raw.attributes
            .get(attr)
            .is_some_and(|instance| instance.effective_tier(raw.tier) != SensitivityTier::T3)
    });
    Some(edge)
}

fn retain_referenced_content(projection: &mut Projection) {
    let ids: BTreeSet<_> = projection.entities.iter().map(|entity| entity.id).collect();
    projection
        .edges
        .retain(|edge| ids.contains(&edge.from) && ids.contains(&edge.to));
    for story in &mut projection.stories {
        story.steps.retain(|step| ids.contains(&step.entity));
    }
}
