use std::collections::BTreeMap;

use cn_graph::GraphIndex;
use cn_model::{
    ActorRef, Group, GroupId, OpId, Origin, PersonId, ProvenanceEnvelope, SensitivityTier,
    TemplateId, Timestamp,
};
use cn_perm::{Projection, ViewerContext, project, viewer_cache_key};
use cn_schema::GroupTemplate;
use cn_store::{GroupState, Hlc, Operation, SortKey, StoreReport};

use crate::dto::LoadReport;
use crate::error::{ApiError, ErrorCode};

pub(crate) struct GroupSession {
    pub(crate) state: GroupState,
    pub(crate) revision: u64,
    pub(crate) report: StoreReport,
    pending: Option<PendingLoad>,
    projection_cache: BTreeMap<String, Projection>,
    index_cache: BTreeMap<String, GraphIndex>,
}

pub(crate) struct PendingLoad {
    pub(crate) viewer: ViewerContext,
    pub(crate) template_json: String,
    pub(crate) template: GroupTemplate,
    pub(crate) ops: Vec<Operation>,
}

impl GroupSession {
    pub(crate) fn pending(
        viewer: ViewerContext,
        template_json: String,
        template: GroupTemplate,
    ) -> Self {
        Self {
            state: GroupState::default(),
            revision: 0,
            report: StoreReport::default(),
            pending: Some(PendingLoad {
                viewer,
                template_json,
                template,
                ops: Vec::new(),
            }),
            projection_cache: BTreeMap::new(),
            index_cache: BTreeMap::new(),
        }
    }

    pub(crate) fn push_pending_ops(&mut self, ops: Vec<Operation>) -> Result<usize, ApiError> {
        let Some(pending) = self.pending.as_mut() else {
            return Err(ApiError::new(ErrorCode::LoadNotBegun, "load was not begun"));
        };
        let count = ops.len();
        pending.ops.extend(ops);
        Ok(count)
    }

    pub(crate) fn commit_pending(
        &mut self,
        group_id: GroupId,
        now_ms: i64,
    ) -> Result<LoadReport, ApiError> {
        let Some(pending) = self.pending.take() else {
            return Err(ApiError::new(ErrorCode::LoadNotBegun, "load was not begun"));
        };
        let ops = load_ops_with_group_create(group_id, now_ms, &pending)?;
        let (state, report) = cn_store::fold(ops);
        if state
            .group
            .as_ref()
            .is_none_or(|group| group.id != group_id)
        {
            self.pending = Some(pending);
            return Err(ApiError::new(
                ErrorCode::GroupNotLoaded,
                "group did not load",
            ));
        }
        self.state = state;
        self.revision = 0;
        self.report = report;
        self.clear_caches();
        let redacted = cn_perm::redact_report(&self.state, &pending.viewer, &self.report);
        Ok(LoadReport {
            revision: self.revision,
            report: redacted,
        })
    }

    // Both caches key on `viewer_cache_key` (the canonical authorization
    // context), NOT the hashed `viewer_fingerprint`: the 32-bit fingerprint
    // admits collisions across viewers, and a collision here would serve one
    // viewer another viewer's projection (2026-07-17 adversarial round).
    // The cache key is in-memory only and never serialized.
    pub(crate) fn projection_for(&mut self, viewer: &ViewerContext) -> Projection {
        let key = viewer_cache_key(&self.state, viewer);
        if let Some(projection) = self.projection_cache.get(&key) {
            return projection.clone();
        }
        let projection = project(&self.state, viewer, self.revision);
        self.projection_cache.insert(key, projection.clone());
        projection
    }

    pub(crate) fn index_for(&mut self, viewer: &ViewerContext) -> (Projection, GraphIndex) {
        let key = viewer_cache_key(&self.state, viewer);
        let projection = self.projection_for(viewer);
        if let Some(index) = self.index_cache.get(&key) {
            return (projection, index.clone());
        }
        let index = GraphIndex::build(&projection);
        self.index_cache.insert(key, index.clone());
        (projection, index)
    }

    pub(crate) fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
        self.clear_caches();
    }

    pub(crate) fn clear_caches(&mut self) {
        self.projection_cache.clear();
        self.index_cache.clear();
    }
}

fn load_ops_with_group_create(
    group_id: GroupId,
    now_ms: i64,
    pending: &PendingLoad,
) -> Result<Vec<Operation>, ApiError> {
    if pending.ops.iter().any(is_group_create) {
        return Ok(pending.ops.clone());
    }
    let mut ops = Vec::with_capacity(pending.ops.len().saturating_add(1));
    ops.push(group_create_op(group_id, now_ms, pending)?);
    ops.extend(pending.ops.clone());
    Ok(ops)
}

fn is_group_create(op: &Operation) -> bool {
    matches!(op.kind, cn_store::OpKind::GroupCreate { .. })
}

fn group_create_op(
    group_id: GroupId,
    now_ms: i64,
    pending: &PendingLoad,
) -> Result<Operation, ApiError> {
    let at = Timestamp(now_ms);
    let human = responsible_human(&pending.viewer, at);
    let actor = ActorRef::Human(human);
    let op_id = OpId::new(at);
    let group = group_record(group_id, human, at, pending)?;
    Ok(Operation {
        op_id,
        group_id,
        actor: actor.clone(),
        responsible_human: human,
        recorded_at: at,
        sort_key: SortKey::new(
            Hlc {
                wall_ms: now_ms,
                counter: 0,
            },
            &actor,
            op_id,
        ),
        template_version: pending.template.schema_version.clone(),
        kind: cn_store::OpKind::GroupCreate {
            group,
            template_json: pending.template_json.clone(),
        },
        schema_version: cn_model::model_schema_version(),
    })
}

fn group_record(
    group_id: GroupId,
    human: PersonId,
    at: Timestamp,
    pending: &PendingLoad,
) -> Result<Group, ApiError> {
    let provenance = ProvenanceEnvelope::new(Origin::Authored, ActorRef::Human(human), human, at)
        .map_err(|err| ApiError::internal(err.to_string()))?;
    let template_id: TemplateId = pending.template.template_id.clone();
    Group::new(
        group_id,
        pending.template.name.clone(),
        template_id,
        pending.template.schema_version.clone(),
        provenance,
        SensitivityTier::T1,
    )
    .map_err(|err| ApiError::internal(err.to_string()))
}

fn responsible_human(viewer: &ViewerContext, at: Timestamp) -> PersonId {
    match viewer {
        ViewerContext::Person { person } => *person,
        ViewerContext::Anonymous => PersonId::new(at),
    }
}

pub(crate) fn merge_report(target: &mut StoreReport, next: StoreReport) {
    target.quarantined.extend(next.quarantined);
    target.warnings.extend(next.warnings);
    target.applied = target.applied.saturating_add(next.applied);
    target.deduped = target.deduped.saturating_add(next.deduped);
}
