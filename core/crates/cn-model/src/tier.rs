use serde::{Deserialize, Serialize};

use crate::Circle;

/// TSDF-style sensitivity tier ordered by restrictiveness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SensitivityTier {
    /// Least restrictive tier.
    T0,
    /// Network ceiling tier.
    T1,
    /// Group ceiling tier.
    T2,
    /// Most restrictive tier.
    T3,
}

impl SensitivityTier {
    /// Returns the audience ceiling for this sensitivity tier.
    pub fn ceiling(self) -> Circle {
        match self {
            Self::T0 => Circle::Public,
            Self::T1 => Circle::Network,
            Self::T2 => Circle::Group,
            Self::T3 => Circle::Private,
        }
    }

    /// Returns the more restrictive of two sensitivity tiers.
    pub fn most_restrictive(a: Self, b: Self) -> Self {
        std::cmp::max(a, b)
    }
}
