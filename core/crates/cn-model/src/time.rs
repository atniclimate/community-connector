use serde::{Deserialize, Serialize};

/// Milliseconds since the Unix epoch, injected by callers - the core never
/// reads a system clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(pub i64);
