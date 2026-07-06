use cn_model::*;
use serde::{Deserialize, Serialize};

/// Hybrid logical clock value. Ord is lexicographic by wall time then counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Hlc {
    pub wall_ms: i64,
    pub counter: u32,
}

/// Caller-fed hybrid logical clock.
#[derive(Debug, Clone)]
pub struct HlcClock {
    last: Hlc,
}

impl HlcClock {
    /// Creates a clock starting before any caller-supplied timestamp.
    pub fn new() -> Self {
        Self {
            last: Hlc {
                wall_ms: i64::MIN,
                counter: 0,
            },
        }
    }

    /// Returns a strictly increasing HLC under stalled or regressing wall time.
    pub fn tick(&mut self, now_ms: i64) -> Hlc {
        if now_ms > self.last.wall_ms {
            self.last = Hlc {
                wall_ms: now_ms,
                counter: 0,
            };
        } else {
            self.last.counter = self.last.counter.saturating_add(1);
        }
        self.last
    }
}

impl Default for HlcClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Canonical total order from ADR-002 D5.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SortKey {
    pub hlc: Hlc,
    pub actor_key: String,
    pub op_id: OpId,
}

impl SortKey {
    /// Builds the ADR-002 D5 sort key for an actor and operation.
    pub fn new(hlc: Hlc, actor: &ActorRef, op_id: OpId) -> Self {
        Self {
            hlc,
            actor_key: actor_key(actor),
            op_id,
        }
    }
}

/// Returns the canonical ADR-002 D5 actor key string.
pub fn actor_key(actor: &ActorRef) -> String {
    match actor {
        ActorRef::Human(id) => format!("human:{id}"),
        ActorRef::Agent { agent_id } => format!("agent:{agent_id}"),
    }
}

/// Event-sourced operation envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    pub op_id: OpId,
    pub group_id: GroupId,
    pub actor: ActorRef,
    pub responsible_human: PersonId,
    pub recorded_at: Timestamp,
    pub sort_key: SortKey,
    pub template_version: semver::Version,
    pub kind: OpKind,
    pub schema_version: semver::Version,
}

/// Store operation payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum OpKind {
    GroupCreate {
        group: Group,
        template_json: String,
    },
    EntityCreate {
        entity: Entity,
    },
    EntityLifecycleSet {
        entity: EntityId,
        lifecycle: Lifecycle,
    },
    AttributeSet {
        entity: EntityId,
        attr: AttrId,
        instance: AttributeInstance,
    },
    AttributeRemove {
        entity: EntityId,
        attr: AttrId,
    },
    VisibilitySet {
        target: VisibilityTarget,
        visibility: Circle,
    },
    TierSet {
        target: TierTarget,
        tier: SensitivityTier,
    },
    EdgeCreate {
        edge: Edge,
    },
    EdgeWeightSet {
        edge: EdgeId,
        weight: Option<f64>,
    },
    EdgeLifecycleSet {
        edge: EdgeId,
        lifecycle: Lifecycle,
    },
    MembershipAdd {
        membership: Membership,
    },
    MembershipLifecycleSet {
        membership: MembershipId,
        lifecycle: Lifecycle,
    },
    TrustGrantCreate {
        grant: TrustGrant,
    },
    TrustGrantRevoke {
        grant: TrustGrantId,
        at: Timestamp,
    },
    CustodyAppend {
        target: CustodyTarget,
        event: CustodyEvent,
    },
    StoryCreate {
        story: Story,
    },
    StoryUpdate {
        story: StoryId,
        title: Option<String>,
        steps: Option<Vec<StoryStep>>,
    },
    StoryLifecycleSet {
        story: StoryId,
        lifecycle: Lifecycle,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisibilityTarget {
    EntityPresence(EntityId),
    Attribute(EntityId, AttrId),
    Edge(EdgeId),
    Story(StoryId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TierTarget {
    Entity(EntityId),
    Attribute(EntityId, AttrId),
    Edge(EdgeId),
    Story(StoryId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyTarget {
    Entity(EntityId),
    Attribute(EntityId, AttrId),
    Edge(EdgeId),
    Story(StoryId),
    Group,
}
