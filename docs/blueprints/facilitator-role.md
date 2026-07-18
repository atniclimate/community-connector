# Blueprint: GroupRole::Facilitator (P3.1-P3.4)

Status: director blueprint, 2026-07-17. Implements integration plan spec 6.3
(D-028, D-036) as gate-blind Phase 3 work per PLAN_1.0.md. Permission-adjacent:
implementation is grind HIGH and gets a MANDATORY adversarial Codex round
before its commits are accepted. This blueprint changes no accepted ADR.

## Design intent

A facilitator is the person running data entry and story authoring at the
pilot (D-028). The role grants narrow WRITE authority and NO new read reach.
The controlling principle is least new privilege: every cell below either
inherits an existing rule unchanged or grants the minimum named power.

## 1. Model change (cn-model/src/group.rs)

Add a third variant:

```rust
pub enum GroupRole {
    Member,
    Governance,
    /// Pilot facilitator: entry/import and story authoring authority
    /// without governance powers (D-028, integration plan 6.3).
    Facilitator,
}
```

serde stays `snake_case` (`"facilitator"` on the wire). This is a NON-breaking
schema addition: no persisted format changes shape; existing ops replay
unchanged (I7 unaffected; op-log major stays).

## 2. Read reach (cn-perm/src/viewer.rs) - NO code change to circles

- `is_active_member` already returns true for any active role, and
  `admissible_circle` grants `Circle::Group` to any active member. A
  facilitator therefore reads at member level (Group circle) automatically.
- A facilitator is NOT `is_governance`: governance-only surfaces (full custody
  chains per D-033, governance counts) stay hidden. Add NO facilitator arm to
  any read-path predicate.
- New predicate beside `is_governance`:

```rust
pub fn is_facilitator_or_governance(state: &GroupState, person: PersonId) -> bool {
    active_roles_for(state, &person)
        .any(|role| matches!(role, GroupRole::Facilitator | GroupRole::Governance))
}
```

## 3. Fingerprint (cn-perm/src/viewer.rs `active_role_names`)

The match in `active_role_names` is exhaustive over two variants today; adding
the third FORCES a new arm (the compiler is the tripwire - do not add a
wildcard arm):

```rust
GroupRole::Facilitator => "facilitator".to_string(),
```

Required test (P3.3): two viewers with identical memberships/grants except
role Member vs Facilitator produce DIFFERENT `viewer_fingerprint` values, so
`GroupSession`'s projection cache (cn-api/src/session.rs) can never serve a
facilitator a member's cached projection or vice versa. Also assert a
Facilitator+Member dual-role holder fingerprints differently from either
single role (roles vec is sorted, so ordering is deterministic).

## 4. Authority matrix (cn-perm/src/authz.rs)

Op-kind by role. UNCHANGED means the existing rule already yields the right
answer for facilitators BECAUSE they are active members; those cells still
get explicit tests (P3.2) - inherited behavior is asserted, not assumed.

| OpKind | Anonymous | Member | Facilitator | Governance | Change? |
|---|---|---|---|---|---|
| GroupCreate | bootstrap rule unchanged | - | - | - | none |
| EntityCreate (unowned) | deny | allow | allow | allow | none (require_member) |
| EntityCreate (owned by other) | deny | deny | **deny** | allow | none - facilitator does NOT bypass owner controls |
| EdgeCreate / StoryCreate | deny | allow | allow | allow | none (require_member) |
| AttributeSet/Remove (unowned target) | deny | allow | allow | allow | none |
| AttributeSet/Remove (owned by other) | deny | deny | **deny** | allow | none |
| EntityLifecycleSet (unowned) | deny | deny | **deny** | allow | none (require_governance) - archival stays governance |
| EdgeLifecycleSet / StoryLifecycleSet | deny | deny | **deny** | allow | none |
| VisibilitySet (owned) | deny | owner only | owner only | deny (non-owner) | none - facilitator can NEVER loosen another's visibility |
| TierSet | deny | owner tighten-only | owner tighten-only | allow | none - tier authority is ATNI Climate via governance (D-034) |
| EdgeWeightSet | deny | allow | allow | allow | none (require_member) |
| MembershipAdd / MembershipLifecycleSet | deny | deny | **deny** | allow | none - facilitator CANNOT govern membership or grant roles |
| TrustGrantCreate/Revoke | grantor-only rule unchanged | | | | none |
| **StoryUpdate** | deny | deny | **ALLOW** | allow | **CHANGED: require_governance -> require is_facilitator_or_governance** |
| **CustodyAppend** | deny | any visible target (existing) | **created-records only (NEW)** | any target | **CHANGED - see below** |

### StoryUpdate (the one loosening)

`OpKind::StoryUpdate` moves from `require_governance` to
`is_facilitator_or_governance`. This is the D-037 in-app authoring authority.
StoryCreate stays member-open (existing rule, documented as such).

### CustodyAppend (the one tightening - NEW role/ownership logic)

Existing rule: governance -> any target; member -> any VISIBLE target. The
facilitator role must not silently widen custody reach through bulk
entry/import powers. New rule, replacing the body of `authorize_custody`:

- governance: any target (unchanged).
- viewer whose active roles include Member: any visible target (unchanged -
  the member rule dominates for dual-role holders; the role never SUBTRACTS).
- viewer whose active roles are Facilitator-only (no Member, no Governance):
  custody append allowed ONLY on targets whose provenance
  `responsible_human` is the submitter (their own created/imported records),
  and the target must still be visible to them.

Rationale: pilot facilitators are typically staff, not community members
(D-028); their custody authority follows their entry work, not the whole
visible graph. A facilitator who is ALSO a member keeps member reach - the
matrix cell tests must cover facilitator-only, member-only, and dual-role.

## 5. Property test extension (P3.4)

Extend the cn-perm no-leak property (cn-perm/tests/blueprint.rs) from its
current viewer classes to all five: anonymous, member, facilitator, self,
governance. The property stays a READ property: no projection, detail,
search, path, or export surface returns an attribute above the viewer's
admissible circle, and governance-only custody depth never appears for a
facilitator. The proptest strategy that generates memberships must now
generate Facilitator roles (including dual-role holders).

## 6. Out of scope for this blueprint

- No UI work (P3.5 wizard / P3.6 entry forms are separate units layered on
  cn-api submit - they add NO permission logic in the app layer, I2).
- No new read tier, no consent semantics (P5.2 parks on G-RAT), no importer.
- No change to bootstrap, trust-grant, or tier rules.

## 7. Acceptance (P3.1-P3.4 together)

- cargo test --workspace green including new exhaustive matrix tests (every
  matrix row above has a test per relevant role, including the three custody
  role-combinations).
- Fingerprint distinctness tests green.
- Extended five-class no-leak property green.
- check-all green; adversarial Codex round run against the full diff with
  this blueprint attached; blocking findings resolved before acceptance.
