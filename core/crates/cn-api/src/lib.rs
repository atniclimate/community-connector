//! Native-safe facade over the core crates.
//!
//! The public surface follows ADR-003 D1-D4 and A-B5: callers pass JSON
//! strings, receive envelope JSON strings, and never get raw store state.

mod dto;
mod error;
mod export;
mod session;
mod submit;
mod wire;

use std::collections::BTreeMap;

use cn_graph::{GraphError, SearchQuery};
use cn_model::{AttrId, EntityId, GroupId, SensitivityTier};
use cn_perm::{ProjectedEntity, Projection, ViewerContext};
use cn_store::{Operation, StoreReport};
use dto::{
    CoreInfo, DetailValue, EntityDetail, ExportOptions, ExportSnapshot, LoadReport,
    NeighborhoodRequest, PathRequest, SubmitReport,
};
use error::{ApiError, ErrorCode};
use export::export_projection;
use serde_json::{Value, json};
use session::{GroupSession, merge_report};
use submit::redact_outcomes;
use wire::{
    parse_entity_id, parse_group_id, parse_group_id_for_lookup, parse_json, parse_ops_jsonl,
    parse_viewer, reject_template_report, reject_unsupported_ops, respond,
};

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
        let submitted_ops = ops.clone();
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
        let projection = cn_perm::project(&session.state, &viewer, session.revision);
        merge_report(&mut session.report, call_report);
        Ok(SubmitReport {
            revision: session.revision,
            outcomes: redact_outcomes(outcomes, &submitted_ops, &projection),
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
