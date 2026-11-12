//! Record mirror: what each audience receives of a person's own record.
//!
//! The mirror never re-derives the visibility rule. Each audience is reduced
//! to the circle `admissible_circle` resolves for a representative viewer of
//! that audience, and the per-audience flags then come from the same
//! `value_visible` / `required_circle` rule and `effective_tier` that
//! `project` applies, so the mirror cannot drift from the projection.

use std::collections::{BTreeMap, BTreeSet};

use cn_model::{AttrId, Circle, Entity, EntityId, KindId, Lifecycle, PersonId, SensitivityTier};
use cn_store::GroupState;
use serde::Serialize;

use crate::rules::value_visible;

/// An audience other than the owner. Ordered from widest to narrowest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    /// An anonymous viewer.
    Public,
    /// A person who is not the owner, holds no trust grant from the owner,
    /// and is not an active member of this group.
    Network,
    /// An active member who is not the owner and holds no trust grant.
    Group,
    /// A person holding an active trust grant from the owner that covers
    /// this group.
    Trusted,
}

impl Audience {
    /// Every audience, in mirror order.
    pub const ALL: [Audience; 4] = [
        Audience::Public,
        Audience::Network,
        Audience::Group,
        Audience::Trusted,
    ];

    /// The circle `admissible_circle` resolves for a representative viewer
    /// of this audience (spec section 2; ADR-001 D5).
    pub fn circle(self) -> Circle {
        match self {
            Audience::Public => Circle::Public,
            Audience::Network => Circle::Network,
            Audience::Group => Circle::Group,
            Audience::Trusted => Circle::Trusted,
        }
    }
}

/// What each audience receives of the records `owner` owns in this group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecordMirror {
    pub owner: PersonId,
    pub entities: BTreeMap<EntityId, EntityMirror>,
}

/// One active entity the owner owns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntityMirror {
    pub kind: KindId,
    /// Presence of the entity itself; its tier is the entity tier.
    pub presence: VisibilityMirror,
    pub attributes: BTreeMap<AttrId, VisibilityMirror>,
}

/// The owner's chosen circle for one value, the tier that caps it, and what
/// each audience receives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VisibilityMirror {
    /// The circle the owner chose.
    pub visibility: Circle,
    /// Most restrictive of the entity tier and any attribute override.
    pub effective_tier: SensitivityTier,
    /// Widest circle the effective tier allows.
    pub ceiling: Circle,
    /// Whether each audience receives this value. For an attribute this is
    /// false whenever the audience cannot see the entity's presence, because
    /// `project` then omits the whole entity.
    pub audiences: BTreeMap<Audience, bool>,
    /// The chosen circle is wider than the ceiling allows.
    pub tier_capped: bool,
    /// Audiences the owner's chosen circle admits that the ceiling blocks.
    /// This is about the value's own settings and ignores presence.
    pub ceiling_blocks: BTreeSet<Audience>,
}

/// Builds the record mirror for `owner`: every active entity they own in
/// this group, with presence and each attribute resolved per audience.
/// Archived entities and entities owned by others never appear.
pub fn record_mirror(state: &GroupState, owner: PersonId) -> RecordMirror {
    let entities = state
        .entities
        .values()
        .filter(|entity| entity.lifecycle == Lifecycle::Active && entity.owner == Some(owner))
        .map(|entity| (entity.id, entity_mirror(entity)))
        .collect();
    RecordMirror { owner, entities }
}

fn entity_mirror(entity: &Entity) -> EntityMirror {
    let presence = visibility_mirror(entity.presence_visibility, entity.tier, None);
    let attributes = entity
        .attributes
        .iter()
        .map(|(attr, instance)| {
            let tier = instance.effective_tier(entity.tier);
            let mirror = visibility_mirror(instance.visibility, tier, Some(&presence.audiences));
            (attr.clone(), mirror)
        })
        .collect();
    EntityMirror {
        kind: entity.kind.clone(),
        presence,
        attributes,
    }
}

fn visibility_mirror(
    visibility: Circle,
    effective_tier: SensitivityTier,
    presence: Option<&BTreeMap<Audience, bool>>,
) -> VisibilityMirror {
    let ceiling = effective_tier.ceiling();
    let mut audiences = BTreeMap::new();
    let mut ceiling_blocks = BTreeSet::new();
    for audience in Audience::ALL {
        let admitted = value_visible(audience.circle(), visibility, effective_tier);
        if !admitted && audience.circle() <= visibility {
            ceiling_blocks.insert(audience);
        }
        let present = presence.is_none_or(|map| map.get(&audience) == Some(&true));
        audiences.insert(audience, admitted && present);
    }
    VisibilityMirror {
        visibility,
        effective_tier,
        ceiling,
        audiences,
        tier_capped: visibility > ceiling,
        ceiling_blocks,
    }
}
