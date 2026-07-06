//! Core model types for Community Connector.

pub mod attribute;
pub mod circle;
pub mod edge;
pub mod entity;
pub mod error;
pub mod group;
pub mod ids;
pub mod lifecycle;
pub mod membership;
pub mod provenance;
pub mod story;
pub mod tier;
pub mod time;
pub mod trust;

pub use attribute::*;
pub use circle::Circle;
pub use edge::Edge;
pub use entity::Entity;
pub use error::ModelError;
pub use group::{Group, GroupRole};
pub use ids::*;
pub use lifecycle::Lifecycle;
pub use membership::Membership;
pub use provenance::*;
pub use story::{Story, StoryStep};
pub use tier::SensitivityTier;
pub use time::Timestamp;
pub use trust::{TrustGrant, TrustScope};

/// Current persisted model schema version.
pub const MODEL_SCHEMA_VERSION: &str = "0.1.0";

/// Returns the current model schema version.
pub fn model_schema_version() -> semver::Version {
    semver::Version::new(0, 1, 0)
}

/// Returns true when the supplied schema version is accepted by this model.
pub fn accepts_schema(v: &semver::Version) -> bool {
    let current = model_schema_version();
    if current.major == 0 || v.major == 0 {
        v.major == current.major && v.minor == current.minor
    } else {
        v.major == current.major
    }
}
