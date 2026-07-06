# Blueprint: cn-perm (Phase 2, authored by director)

Sources are NORMATIVE: docs/specs/permission-model.md, ADR-001 (D5-D7,
A-B1..A-B4), ADR-002 (A-B2, A-B6), ADR-003 (A-B1..A-B4, round 2). This crate
is the ONLY place permission decisions live (I2). Everything wasm-safe.

## Dependencies

```toml
[dependencies]
cn-model = { path = "../cn-model" }
cn-schema = { path = "../cn-schema" }
cn-store = { path = "../cn-store" }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
crc32fast = "1"

[dev-dependencies]
proptest = "1"
serde_json = "1"
```

## Module: viewer

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ViewerContext { Anonymous, Person { person: PersonId } }
```

The Phase 5 viewer-switcher ("view as public / group / trusted / self") is
realized by passing DIFFERENT contexts, never by modifying rules. Admin is
not a context: governance is a Membership ROLE that widens REPORT detail
only, never value access (spec section 3).

```rust
/// Highest circle this viewer reaches for a value owned by `owner` in this
/// group (spec section 2; v0 "Network" = any authenticated person, firmed
/// up by the future network ADR - document this on the fn):
/// Anonymous -> Public.
/// Person    -> Network; plus Group if ACTIVE membership in the group;
///              plus Trusted if an ACTIVE TrustGrant grantor==owner,
///              grantee==viewer, scope All or Group(this);
///              plus Private if viewer == owner.
/// Values with NO owner (owner: None) use the entity's group as owner
/// authority: Private unreachable, Trusted unreachable, so their floor is
/// Group behavior.
pub fn admissible_circle(state: &GroupState, viewer: &ViewerContext,
                         owner: Option<&PersonId>) -> Circle;
```

## Module: rules (pure functions; spec section 5 verbatim)

```rust
pub fn effective_tier(container: SensitivityTier, override_: Option<SensitivityTier>) -> SensitivityTier;
// = SensitivityTier::most_restrictive(container, override_.unwrap_or(container))

pub fn required_circle(value_visibility: Circle, eff_tier: SensitivityTier) -> Circle;
// = min(value_visibility, eff_tier.ceiling())   (min = more restrictive)

pub fn value_visible(admissible: Circle, value_visibility: Circle, eff_tier: SensitivityTier) -> bool;
// = admissible <= required_circle(...)
// ERRATA (director, 2026-07-06): originally written `>=`, which contradicted
// the min() rule under the audience-size Ord (deeper access = SMALLER value:
// an owner reaching Private(0) sees a Private-required value; anonymous at
// Public(4) does not). Implementation and tests use `<=`; the tier-cell and
// property tests pin the semantics.
```

## Module: projection

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Projection {
    pub group_id: GroupId,
    pub viewer_fingerprint: String,
    pub revision: u64,               // supplied by caller (cn-api tracks it)
    pub entities: Vec<ProjectedEntity>,
    pub edges: Vec<ProjectedEdge>,
    pub stories: Vec<ProjectedStory>,
}
pub struct ProjectedEntity { pub id: EntityId, pub kind: KindId,
    pub owner_is_viewer: bool,
    pub attributes: BTreeMap<AttrId, AttributeValue> }   // ONLY visible values
pub struct ProjectedEdge { pub id: EdgeId, pub kind: KindId,
    pub from: EntityId, pub to: EntityId, pub directed: bool,
    pub weight: Option<f64>,
    pub attributes: BTreeMap<AttrId, AttributeValue> }
pub struct ProjectedStory { pub id: StoryId, pub title: String,
    pub steps: Vec<ProjectedStoryStep> }   // silent elision (A-B4): only
                                           // steps whose entity is projected
pub struct ProjectedStoryStep { pub entity: EntityId, pub narration: String }

pub fn project(state: &GroupState, viewer: &ViewerContext, revision: u64) -> Projection;
```

Projection rules (each maps to a spec/ADR clause - cite in doc comments):
1. Only Active lifecycle objects are considered at all.
2. Entity projected iff value_visible(admissible for entity.owner,
   entity.presence_visibility, effective_tier(entity.tier, None)).
3. Attribute included iff its OWN rule passes: admissible for entity.owner,
   instance.visibility, effective_tier(entity.tier, instance tier override).
   (Use AttributeInstance::effective_tier.)
4. Edge projected iff BOTH endpoints are projected AND the edge's own circle
   + tier pass (ADR-001 D6); edge attributes filter like entity attributes
   with the edge's tier as container.
5. Story projected iff its own circle + tier pass; steps referencing
   non-projected entities are elided silently - no markers, no counts
   (A-B4). A story with zero visible steps still projects (title only) if
   the story itself is visible.
6. NOTHING in the output may reference a non-projected entity id anywhere.
7. Provenance envelopes and tiers are NOT in projections (they are detail
   data; cn-api's entity_detail path adds viewer-filtered detail later -
   still only via this crate's rules; that fn lands with cn-api).
8. viewer_fingerprint = crc32 of the canonical string:
   "anon" | "person:<uuid>" + ":" + sorted active grant ids where viewer is
   grantee + ":" + role names of viewer's active memberships in this group
   + ":" + template_version (ADR-003 amendment). Provide
   `pub fn viewer_fingerprint(state, viewer) -> String`.

## Module: reports (ADR-003 A-B1 + round 2)

```rust
/// Filter a StoreReport for a viewer: keeps ONLY (a) findings whose subject
/// object is projected for this viewer and (b) outcomes of ops whose
/// actor's responsible_human == the viewer (their own submissions), with
/// references to non-visible objects reduced to NotFound opacity. No counts
/// of anything else - the redacted report must be indistinguishable from a
/// report generated over a world where hidden objects do not exist.
/// Governance-role viewers receive the report unfiltered.
pub fn redact_report(state: &GroupState, viewer: &ViewerContext,
                     report: &cn_store::StoreReport) -> cn_store::StoreReport;
```

## Module: authz (implements cn_store::Authorizer; ADR-002 A-B6)

```rust
pub struct PermAuthorizer;
impl cn_store::Authorizer for PermAuthorizer { ... }
```

Rules (op actor's responsible_human is "the submitter"):
- GroupCreate: only when state has no group yet.
- Entity/Edge/Story Create, AttributeSet/Remove on UNOWNED records: any
  ACTIVE member. On OWNED records (entity.owner = Some(p)): only p or
  governance.
- Entity/Edge/Story LifecycleSet: owner (if owned) or governance.
- VisibilitySet: the value's owner (entity.owner for presence/attributes;
  for unowned targets: governance).
- TierSet: governance (any tier) OR owner with STRICTLY more restrictive
  tier than current effective (ADR-002 round-2 predicate). Anything else:
  typed denial.
- MembershipAdd/LifecycleSet: governance only (bootstrap exception: the
  group's FIRST membership may be self-added by the group creator).
- TrustGrantCreate/Revoke: grantor == submitter only.
- CustodyAppend: any active member on visible targets; governance otherwise.
- Anonymous never authorizes anything.
Every denial returns a distinct code string (stable, documented).

## Test obligations

1. THE PROPERTY TEST (Phase 2 acceptance, CLAUDE.md): proptest generating
   arbitrary small states (up to ~20 entities, random owners/circles/tiers/
   attribute overrides, random grants incl. revoked, random memberships
   incl. archived, edges, stories) and arbitrary viewers. For EVERY
   projected attribute assert value_visible(...) holds; for every attribute
   NOT projected on a projected entity assert it fails; assert no edge
   references an unprojected endpoint; assert no story step references an
   unprojected entity. Soundness AND completeness.
2. Tier ceiling cells: T3 attribute marked Public is invisible to everyone
   except owner (and invisible in export contexts - covered at cn-api);
   T2 visible to member, not to non-member person; T1 visible to any
   Person, not Anonymous; T0+Public visible to Anonymous.
3. The fisheries scenario from the spec: T1 fishing_site entity, T3
   site_knowledge override, trusted-only location - owner sees all, trusted
   grantee sees location but never site_knowledge, member sees neither,
   anonymous sees nothing (presence gated).
4. Revoked grant: trusted attribute visible before revoke, invisible after
   (fold a TrustGrantRevoke, reproject).
5. Archived member: their membership no longer yields Group access.
6. Story elision: hidden middle step elided, order preserved, no gap
   markers; fully-hidden story absent; visible-but-empty story present.
7. redact_report: non-member submitter sees own quarantined op with
   MissingTarget reduced to NotFound opacity; governance sees everything;
   member sees findings only for projected objects; a viewer with NOTHING
   projected receives an EMPTY report (no counts).
8. Authorizer: each rule above gets a positive and negative case, including
   the TierSet owner-tighten happy path and the owner-loosen denial, and
   the first-membership bootstrap exception.
9. Fingerprint changes when: a grant to the viewer is created/revoked, the
   viewer's role changes, template version changes; and is IDENTICAL for
   irrelevant state changes (someone else's grant).

Verification: fmt, clippy -D warnings, test --workspace, and
`cargo build --target wasm32-unknown-unknown -p cn-perm` (proptest is
dev-only so wasm build must not require it).
