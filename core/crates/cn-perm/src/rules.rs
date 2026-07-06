use cn_model::{Circle, SensitivityTier};

/// Returns the effective tier for a container and optional value override
/// (spec section 5; ADR-001 D7 and A-B3).
pub fn effective_tier(
    container: SensitivityTier,
    override_: Option<SensitivityTier>,
) -> SensitivityTier {
    let value_tier = match override_ {
        Some(tier) => tier,
        None => container,
    };
    SensitivityTier::most_restrictive(container, value_tier)
}

/// Returns the restrictive circle required by value visibility and tier
/// ceiling (spec section 5; ADR-001 D7 and A-B2).
pub fn required_circle(value_visibility: Circle, eff_tier: SensitivityTier) -> Circle {
    std::cmp::min(value_visibility, eff_tier.ceiling())
}

/// Returns whether a viewer's resolved circle admits the value (spec section 5;
/// ADR-001 D5 and A-B2).
pub fn value_visible(
    admissible: Circle,
    value_visibility: Circle,
    eff_tier: SensitivityTier,
) -> bool {
    admissible <= required_circle(value_visibility, eff_tier)
}
