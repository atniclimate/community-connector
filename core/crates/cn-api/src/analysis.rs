//! Person-centered analysis calls: continuance, introductions, and the
//! record mirror. Whose results a viewer may see is decided by
//! `cn_perm::analysis_scope`, and whether an entity is the viewer's own comes
//! from the projection's `owner_is_viewer`; this module only applies both
//! (I2). Split from lib.rs to keep that module near the I5 size budget.

use std::collections::BTreeSet;

use cn_graph::{Continuance, IntroductionRequest, IntroductionSuggestion};
use cn_model::EntityId;
use cn_perm::{AnalysisScope, Projection, ViewerContext};
use serde_json::{Value, json};

use crate::Api;
use crate::dto::ContinuanceRequest;
use crate::error::ApiError;
use crate::graph_error;
use crate::wire::{parse_group_id_for_lookup, parse_json, parse_viewer};

impl Api {
    pub(crate) fn graph_continuance_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        request_json: &str,
    ) -> Result<Continuance, ApiError> {
        let _request: ContinuanceRequest = parse_json(request_json)?;
        let (scope, projection, index) = self.scoped_index(group_id, viewer_ctx_json)?;
        let mut out = cn_graph::continuance(&index);
        match scope {
            AnalysisScope::All => {}
            AnalysisScope::OwnRecords(_) => {
                let own = own_entities(&projection);
                out.articulation_points.retain(|id, _| own.contains(id));
                out.bridges
                    .retain(|_, tie| own.contains(&tie.from) || own.contains(&tie.to));
                // Component keys name the smallest member id, often someone
                // else's, so a "what rests on me" view leaves them out.
                out.components.clear();
            }
            AnalysisScope::Nothing => {
                out.articulation_points.clear();
                out.bridges.clear();
                out.components.clear();
            }
        }
        Ok(out)
    }

    pub(crate) fn suggest_introductions_impl(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
        request_json: &str,
    ) -> Result<Vec<IntroductionSuggestion>, ApiError> {
        let request: IntroductionRequest = parse_json(request_json)?;
        let (scope, projection, index) = self.scoped_index(group_id, viewer_ctx_json)?;
        // Not owning the center answers exactly like a hidden or missing
        // center (ADR-003 A-B4).
        if scope != AnalysisScope::All && !own_entities(&projection).contains(&request.center) {
            return Err(ApiError::not_found());
        }
        cn_graph::suggest_introductions(&projection, &index, &request).map_err(graph_error)
    }

    pub(crate) fn record_mirror_impl(
        &self,
        group_id: &str,
        viewer_ctx_json: &str,
    ) -> Result<Value, ApiError> {
        let group_id = parse_group_id_for_lookup(group_id)?;
        let viewer = parse_viewer(viewer_ctx_json)?;
        let session = self.groups.get(&group_id).ok_or_else(ApiError::not_found)?;
        match viewer {
            ViewerContext::Anonymous => Ok(json!({ "owner": null, "entities": {} })),
            ViewerContext::Person { person } => {
                serde_json::to_value(cn_perm::record_mirror(&session.state, person))
                    .map_err(|err| ApiError::internal(err.to_string()))
            }
        }
    }

    fn scoped_index(
        &mut self,
        group_id: &str,
        viewer_ctx_json: &str,
    ) -> Result<(AnalysisScope, Projection, cn_graph::GraphIndex), ApiError> {
        let group_id = parse_group_id_for_lookup(group_id)?;
        let viewer = parse_viewer(viewer_ctx_json)?;
        let session = self
            .groups
            .get_mut(&group_id)
            .ok_or_else(ApiError::not_found)?;
        let scope = cn_perm::analysis_scope(&session.state, &viewer);
        let (projection, index) = session.index_for(&viewer);
        Ok((scope, projection, index))
    }
}

fn own_entities(projection: &Projection) -> BTreeSet<EntityId> {
    projection
        .entities
        .iter()
        .filter(|entity| entity.owner_is_viewer)
        .map(|entity| entity.id)
        .collect()
}
