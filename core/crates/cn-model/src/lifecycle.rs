use serde::{Deserialize, Serialize};

/// Lifecycle state for LWW model records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    /// Active record.
    Active,
    /// Archived record.
    Archived,
}
