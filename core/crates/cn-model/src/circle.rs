use serde::{Deserialize, Serialize};

/// Audience lattice ordered by audience size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Circle {
    /// Visible only to the owner.
    Private,
    /// Visible to trusted people.
    Trusted,
    /// Visible to members of a group.
    Group,
    /// Visible to the network.
    Network,
    /// Visible publicly.
    Public,
}
