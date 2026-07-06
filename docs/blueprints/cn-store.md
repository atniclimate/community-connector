# Blueprint: cn-store (Phase 2, authored by director from ADR-002 + amendments)

The event-sourced heart. Pure fold logic MUST be wasm-safe (no fs, no clock);
file persistence is native-only behind `#[cfg(not(target_arch = "wasm32"))]`.

## Prerequisite: add Story to cn-model first

cn-model lacks Story (deferred from its blueprint). Add to cn-model,
following all its conventions (constructor requires provenance + tier, I6;
serde; tests):

```rust
pub struct Story {
    pub id: StoryId, pub group_id: GroupId, pub title: String,
    pub steps: Vec<StoryStep>,
    pub visibility: Circle, pub lifecycle: Lifecycle,
    pub provenance: ProvenanceEnvelope, pub tier: SensitivityTier,
    pub schema_version: semver::Version,
}
pub struct StoryStep { pub entity: EntityId, pub narration: String }
```

## Dependencies (cn-store)

```toml
[dependencies]
cn-model = { path = "../cn-model" }
cn-schema = { path = "../cn-schema" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
semver = { version = "1", features = ["serde"] }
thiserror = "2"
crc32fast = "1"
```

## Module: op

```rust
/// Hybrid logical clock value. Ord: (wall_ms, counter) lexicographic.
pub struct Hlc { pub wall_ms: i64, pub counter: u32 }

/// Caller-fed clock: tick(now_ms) returns a strictly increasing Hlc even if
/// wall time stalls or regresses (never goes backward; counter bumps).
pub struct HlcClock { /* last: Hlc */ }
impl HlcClock { pub fn new() -> Self; pub fn tick(&mut self, now_ms: i64) -> Hlc; }

/// Canonical total order (ADR-002 D5): (hlc, actor_key, op_id).
/// actor_key is the canonical string of ActorRef: "human:<uuid>" or
/// "agent:<agent_id>". Derive Ord exactly in that field order.
pub struct SortKey { pub hlc: Hlc, pub actor_key: String, pub op_id: OpId }

pub struct Operation {
    pub op_id: OpId,
    pub group_id: GroupId,
    pub actor: ActorRef,
    pub responsible_human: PersonId,
    pub recorded_at: Timestamp,          // display/audit only - never ordering
    pub sort_key: SortKey,               // ADR-002 round-2: split from recorded_at
    pub template_version: semver::Version,  // ADR-002 A-B3
    pub kind: OpKind,
    pub schema_version: semver::Version,
}
```

`OpKind` (serde tag = "op"), payloads use cn-model types:

```rust
pub enum OpKind {
    GroupCreate { group: Group, template_json: String },
    EntityCreate { entity: Entity },
    EntityLifecycleSet { entity: EntityId, lifecycle: Lifecycle },
    AttributeSet { entity: EntityId, attr: AttrId, instance: AttributeInstance },
    AttributeRemove { entity: EntityId, attr: AttrId },
    VisibilitySet { target: VisibilityTarget, visibility: Circle },
    TierSet { target: TierTarget, tier: SensitivityTier },
    EdgeCreate { edge: Edge },
    EdgeWeightSet { edge: EdgeId, weight: Option<f64> },
    EdgeLifecycleSet { edge: EdgeId, lifecycle: Lifecycle },
    MembershipAdd { membership: Membership },
    MembershipLifecycleSet { membership: MembershipId, lifecycle: Lifecycle },
    TrustGrantCreate { grant: TrustGrant },
    TrustGrantRevoke { grant: TrustGrantId, at: Timestamp },
    CustodyAppend { target: CustodyTarget, event: CustodyEvent },
    StoryCreate { story: Story },
    StoryUpdate { story: StoryId, title: Option<String>, steps: Option<Vec<StoryStep>> },
    StoryLifecycleSet { story: StoryId, lifecycle: Lifecycle },
}
pub enum VisibilityTarget { EntityPresence(EntityId), Attribute(EntityId, AttrId), Edge(EdgeId), Story(StoryId) }
pub enum TierTarget { Entity(EntityId), Attribute(EntityId, AttrId), Edge(EdgeId), Story(StoryId) }
pub enum CustodyTarget { Entity(EntityId), Attribute(EntityId, AttrId), Edge(EdgeId), Story(StoryId), Group }
```

Note: `EntityArchive`/`EdgeArchive` from ADR-002 D3 are realized as
`*LifecycleSet` per A-B1 (lifecycle is a plain LWW field). Unarchive = set
Active.

## Module: fold (pure, wasm-safe)

```rust
pub struct GroupState {
    pub group: Option<Group>,
    pub template: Option<cn_schema::GroupTemplate>,
    pub entities: BTreeMap<EntityId, Entity>,
    pub edges: BTreeMap<EdgeId, Edge>,
    pub memberships: BTreeMap<MembershipId, Membership>,
    pub trust_grants: BTreeMap<TrustGrantId, TrustGrant>,
    pub stories: BTreeMap<StoryId, Story>,
    // private: field_clocks: HashMap<(ObjectKey, FieldKey), SortKey>
    // private: seen: BTreeSet<OpId>
    // private: quarantine: Vec<Quarantined>
}
pub enum FieldKey {   // field-level LWW granularity (ADR-002 round 2)
    Lifecycle, PresenceVisibility, Visibility, Tier, Owner, Weight,
    Attribute(AttrId),            // value + provenance as one field
    AttributeVisibility(AttrId), AttributeTier(AttrId),
    StoryTitle, StorySteps,
}
```

Fold rules (implement EXACTLY; each maps to an ADR clause):
1. Dedup: op_id already seen -> no-op (ADR-002 D4).
2. Apply in canonical SortKey order; every field write records its SortKey
   in field_clocks and lands ONLY if strictly greater than the recorded one
   (round-2 rule; creates record all their fields' keys).
3. Creates (EntityCreate etc.) on an existing id: treat the whole object as
   fields - later sort_key wins field-by-field; identical op_id impossible
   past dedup. Simplest compliant implementation: a second Create for an
   existing id is quarantined with reason DuplicateCreate (deterministic
   because canonical order fixes which arrived "first").
4. Ops referencing missing targets quarantine (reason MissingTarget) and the
   quarantine is re-examined to FIXPOINT in canonical order after every
   admitting pass (ADR-002 A-B4). Late admission cannot violate rule 2.
5. Template validation: an op validates against ITS template_version
   (A-B3). v0 simplification (record in result message): only the CURRENT
   pinned template version is known, so ops with template_version != the
   group's pinned version quarantine with reason TemplateVersionUnknown -
   migration chains land with TemplateMigrationApply in a later phase.
   Instance-level checks use cn_schema::validate_entity / validate_edge;
   semantic FAILURES quarantine (reason FailedValidation) with the findings
   attached.
6. TrustGrantRevoke maps to TrustGrant::revoke; AlreadyRevoked is a no-op at
   fold level (idempotent merge), RevokedBeforeGranted quarantines.
7. CustodyAppend: valid on Archived targets; unknown target -> MissingTarget
   quarantine. Duplicate custody event ids on the same envelope: no-op.
8. Fold NEVER panics; every quarantine lands in the StoreReport (I3, I12).

```rust
pub fn fold(ops: impl IntoIterator<Item = Operation>) -> (GroupState, StoreReport);
impl GroupState { pub fn apply(&mut self, op: Operation, report: &mut StoreReport); }
pub struct StoreReport { pub quarantined: Vec<QuarantineEntry>, pub warnings: Vec<StoreFinding>, pub applied: usize, pub deduped: usize }
pub struct QuarantineEntry { pub op_id: OpId, pub reason: QuarantineReason, pub findings: Vec<cn_schema::Finding> }
pub enum QuarantineReason { MissingTarget, DuplicateCreate, TemplateVersionUnknown, FailedValidation, RevokedBeforeGranted, WrongGroup }
```

(Report unification with cn-schema types is recorded debt; do not refactor.)

## Module: authz (trait only - implementation is cn-perm's, I2)

```rust
pub trait Authorizer {
    fn authorize(&self, state: &GroupState, op: &Operation) -> Result<(), AuthzDenial>;
}
pub struct AuthzDenial { pub code: String, pub message: String }
pub fn submit<A: Authorizer>(state: &mut GroupState, authorizer: &A,
    ops: Vec<Operation>, report: &mut StoreReport) -> Vec<SubmitOutcome>;
pub enum SubmitOutcome { Applied(OpId), Denied { op: OpId, denial: AuthzDenial }, Quarantined(OpId) }
```

## Module: log (native only: `#[cfg(not(target_arch = "wasm32"))]`)

```rust
pub struct OpLog { /* path, file handle */ }
impl OpLog {
    pub fn open(path: &Path) -> Result<(Self, Vec<Operation>, StoreReport), StoreError>;
    // - one op per line (serde_json); torn/invalid FINAL line -> truncate +
    //   warning finding TornLineRecovered (ADR-002 A-B7). Invalid NON-final
    //   line -> StoreError::CorruptLog (unrecoverable, typed).
    pub fn append_batch(&mut self, ops: &[Operation]) -> Result<(), StoreError>;
    // - writes all lines then ONE flush+sync (batch durability, A-B7).
}
pub struct Snapshot;   // save/load GroupState + watermark(last op_id + sort_key) + crc32
impl Snapshot {
    pub fn save(path: &Path, state: &GroupState) -> Result<(), StoreError>;
    pub fn load(path: &Path) -> Result<Option<(GroupState, SnapshotMeta)>, StoreError>;
    // checksum/parse failure or watermark ahead of provided log tip ->
    //   Ok(None) + caller refolds; expose the reason via StoreReport warning
    //   (SnapshotDiscarded { reason }).
}
```

`StoreError` (thiserror): Io (wraps std::io::Error with path context),
CorruptLog { line: usize }, Serialize(String), UnsupportedSchemaVersion.

## Test obligations

1. Convergence property: for a fixed op set over the research-network
   fixture template, fold(ops) == fold(shuffled(ops)) == fold(ops ++ ops)
   (duplication + reordering), asserted over final public state - at least
   3 shuffles, seeded deterministically (no RNG from system entropy; use a
   fixed-seed LCG or hand-rolled shuffles).
2. Round-2 LWW: quarantined low-sort AttributeSet admitted AFTER a
   higher-sort write to the same field does not overwrite it (the exact
   round-2 scenario: set(sort 1) quarantined, create(sort 2), set(sort 3),
   then fixpoint admits sort 1 - final value is sort 3's).
3. Fixpoint: EdgeCreate before its endpoints exists quarantines, then
   admits once both EntityCreates arrive; three-deep dependency chains
   admit in one fold call.
4. Lifecycle LWW: archive then concurrent lower-sort AttributeSet still
   lands in the record; liveness derived from lifecycle only; unarchive via
   later Active set wins by sort order.
5. HlcClock: monotonic under stalled and REGRESSING wall clock; counter
   resets on wall advance.
6. Authorization: a denying Authorizer prevents append/application and
   yields Denied outcome; a permitting one applies (use test stub
   authorizers - cn-perm is NOT a dependency).
7. Log: append_batch then reopen replays identical state; torn final line
   (write garbage bytes without newline) recovers with warning; corrupt
   middle line -> CorruptLog error.
8. Snapshot: save/load round-trip; flipped byte -> Ok(None) discard path
   with reason; watermark ahead of a shorter log -> discarded.
9. Validation quarantine: entity with attr not in template quarantines with
   FailedValidation carrying cn-schema findings; weight on Forbidden edge
   kind quarantines.
10. WrongGroup: op whose group_id mismatches the state's group quarantines.

Verification: fmt, clippy -D warnings, test --workspace, and
`cargo build --target wasm32-unknown-unknown -p cn-store` (fold + op modules
must compile for wasm; log module gated out).
