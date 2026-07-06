# Blueprint: cn-model (Phase 2, authored by director from ADR-001 + amendments)

Implementer contract: implement EXACTLY these public types and invariants in
`core/crates/cn-model`. You may add private helpers and internal modules, but
every public item below must exist with these semantics. No system time or
RNG calls outside the documented constructors; no `unwrap`/`expect` outside
tests; all fallible paths return `ModelError` (I3). Hyphens in all docs.

## Dependencies (core/crates/cn-model/Cargo.toml)

```toml
[dependencies]
uuid = { version = "1", features = ["v7", "serde", "js"] }
serde = { version = "1", features = ["derive"] }
semver = { version = "1", features = ["serde"] }
thiserror = "2"

[dev-dependencies]
serde_json = "1"
```

`js` feature makes uuid's RNG work under wasm32-unknown-unknown; it is a
no-op on native. NOTHING in this crate may call `SystemTime::now` - all
timestamps are injected by callers (cn-api supplies a clock; tests supply
constants). ADR-002 sort keys do NOT live here; operations are cn-store's.

## Module: time

```rust
/// Milliseconds since the Unix epoch, injected by callers - the core never
/// reads a system clock (determinism + wasm safety).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(pub i64);
```

## Module: ids

A macro generates UUIDv7 newtypes. Each has:
`fn new(at: Timestamp) -> Self` (uuid v7 built from the injected timestamp),
`fn from_uuid(u: Uuid) -> Self`, `fn as_uuid(&self) -> &Uuid`, `Display`,
`FromStr` (error -> ModelError::InvalidId), serde transparent, Ord.

Generate: `EntityId`, `EdgeId`, `GroupId`, `PersonId`, `OpId`, `StoryId`,
`CustodyEventId`, `MembershipId`, `TrustGrantId`.

String-backed validated newtypes (lowercase slug, `^[a-z][a-z0-9_-]*$`,
constructor returns `Result<Self, ModelError>`; serde with validation on
deserialize): `KindId`, `AttrId`, `TemplateId`.

## Module: circle

```rust
/// Audience lattice. Ord = audience size: Private < Trusted < Group <
/// Network < Public. `min` of two circles is the MORE RESTRICTIVE one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Circle { Private, Trusted, Group, Network, Public }
```

## Module: tier

```rust
/// TSDF-style sensitivity tier. Ord = restrictiveness: T0 < T1 < T2 < T3
/// (T3 most restrictive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SensitivityTier { T0, T1, T2, T3 }

impl SensitivityTier {
    /// ADR-001 A-B2 ceiling ON the circle lattice.
    /// T0 -> Public, T1 -> Network, T2 -> Group, T3 -> Private.
    pub fn ceiling(self) -> Circle;
    /// max restrictiveness (= std::cmp::max under this Ord).
    pub fn most_restrictive(a: Self, b: Self) -> Self;
}
```

## Module: provenance

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    SelfReported,
    Ingested { source: String },
    Derived { inputs: Vec<OpId> },
    Authored,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorRef { Human(PersonId), Agent { agent_id: String } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustodyAction { Created, Imported, Migrated, Corrected, Exported }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustodyEvent {
    pub id: CustodyEventId,
    pub action: CustodyAction,
    pub at: Timestamp,
    pub actor: ActorRef,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceEnvelope {
    origin: Origin,
    recorded_by: ActorRef,
    responsible_human: PersonId,
    recorded_at: Timestamp,
    custody: Vec<CustodyEvent>,   // PRIVATE - append-only via method (ADR-001 D8)
    schema_version: semver::Version,
}

impl ProvenanceEnvelope {
    /// The ONLY constructor. When `recorded_by` is Agent, `responsible_human`
    /// is the accountable person (I6/R10); when Human, it must equal that
    /// person (pass the same id) - constructor enforces both, returning
    /// ModelError::ResponsibleHumanMismatch otherwise.
    pub fn new(origin: Origin, recorded_by: ActorRef, responsible_human: PersonId,
               recorded_at: Timestamp) -> Result<Self, ModelError>;
    pub fn append_custody(&mut self, event: CustodyEvent);   // push-only
    pub fn custody(&self) -> &[CustodyEvent];
    // getters for the other private fields (read-only)
}
```

Deserialization must not bypass invariants: implement a validating
`Deserialize` (deserialize into a private raw struct, then run the same
checks as `new`, plus custody accepted as-is because logs replay it).

## Module: attribute

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum AttributeValue {
    Text(String),
    Number(f64),
    Enum(String),                    // symbol; template validates membership (cn-schema)
    Tags(std::collections::BTreeSet<String>),
    Date(IsoDate),
    Geo(GeoValue),
    Link(LinkValue),
    Media(MediaRefId),
}

/// "YYYY-MM-DD", validated (real calendar date, no time component).
pub struct IsoDate(String);          // constructor validates; serde validates

#[derive(...)] #[serde(rename_all = "snake_case")]
pub enum GeoValue { Point { lat: f64, lon: f64 }, Region { name: String } }
// Point constructor validates lat in [-90,90], lon in [-180,180].

/// URL with optional format hint. `format: email` values are stored as
/// mailto: URLs (ADR-001 amendment).
pub struct LinkValue { url: String, format: Option<LinkFormat> }  // validated constructor
pub enum LinkFormat { Email, Url }

pub struct MediaRefId(String);       // opaque, non-empty; resolution is Phase 4

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeInstance {
    pub value: AttributeValue,
    pub visibility: Circle,
    tier_override: Option<SensitivityTier>,   // PRIVATE - tighten-only setter
    pub provenance: ProvenanceEnvelope,
}

impl AttributeInstance {
    pub fn new(value: AttributeValue, visibility: Circle,
               provenance: ProvenanceEnvelope) -> Self;   // no override initially
    /// ADR-001 A-B3: override may only TIGHTEN relative to the entity tier
    /// context in which it is evaluated; the setter enforces
    /// `new_tier > current effective tier is the only accepted direction`
    /// given the entity tier passed in. Returns ModelError::TierLoosenAttempt.
    pub fn tighten_tier(&mut self, entity_tier: SensitivityTier,
                        new_tier: SensitivityTier) -> Result<(), ModelError>;
    /// max restrictiveness of entity tier and override (ADR-001 A-B3).
    pub fn effective_tier(&self, entity_tier: SensitivityTier) -> SensitivityTier;
}
```

## Module: lifecycle

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle { Active, Archived }   // ADR-002 A-B1: plain LWW field
```

## Modules: entity, edge, group, membership, trust

All structs: public readonly getters or public fields as noted; every
constructor REQUIRES a ProvenanceEnvelope and SensitivityTier (I6) - there is
no Default and no constructor without them.

```rust
pub struct Entity {
    pub id: EntityId,
    pub group_id: GroupId,
    pub kind: KindId,
    pub attributes: std::collections::BTreeMap<AttrId, AttributeInstance>,
    pub owner: Option<PersonId>,
    pub presence_visibility: Circle,
    pub lifecycle: Lifecycle,
    pub provenance: ProvenanceEnvelope,
    pub tier: SensitivityTier,
    pub schema_version: semver::Version,
}
impl Entity { pub fn new(id, group_id, kind, presence_visibility, provenance, tier) -> Self; }
// new() starts Active, empty attributes, schema_version = MODEL_SCHEMA_VERSION.

pub struct Edge {
    pub id: EdgeId, pub group_id: GroupId, pub kind: KindId,
    pub from: EntityId, pub to: EntityId, pub directed: bool,
    pub weight: Option<f64>,          // finite; setter rejects NaN/inf
    pub attributes: BTreeMap<AttrId, AttributeInstance>,
    pub visibility: Circle, pub lifecycle: Lifecycle,
    pub provenance: ProvenanceEnvelope, pub tier: SensitivityTier,
    pub schema_version: semver::Version,
}
impl Edge { pub fn new(id, group_id, kind, from, to, directed, visibility, provenance, tier) -> Self; }

pub struct Group {
    pub id: GroupId, pub name: String,
    pub template_id: TemplateId, pub template_version: semver::Version,  // pinned (ADR-001 D9)
    pub provenance: ProvenanceEnvelope, pub tier: SensitivityTier,
    pub schema_version: semver::Version,
}
impl Group { pub fn new(...) -> Result<Self, ModelError>; }  // name non-empty

#[derive(...)] #[serde(rename_all = "snake_case")]
pub enum GroupRole { Member, Governance }

pub struct Membership {   // CORE primitive, not a template edge (ADR-001 D5)
    pub id: MembershipId, pub group_id: GroupId, pub person: PersonId,
    pub role: GroupRole, pub lifecycle: Lifecycle,
    pub provenance: ProvenanceEnvelope, pub schema_version: semver::Version,
}

pub struct TrustGrant {   // CORE primitive (ADR-001 D5); owner-managed, audited
    pub id: TrustGrantId, pub grantor: PersonId, pub grantee: PersonId,
    pub scope: TrustScope, pub granted_at: Timestamp,
    pub revoked_at: Option<Timestamp>,   // set-once via revoke()
    pub provenance: ProvenanceEnvelope, pub schema_version: semver::Version,
}
pub enum TrustScope { All, Group(GroupId) }
impl TrustGrant {
    pub fn revoke(&mut self, at: Timestamp) -> Result<(), ModelError>;  // AlreadyRevoked; at >= granted_at
    pub fn is_active(&self) -> bool;
}
```

`pub const MODEL_SCHEMA_VERSION: &str = "0.1.0";` plus
`pub fn model_schema_version() -> semver::Version;` and
`pub fn accepts_schema(v: &semver::Version) -> bool` - true iff same MAJOR
(I7; 0.x: same MAJOR and MINOR).

## Module: error

```rust
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ModelError {
    #[error("invalid id: {0}")] InvalidId(String),
    #[error("invalid slug '{value}' for {what}")] InvalidSlug { what: &'static str, value: String },
    #[error("invalid date: {0}")] InvalidDate(String),
    #[error("invalid url: {0}")] InvalidUrl(String),
    #[error("invalid geo coordinates: lat {lat}, lon {lon}")] InvalidGeo { lat: f64, lon: f64 },
    #[error("non-finite weight")] NonFiniteWeight,
    #[error("responsible human mismatch for human actor")] ResponsibleHumanMismatch,
    #[error("tier override may only tighten (current effective {current:?}, attempted {attempted:?})")]
    TierLoosenAttempt { current: SensitivityTier, attempted: SensitivityTier },
    #[error("trust grant already revoked")] AlreadyRevoked,
    #[error("revocation predates grant")] RevokedBeforeGranted,
    #[error("empty name")] EmptyName,
    #[error("unsupported schema version {0}")] UnsupportedSchemaVersion(String),
}
```

## Test obligations (author these; they define done)

1. Circle Ord: Private < Trusted < Group < Network < Public; min() picks the
   more restrictive.
2. Tier ceilings exactly per A-B2; most_restrictive symmetric.
3. ProvenanceEnvelope::new rejects Agent-recorded without matching
   accountability semantics and Human-recorded with a DIFFERENT
   responsible_human; accepts the two valid shapes.
4. Custody is append-only: no public mutation of existing events (compile-time
   API review) and append preserves order.
5. AttributeInstance::tighten_tier accepts strictly-more-restrictive, rejects
   equal and looser with TierLoosenAttempt; effective_tier = max restrictive.
6. IsoDate: accepts 2026-02-28, rejects 2026-02-30, 2026-13-01, "junk",
   datetime strings. GeoValue::point rejects out-of-range. LinkValue: email
   format stores mailto:, rejects clearly invalid URLs. Slug newtypes reject
   uppercase/spaces/leading digits.
7. Serde round-trip for every public type (construct -> json -> back -> eq);
   deserializing an envelope that violates constructor invariants FAILS.
8. Edge weight setter rejects NaN and infinity.
9. TrustGrant revoke: happy path, AlreadyRevoked, RevokedBeforeGranted;
   is_active flips.
10. accepts_schema: same 0.MINOR true, different MINOR false (0.x rule),
    and for 1.x hypothetical: same major true.
11. ids: v7 ids from a fixed Timestamp are distinct (random component) and
    FromStr/Display round-trip.
```

Verification: `cargo fmt --check`, `cargo clippy --workspace --all-targets
-- -D warnings`, `cargo test --workspace` from core/ - all green.
