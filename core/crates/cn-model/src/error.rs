use thiserror::Error;

use crate::SensitivityTier;

/// Error type for all fallible model validation paths.
#[derive(Debug, Error, PartialEq)]
pub enum ModelError {
    /// Invalid UUID-backed identifier.
    #[error("invalid id: {0}")]
    InvalidId(String),
    /// Invalid slug-backed identifier.
    #[error("invalid slug '{value}' for {what}")]
    InvalidSlug { what: &'static str, value: String },
    /// Invalid ISO date value.
    #[error("invalid date: {0}")]
    InvalidDate(String),
    /// Invalid URL value.
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    /// Invalid geographic coordinates.
    #[error("invalid geo coordinates: lat {lat}, lon {lon}")]
    InvalidGeo { lat: f64, lon: f64 },
    /// Non-finite edge weight.
    #[error("non-finite weight")]
    NonFiniteWeight,
    /// Human actor accountability mismatch.
    #[error("responsible human mismatch for human actor")]
    ResponsibleHumanMismatch,
    /// Tier override attempted to loosen or keep the current tier.
    #[error(
        "tier override may only tighten (current effective {current:?}, attempted {attempted:?})"
    )]
    TierLoosenAttempt {
        /// Current effective tier.
        current: SensitivityTier,
        /// Attempted replacement tier.
        attempted: SensitivityTier,
    },
    /// Trust grant was already revoked.
    #[error("trust grant already revoked")]
    AlreadyRevoked,
    /// Revocation timestamp predates the grant.
    #[error("revocation predates grant")]
    RevokedBeforeGranted,
    /// Name must not be empty.
    #[error("empty name")]
    EmptyName,
    /// Unsupported persisted schema version.
    #[error("unsupported schema version {0}")]
    UnsupportedSchemaVersion(String),
}
