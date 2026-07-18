# Authority Matrix - Role x Op-Kind (P3.2)

Status: implemented 2026-07-17 from the committed director blueprint
`docs/blueprints/facilitator-role.md` (integration plan 6.3; D-028, D-036,
D-037). Permission-adjacent: covered by the mandatory adversarial round.

Write authorization lives only in `core/crates/cn-perm/src/authz.rs` (I2).
Every cell below is asserted by `core/crates/cn-perm/tests/authority_matrix.rs`;
inherited behavior is asserted, not assumed. End-to-end coverage through the
cn-api facade lives in `core/crates/cn-api/tests/facilitator_role.rs`.

## Roles

- **Anonymous**: a submitter with no active membership in the group.
- **Member**: active `GroupRole::Member`.
- **Facilitator**: active `GroupRole::Facilitator` (D-028: the person running
  data entry and story authoring at the pilot). Narrow WRITE authority, NO new
  read reach: a facilitator reads at member level (Group circle) and is not
  governance for any read surface, report, or custody-depth purpose.
- **Governance**: active `GroupRole::Governance`.
- **Owner**: the person a record's `owner` field names; owner rules are
  role-independent.

Dual-role holders take the union of their roles' authority; a role never
subtracts. Roles are additive memberships, so one person can hold Member and
Facilitator at once.

## Matrix

UNCHANGED means the pre-facilitator rule already yields the right answer for
facilitators because they are active members. The two CHANGED rows are the
entire behavioral delta of the facilitator role.

| OpKind | Anonymous | Member | Facilitator | Governance | Change |
|---|---|---|---|---|---|
| GroupCreate | bootstrap rule unchanged (first group only) | - | - | - | none |
| EntityCreate (unowned) | deny `not_member` | allow | allow | allow | none |
| EntityCreate (owned by other) | deny | deny | deny | allow | none - facilitator does NOT bypass owner controls |
| EdgeCreate / StoryCreate | deny `not_member` | allow | allow | allow | none |
| AttributeSet/Remove (unowned target) | deny | allow | allow | allow | none |
| AttributeSet/Remove (owned by other) | deny | deny | deny | allow | none |
| EntityLifecycleSet (unowned) | deny | deny | deny | allow | none - archival stays governance |
| EdgeLifecycleSet / StoryLifecycleSet | deny | deny | deny | allow | none |
| VisibilitySet (owned) | deny | owner only | owner only | deny (non-owner) | none - facilitator can NEVER loosen another's visibility |
| VisibilitySet (unowned edge/story/entity) | deny | deny | deny | allow | none |
| TierSet | deny | owner tighten-only | owner tighten-only | allow | none - tier authority is ATNI Climate via governance (D-034) |
| EdgeWeightSet | deny `not_member` | allow | allow | allow | none |
| MembershipAdd / MembershipLifecycleSet | deny | deny | deny | allow | none - facilitator CANNOT govern membership or grant roles |
| TrustGrantCreate/Revoke | grantor-only rule unchanged for all roles | | | | none |
| StoryUpdate | deny | deny | **visible targets only** | allow | **CHANGED**: `require_governance` -> target-aware facilitator-or-governance |
| CustodyAppend | deny | any visible target | **created-records only** | any target | **CHANGED** - see below |

## StoryUpdate (the one loosening)

`OpKind::StoryUpdate` moved from `require_governance` to a target-aware
facilitator-or-governance rule (D-037 in-app authoring authority, tightened by
the 2026-07-17 adversarial round). Evaluation order (`authorize_story_update`):

1. Governance: any story, unchanged.
2. Non-facilitators are denied `not_facilitator_or_governance` BEFORE any
   target lookup, so they learn nothing about a story's existence.
3. The target story must exist (`target_missing`) and be visible to the
   facilitator (`target_hidden`): facilitator write reach never exceeds
   facilitator read reach, so a facilitator cannot blind-overwrite a hidden
   story by guessing its id.

StoryCreate stays member-open - that is the pre-existing rule, documented as
such, not a facilitator change.

## CustodyAppend (the one tightening - new role/ownership logic)

Rule, in evaluation order (`authorize_custody`):

1. Governance: any target, unchanged.
2. Any other submitter must be an active member of some role (`not_member`).
3. The target must be visible to the submitter (`target_missing` /
   `target_hidden`), unchanged.
4. Viewer whose active roles include Member: any visible target, unchanged.
   The member rule dominates for dual-role holders.
5. Viewer whose active roles are Facilitator-only (no Member, no Governance):
   allowed ONLY when the targeted record's provenance `responsible_human` is
   the submitter - their own created/imported records. Otherwise
   `custody_not_own_record`.

Rationale (blueprint section 4): pilot facilitators are typically staff, not
community members (D-028); their custody authority follows their entry work,
not the whole visible graph.

Target resolution for step 5 uses the targeted record's OWN provenance
envelope:

- Entity / Edge / Story: that record's envelope.
- Attribute: the attribute instance's envelope, NOT the containing entity's.
  A facilitator who created an entity does not gain custody over attributes a
  member later entered on it; the least-new-privilege principle follows the
  entry work itself. (Implementation interpretation - the blueprint says
  "targets whose provenance responsible_human is the submitter"; the attribute
  instance is the targeted record.)
- Group: the group record's envelope (in practice the group creator, so a
  facilitator-only viewer is denied).

## Denial codes

Stable codes from `PermAuthorizer` touched or added by this work:
`not_facilitator_or_governance` (new, StoryUpdate), `custody_not_own_record`
(new, facilitator-only custody), plus the unchanged `not_member`,
`not_governance`, `not_owner`, `not_owner_or_governance`,
`tier_not_tightened`, `bootstrap_denied`, `grantor_mismatch`,
`target_missing`, `target_hidden`, `group_exists`. StoryUpdate now also
returns `target_missing` / `target_hidden` for facilitator submitters
(target-aware rule above).

## Read reach and cache (P3.1/P3.3 context)

- `is_facilitator_or_governance` (`cn-perm/src/viewer.rs`) is a write-path
  predicate only; no read-path predicate gained a facilitator arm.
- `admissible_circle` is unchanged: any active role reads at Group circle.
- `active_role_names` now emits `"facilitator"`, so `viewer_fingerprint`
  distinguishes member, facilitator, and dual-role viewers.
- The hashed `viewer_fingerprint` is a 32-bit CRC and admits engineered
  collisions across viewers (adversarial round finding, with a concrete
  member/facilitator collision pair). The cn-api projection and index caches
  therefore key on `viewer_cache_key` - the canonical authorization context,
  collision-free by construction, in-memory only, never serialized - so a
  cache entry can never be served across distinct viewer contexts. The
  exported `Projection.viewer_fingerprint` field keeps the hashed form.
- Report redaction is unchanged: a facilitator receives the member-level
  redacted report, never the governance report.

## Test traceability

| Matrix row | Test (`cn-perm/tests/authority_matrix.rs`) |
|---|---|
| GroupCreate | `group_create_cells` |
| EntityCreate (unowned) | `entity_create_unowned_cells` |
| EntityCreate (owned by other) | `entity_create_owned_by_other_cells` |
| EdgeCreate | `edge_create_cells` |
| StoryCreate | `story_create_cells` |
| AttributeSet/Remove (unowned) | `attribute_mutation_unowned_target_cells` |
| AttributeSet/Remove (owned) | `attribute_mutation_owned_by_other_cells` |
| EntityLifecycleSet | `entity_lifecycle_cells` |
| Edge/StoryLifecycleSet | `edge_and_story_lifecycle_cells` |
| VisibilitySet | `visibility_cells` |
| TierSet | `tier_cells` |
| EdgeWeightSet | `edge_weight_cells` |
| MembershipAdd | `membership_add_cells` |
| MembershipLifecycleSet | `membership_lifecycle_cells` |
| TrustGrantCreate/Revoke | `trust_grant_cells` |
| StoryUpdate | `story_update_cells` |
| CustodyAppend (governance) | `custody_governance_cells` |
| CustodyAppend (member-only) | `custody_member_only_cells` |
| CustodyAppend (facilitator-only) | `custody_facilitator_only_cells` |
| CustodyAppend (dual-role) | `custody_dual_role_cells` |

Fingerprint distinctness and facilitator read-reach/report tests:
`cn-perm/tests/blueprint.rs`
(`fingerprint_distinguishes_member_facilitator_and_dual_role`,
`facilitator_reads_at_member_level_and_report_is_not_governance`); five-class
no-leak property: `property_projection_sound_and_complete` with the extended
`role_strategy`. Wire round-trip and unknown-role rejection (D-044.7):
`cn-model/src/group.rs` tests.
