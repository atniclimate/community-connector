//! Native-safe facade over the core crates.
//!
//! The public surface follows ADR-003 D1-D4 and A-B5: callers pass JSON
//! strings, receive envelope JSON strings, and never get raw store state.

mod dto;
mod error;
mod session;

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use cn_graph::{GraphError, SearchQuery};
use cn_model::{AttrId, EntityId, GroupId, SensitivityTier, accepts_schema};
use cn_perm::{ProjectedEdge, ProjectedEntity, ProjectedStory, Projection, ViewerContext};
use cn_store::{Operation, StoreReport, SubmitOutcome};
use dto::{
    CoreInfo, DetailValue, EntityDetail, ExportOptions, ExportSnapshot, LoadReport,
    NeighborhoodRequest, PathRequest, SubmitReport,
};
use error::{ApiError, ErrorCode, ErrorEnvelope};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use session::{GroupSession, merge_report, projected_entity_ids};

const BOUNDARY_VERSION: &str = "0.1.0";

/// String-only API facade for ADR-003 D1-D4.
pub struct Api {
    groups: BTreeMap<GroupId, GroupSession>,
}

impl Api {
    /// Creates an empty facade instance (ADR-003 D5).
    pub fn new() -> Self {
        Self {
            groups: BTreeMap::new(),
        }
    }

    /// Returns boundary and core versions (ADR-003 D2).
    pub fn core_info(&self) -> String {
        respond(|| {
            Ok(CoreInfo {
                core_version: cn_model::MODEL_SCHEMA_VERSION,
                boundary_version: BOUNDARY_VERSION,
                supported_schema_majors: vec![0],
            })
        })
    }

    /// Starts streaming group load with viewer-scoped reports (ADR-003 A-B1).
    pub fn load_group_begin(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        template_json: &str,
    ) -> String {
        respond(|| self.load_group_begin_impl(group_id, viewer_ctx_json, template_json))
    }

    /// Adds operation JSONL to a pending load (ADR-003 streaming amendment).
    pub fn load_ops_chunk(&mut self, group_id: &str, ops_jsonl_chunk: &str) -> String {
        respond(|| self.load_ops_chunk_impl(group_id, ops_jsonl_chunk))
    }

    /// Commits a streaming load and returns a redacted report (ADR-003 A-B1).
    pub fn load_group_commit(&mut self, group_id: &str, now_ms: i64) -> String {
        respond(|| self.load_group_commit_impl(group_id, now_ms))
    }

    /// Submits fully formed operations through cn-perm authorization (ADR-003 D1).
    pub fn submit_ops(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        ops_json: &str,
        _now_ms: i64,
    ) -> String {
        respond(|| self.submit_ops_impl(group_id, viewer_ctx_json, ops_json))
    }

    /// Returns the display projection for one viewer (ADR-003 D3).
    pub fn projection(&mut self, group_id: &str, viewer_ctx_json: &str) -> String {
        respond(|| self.projection_impl(group_id, viewer_ctx_json))
    }

    /// Returns projected entity attributes with own-record settings (ADR-003 A-B4).
    pub fn entity_detail(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        entity_id: &str,
    ) -> String {
        respond(|| self.entity_detail_impl(group_id, viewer_ctx_json, entity_id))
    }

    /// Runs a shortest-path query over the viewer projection (ADR-003 D1).
    pub fn query_paths(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        request_json: &str,
    ) -> String {
        respond(|| self.query_paths_impl(group_id, viewer_ctx_json, request_json))
    }

    /// Runs a neighborhood query over the viewer projection (ADR-003 D1).
    pub fn query_neighborhood(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        request_json: &str,
    ) -> String {
        respond(|| self.query_neighborhood_impl(group_id, viewer_ctx_json, request_json))
    }

    /// Searches projected attribute values only (ADR-003 A-B4).
    pub fn search(&mut self, group_id: &str, viewer_ctx_json: &str, query_json: &str) -> String {
        respond(|| self.search_impl(group_id, viewer_ctx_json, query_json))
    }

    /// Returns the viewer-redacted validation report (ADR-003 A-B1).
    pub fn validation_report(&self, group_id: &str, viewer_ctx_json: &str) -> String {
        respond(|| self.validation_report_impl(group_id, viewer_ctx_json))
    }

    /// Exports a narrowing-only snapshot of the viewer projection (ADR-003 A-B2).
    pub fn export_snapshot(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        options_json: &str,
    ) -> String {
        respond(|| self.export_snapshot_impl(group_id, viewer_ctx_json, options_json))
    }

    fn load_group_begin_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        template_json: &str,
    ) -> Result<Value, ApiError> {
        let group_id = parse_group_id(group_id)?;
        if self.groups.contains_key(&group_id) {
            return Err(ApiError::new(
                ErrorCode::GroupExists,
                "group already exists",
            ));
        }
        let viewer = parse_viewer(viewer_ctx_json)?;
        let (template, report) = cn_schema::parse_template(template_json)
            .map_err(|err| ApiError::invalid_json(err.to_string()))?;
        reject_template_report(&report)?;
        self.groups.insert(
            group_id,
            GroupSession::pending(viewer, template_json.to_string(), template),
        );
        Ok(json!({ "accepted": true }))
    }

    fn load_ops_chunk_impl(
        &mut self,
        group_id: &str,
        ops_jsonl_chunk: &str,
    ) -> Result<Value, ApiError> {
        let group_id = parse_group_id_for_lookup(group_id)?;
        let ops = parse_ops_jsonl(ops_jsonl_chunk)?;
        reject_unsupported_ops(&ops)?;
        let count = self
            .groups
            .get_mut(&group_id)
            .ok_or_else(ApiError::not_found)?
            .push_pending_ops(ops)?;
        Ok(json!({ "accepted": count }))
    }

    fn load_group_commit_impl(
        &mut self,
        group_id: &str,
        now_ms: i64,
    ) -> Result<LoadReport, ApiError> {
        let group_id = parse_group_id_for_lookup(group_id)?;
        self.groups
            .get_mut(&group_id)
            .ok_or_else(ApiError::not_found)?
            .commit_pending(group_id, now_ms)
    }

    fn submit_ops_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        ops_json: &str,
    ) -> Result<SubmitReport, ApiError> {
        let group_id = parse_group_id_for_lookup(group_id)?;
        let viewer = parse_viewer(viewer_ctx_json)?;
        let ops: Vec<Operation> = parse_json(ops_json)?;
        reject_unsupported_ops(&ops)?;
        let session = self
            .groups
            .get_mut(&group_id)
            .ok_or_else(ApiError::not_found)?;
        let mut call_report = StoreReport::default();
        let outcomes = cn_store::submit(
            &mut session.state,
            &cn_perm::PermAuthorizer,
            ops,
            &mut call_report,
        );
        if call_report.applied > 0 {
            session.bump_revision();
        }
        let redacted = cn_perm::redact_report(&session.state, &viewer, &call_report);
        merge_report(&mut session.report, call_report);
        Ok(SubmitReport {
            revision: session.revision,
            outcomes: redact_outcomes(outcomes),
            report: redacted,
        })
    }

    fn projection_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
    ) -> Result<Projection, ApiError> {
        let group_id = parse_group_id_for_lookup(group_id)?;
        let viewer = parse_viewer(viewer_ctx_json)?;
        Ok(self
            .groups
            .get_mut(&group_id)
            .ok_or_else(ApiError::not_found)?
            .projection_for(&viewer))
    }

    fn entity_detail_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        entity_id: &str,
    ) -> Result<EntityDetail, ApiError> {
        let group_id = parse_group_id_for_lookup(group_id)?;
        let viewer = parse_viewer(viewer_ctx_json)?;
        let entity_id = parse_entity_id(entity_id)?;
        let session = self
            .groups
            .get_mut(&group_id)
            .ok_or_else(ApiError::not_found)?;
        let projected = projected_entity(session, &viewer, entity_id)?;
        detail_from_projection(session, projected)
    }

    fn query_paths_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        request_json: &str,
    ) -> Result<Option<cn_graph::Path>, ApiError> {
        let request: PathRequest = parse_json(request_json)?;
        let (_projection, index) = self.index_for_request(group_id, viewer_ctx_json)?;
        cn_graph::shortest_path(&index, request.from, request.to, &request.constraints)
            .map_err(graph_error)
    }

    fn query_neighborhood_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        request_json: &str,
    ) -> Result<cn_graph::Neighborhood, ApiError> {
        let request: NeighborhoodRequest = parse_json(request_json)?;
        let (_projection, index) = self.index_for_request(group_id, viewer_ctx_json)?;
        cn_graph::neighborhood(&index, request.center, request.hops, &request.constraints)
            .map_err(graph_error)
    }

    fn search_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        query_json: &str,
    ) -> Result<Vec<cn_graph::SearchHit>, ApiError> {
        let query: SearchQuery = parse_json(query_json)?;
        let (projection, _index) = self.index_for_request(group_id, viewer_ctx_json)?;
        Ok(cn_graph::search(&projection, &query))
    }

    fn validation_report_impl(
        &self,
        group_id: &str,
        viewer_ctx_json: &str,
    ) -> Result<StoreReport, ApiError> {
        let group_id = parse_group_id_for_lookup(group_id)?;
        let viewer = parse_viewer(viewer_ctx_json)?;
        let session = self.groups.get(&group_id).ok_or_else(ApiError::not_found)?;
        Ok(cn_perm::redact_report(
            &session.state,
            &viewer,
            &session.report,
        ))
    }

    fn export_snapshot_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        options_json: &str,
    ) -> Result<ExportSnapshot, ApiError> {
        let group_id = parse_group_id_for_lookup(group_id)?;
        let viewer = parse_viewer(viewer_ctx_json)?;
        let options: ExportOptions = parse_json(options_json)?;
        let session = self
            .groups
            .get_mut(&group_id)
            .ok_or_else(ApiError::not_found)?;
        let projection = export_projection(session, &viewer, &options);
        let template = session
            .state
            .template
            .clone()
            .ok_or_else(|| ApiError::internal("loaded group has no template"))?;
        Ok(ExportSnapshot {
            boundary_version: BOUNDARY_VERSION,
            schema_version: cn_model::model_schema_version(),
            template,
            projection,
        })
    }

    fn index_for_request(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
    ) -> Result<(Projection, cn_graph::GraphIndex), ApiError> {
        let group_id = parse_group_id_for_lookup(group_id)?;
        let viewer = parse_viewer(viewer_ctx_json)?;
        self.groups
            .get_mut(&group_id)
            .ok_or_else(ApiError::not_found)
            .map(|session| session.index_for(&viewer))
    }
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}

fn respond<T, F>(f: F) -> String
where
    T: Serialize,
    F: FnOnce() -> Result<T, ApiError>,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(value)) => serialize_envelope(json!({ "ok": value })),
        Ok(Err(err)) => serialize_envelope(json!({ "err": ErrorEnvelope::from(err) })),
        Err(_) => serialize_envelope(json!({
            "err": ErrorEnvelope::from(ApiError::internal("internal invariant violation"))
        })),
    }
}

fn serialize_envelope(value: Value) -> String {
    match serde_json::to_string(&value) {
        Ok(json) => json,
        Err(err) => {
            let fallback = format!(
                r#"{{"err":{{"code":"internal","message":"{}","details":{{}}}}}}"#,
                json_escape(&err.to_string())
            );
            fallback
        }
    }
}

fn parse_json<T: DeserializeOwned>(json: &str) -> Result<T, ApiError> {
    serde_json::from_str(json).map_err(|err| ApiError::invalid_json(err.to_string()))
}

fn parse_viewer(json: &str) -> Result<ViewerContext, ApiError> {
    serde_json::from_str(json).map_err(|err| ApiError::invalid_viewer(err.to_string()))
}

fn parse_group_id(value: &str) -> Result<GroupId, ApiError> {
    GroupId::from_str(value).map_err(|err| ApiError::invalid_json(err.to_string()))
}

fn parse_group_id_for_lookup(value: &str) -> Result<GroupId, ApiError> {
    GroupId::from_str(value).map_err(|_| ApiError::not_found())
}

fn parse_entity_id(value: &str) -> Result<EntityId, ApiError> {
    EntityId::from_str(value).map_err(|_| ApiError::not_found())
}

fn parse_ops_jsonl(chunk: &str) -> Result<Vec<Operation>, ApiError> {
    let mut ops = Vec::new();
    for (index, line) in chunk.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let op = serde_json::from_str(trimmed).map_err(|err| {
            ApiError::with_details(
                ErrorCode::InvalidJson,
                err.to_string(),
                json!({ "line": index.saturating_add(1) }),
            )
        })?;
        ops.push(op);
    }
    Ok(ops)
}

fn reject_template_report(report: &cn_schema::ValidationReport) -> Result<(), ApiError> {
    if report
        .errors
        .iter()
        .any(|finding| finding.code == cn_schema::FindingCode::UnsupportedSchemaVersion)
    {
        return Err(ApiError::with_details(
            ErrorCode::UnsupportedSchemaVersion,
            "unsupported template schema version",
            json!({ "report": report }),
        ));
    }
    if !report.errors.is_empty() {
        return Err(ApiError::with_details(
            ErrorCode::InvalidJson,
            "template validation failed",
            json!({ "report": report }),
        ));
    }
    Ok(())
}

fn reject_unsupported_ops(ops: &[Operation]) -> Result<(), ApiError> {
    if ops
        .iter()
        .any(|op| !accepts_schema(&op.schema_version) || !accepts_schema(&op.template_version))
    {
        return Err(ApiError::new(
            ErrorCode::UnsupportedSchemaVersion,
            "unsupported operation schema version",
        ));
    }
    Ok(())
}

fn redact_outcomes(outcomes: Vec<SubmitOutcome>) -> Vec<SubmitOutcome> {
    outcomes
}

fn projected_entity(
    session: &mut GroupSession,
    viewer: &ViewerContext,
    entity_id: EntityId,
) -> Result<ProjectedEntity, ApiError> {
    session
        .projection_for(viewer)
        .entities
        .into_iter()
        .find(|entity| entity.id == entity_id)
        .ok_or_else(ApiError::not_found)
}

fn detail_from_projection(
    session: &GroupSession,
    projected: ProjectedEntity,
) -> Result<EntityDetail, ApiError> {
    let mut attributes = BTreeMap::new();
    let raw = session
        .state
        .entities
        .get(&projected.id)
        .ok_or_else(|| ApiError::internal("projected entity missing from state"))?;
    for (attr, value) in projected.attributes {
        let settings = own_settings(projected.owner_is_viewer, raw, &attr)?;
        attributes.insert(
            attr,
            DetailValue {
                value,
                visibility: settings.map(|settings| settings.0),
                tier: settings.map(|settings| settings.1),
            },
        );
    }
    Ok(EntityDetail {
        id: projected.id,
        kind: projected.kind,
        owner_is_viewer: projected.owner_is_viewer,
        attributes,
    })
}

fn own_settings(
    owner_is_viewer: bool,
    raw: &cn_model::Entity,
    attr: &AttrId,
) -> Result<Option<(cn_model::Circle, SensitivityTier)>, ApiError> {
    if !owner_is_viewer {
        return Ok(None);
    }
    let instance = raw
        .attributes
        .get(attr)
        .ok_or_else(|| ApiError::internal("projected attribute missing from state"))?;
    Ok(Some((
        instance.visibility,
        instance.effective_tier(raw.tier),
    )))
}

fn graph_error(err: GraphError) -> ApiError {
    match err {
        GraphError::NotFound => ApiError::not_found(),
    }
}

fn export_projection(
    session: &mut GroupSession,
    viewer: &ViewerContext,
    options: &ExportOptions,
) -> Projection {
    let mut projection = session.projection_for(viewer);
    remove_t3_entities(session, &mut projection);
    apply_kind_filter(&mut projection, options.kinds.as_deref());
    remove_t3_edges(session, &mut projection);
    retain_referenced_content(&mut projection);
    projection
}

fn remove_t3_entities(session: &GroupSession, projection: &mut Projection) {
    projection.entities = projection
        .entities
        .drain(..)
        .filter_map(|entity| export_entity(session, entity))
        .collect();
}

fn export_entity(session: &GroupSession, mut entity: ProjectedEntity) -> Option<ProjectedEntity> {
    let raw = session.state.entities.get(&entity.id)?;
    if raw.tier == SensitivityTier::T3 {
        return None;
    }
    entity
        .attributes
        .retain(|attr, _| export_attr_allowed(raw, attr));
    Some(entity)
}

fn export_attr_allowed(raw: &cn_model::Entity, attr: &AttrId) -> bool {
    raw.attributes
        .get(attr)
        .is_some_and(|instance| instance.effective_tier(raw.tier) != SensitivityTier::T3)
}

fn apply_kind_filter(projection: &mut Projection, kinds: Option<&[cn_model::KindId]>) {
    let Some(kinds) = kinds else {
        return;
    };
    let allowed: BTreeSet<_> = kinds.iter().collect();
    projection
        .entities
        .retain(|entity| allowed.contains(&entity.kind));
}

fn remove_t3_edges(session: &GroupSession, projection: &mut Projection) {
    projection.edges = projection
        .edges
        .drain(..)
        .filter_map(|edge| export_edge(session, edge))
        .collect();
}

fn export_edge(session: &GroupSession, mut edge: ProjectedEdge) -> Option<ProjectedEdge> {
    let raw = session.state.edges.get(&edge.id)?;
    if raw.tier == SensitivityTier::T3 {
        return None;
    }
    edge.attributes
        .retain(|attr, _| export_edge_attr_allowed(raw, attr));
    Some(edge)
}

fn export_edge_attr_allowed(raw: &cn_model::Edge, attr: &AttrId) -> bool {
    raw.attributes
        .get(attr)
        .is_some_and(|instance| instance.effective_tier(raw.tier) != SensitivityTier::T3)
}

fn retain_referenced_content(projection: &mut Projection) {
    let ids = projected_entity_ids(projection);
    projection
        .edges
        .retain(|edge| ids.contains(&edge.from) && ids.contains(&edge.to));
    projection.stories = projection
        .stories
        .drain(..)
        .filter_map(|story| retain_story_refs(story, &ids))
        .collect();
}

fn retain_story_refs(
    mut story: ProjectedStory,
    ids: &BTreeSet<EntityId>,
) -> Option<ProjectedStory> {
    story.steps.retain(|step| ids.contains(&step.entity));
    Some(story)
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| match c {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            other => vec![other],
        })
        .collect()
}
