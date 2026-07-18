//! Permission engine for viewer contexts, projections, report redaction, and
//! write authorization.

pub mod authz;
pub mod projection;
pub mod reports;
pub mod rules;
pub mod viewer;

pub use authz::PermAuthorizer;
pub use projection::{
    DetailProvenance, EntityDetailMetadata, ProjectedEdge, ProjectedEntity, ProjectedStory,
    ProjectedStoryStep, Projection, entity_detail_metadata, export_projection, project,
};
pub use reports::redact_report;
pub use rules::{effective_tier, required_circle, value_visible};
pub use viewer::{ViewerContext, admissible_circle, viewer_cache_key, viewer_fingerprint};
