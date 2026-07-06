//! Permission engine for viewer contexts, projections, report redaction, and
//! write authorization.

pub mod authz;
pub mod projection;
pub mod reports;
pub mod rules;
pub mod viewer;

pub use authz::PermAuthorizer;
pub use projection::{
    ProjectedEdge, ProjectedEntity, ProjectedStory, ProjectedStoryStep, Projection, project,
};
pub use reports::redact_report;
pub use rules::{effective_tier, required_circle, value_visible};
pub use viewer::{ViewerContext, admissible_circle, viewer_fingerprint};
