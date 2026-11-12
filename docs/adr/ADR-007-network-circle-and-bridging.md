# ADR-007: The Network circle and bridging between graphs - TSDF T1 reciprocal compacts and T2 negotiated agreements

- Status: Proposed (2026-11-11). Revised the same day after one review
  round: three review lenses and a Jev citation check (see Review log). Not
  accepted. The human accepts this ADR as a whole, because it is the
  human-gated network ADR (docs/PROJECT_PLAN.md:258). After acceptance,
  Phase A (D1-D4, D7, D8) is director work to build, because building it on
  synthetic fixtures crosses no CLAUDE.md gate. Serving agreement-admitted
  real data to a real outside partner stays behind the existing human gates
  (D4). Phase B (D5) and Phase C (D6) are human gates; this ADR only frames
  them, and NetworkCompact's shape is a proposal for Phase B, not settled
  here.
- Date: 2026-11-11
- Phase: Phase A sits with PLAN_1.0 Phase 6 (personal mode, R4). Phases B
  and C sit with PLAN_1.0 Phase 7 (network readiness, R5) and gate G-NET
  (PLAN_1.0.md:304-315).
- Order: lands after ADR-008. ADR-008 takes the edge-consent activation
  version (model schema 0.1.2); this ADR takes the next patch, 0.1.3. D8
  depends on ADR-008's Confirmed status, and D4 depends on the template
  `subject` field ADR-008 introduces.
- Drivers: R4 (per-attribute sharing across circles, including network), R5
  (network-ready, not networked), R10 (tiering), I2 (permission checks live
  only in cn-perm), I6 (provenance and tier on everything), I7 (versioned
  persisted formats), ADR-003 A-B4 (hidden equals absent), TSDF standard
  section 5.1 (consent from both the governing body and the individual);
  rulings D-015 (forgeable viewer contexts), D-030 (the intake form is the
  consent instrument), D-032 (tier wording is a human choice), D-034 (all
  pilot data enters at T1; ATNI Climate is the tier authority), ADR-005 D7
  (approved remote entries land unowned), D-106 (consent statement v3).
- Anchors: code is cited by file and function on main at 7c45afa. Line
  numbers are left out because main moved twice during review.
- This is the "future network ADR" that docs/blueprints/cn-perm.md,
  core/crates/cn-perm/src/viewer.rs and docs/PROJECT_PLAN.md:258 defer to.
  ADR-006 stays reserved for dedup (PLAN_1.0 P5.6, PLAN_1.0.md:860).

## Context

### What the code does today

- Viewer contexts are `Anonymous` and `Person { person }` only
  (cn-perm viewer.rs, `ViewerContext`).
- `admissible_circle` (viewer.rs) resolves Anonymous to Public, the owner to
  Private, a person holding an active TrustGrant from the owner to Trusted,
  and any active member of this group to Group. Every other person falls
  through to Network.
- Viewers are caller-asserted. The API parses whatever viewer JSON it is
  handed (cn-api wire.rs, `parse_viewer`), and D-015 records this as
  "forgeable viewer contexts". So in practice Network means anyone who
  supplies a UUID.
- The tier ceilings are T0 Public, T1 Network, T2 Group, T3 Private
  (cn-model tier.rs). A value is visible when the viewer's circle is at or
  inside `min(visibility, tier ceiling)` (cn-perm rules.rs,
  `required_circle` and `value_visible`).
- TrustGrant has a grantor, grantee, scope, `granted_at`, `revoked_at`,
  provenance and schema version. It has no purpose and no expiry
  (cn-model trust.rs). Its `revoked_at` is set once and absorbs later
  writes (cn-store fold.rs, `revoke_trust_grant`).
- There are no agreement records and no bridge records anywhere in
  cn-model, and no op kind that creates one (cn-store op.rs, `OpKind`).
- Loaded logs fold without authorization. `fold()` sorts and applies every
  op with no Authorizer (cn-store fold.rs); only the submit path runs the
  permission check.
- The core reads no clock. `Timestamp` is "injected by callers - the core
  never reads a system clock" (cn-model time.rs). `project()` takes no
  `now`, so nothing on the read path can expire. cn-api `submit_ops`
  receives `now_ms` and discards it (`_now_ms`).
- `project()` takes one `GroupState` (cn-perm projection.rs). An op for
  another group is quarantined (fold.rs), so one log can never hold another
  group's act.
- Edges carry no owner: `edge_visible` passes `None` (projection.rs).
- The projection and graph-index caches key on `viewer_cache_key`, which is
  the person, the active trust grant ids, the active role names and the
  template version (viewer.rs). It shares `fingerprint_input` with
  `viewer_fingerprint`, which every Projection carries, and
  `cache_key_survives_fingerprint_collision` (cn-perm tests/blueprint.rs)
  pins a CRC32 collision on that input format. `index_for` (cn-api
  session.rs) builds the GraphIndex from the display projection, and
  search, measures, paths, continuance and introductions all share it.
  There is no time and no use in the key.
- Projections omit provenance envelopes and tiers by construction
  (schemas/op-log.schema.json `$defs.projection`, which is
  `additionalProperties: false`; cn-perm projection.rs). Custody depth is
  governance-only (`entity_detail_metadata`, D-033).
- The template schema has no marker for which entity kinds describe a
  person (schemas/group-template.schema.json: a kind has `id`, `label`,
  `shape`, `color_role` and `attributes`), so the core cannot tell a person
  record from a committee. Nor does any attribute say that it names other
  parties or an affiliation. The ATNI template's person kind has
  `connections`, `committee_memberships`, `organization_affiliations` and
  `origins` (tags) and `tribe` (text); relatives-demo has `nation_or_org`.
  Attribute ids are shared across kinds: `display_name` is on person,
  committee and organization in the ATNI template.
- Governance can replace any owned attribute instance through AttributeSet
  (`authorize_owner_or_member` admits governance, and `set_attribute` in
  fold.rs inserts the caller's instance whole, provenance included), and
  can TierSet any value to any tier (`authorize_tier`).
- The app tells users that T2 means "Shared within this group only"
  (app/src/ui/format.ts, `TIER_PLAIN`); D-032 made that wording a human
  choice.
- The record mirror (cn-perm mirror.rs, landed in c75f904) shows an owner
  what each audience receives of the entities they own.

The pilot consent text makes the gap concrete. All pilot entries enter at T1
(D-034). The consent statement the human wrote says (D-106,
docs/design/intake-consent-text-2026-09-14-v3.md):

> Everything entered here is designated Tier 1 of the Tiered Sovereign Data
> Framework; shared within the network and governed by the community it
> belongs to.

In the code, "the network" for a T1 value whose own circle is Network or
Public is any person who is not a member, which is anyone who types a UUID.

### What TSDF asks for

The TSDF Standard v1.0 (CC-BY-SA 4.0), section 3.1, T1:

> **Definition:** Data shared among members of a trusted Indigenous network
> under reciprocal protocols.

> Access is governed by mutual agreements and a shared understanding of
> responsibility and respect, creating a secure space for InterTribal
> relations.

T1's critical principle is reciprocity, "the foundational Indigenous value
that relationships are mutual and ongoing, not transactional", and "Network
membership itself implies shared responsibilities." T1 is distinguished
from T2 as "governed by standing network protocols".

Section 3.1, T2:

> **Definition:** Data shared with specific external partners through
> explicit, documented agreements that define permitted uses and
> obligations.

> **Critical Principle:** T2 operationalizes **Free, Prior, and Informed
> Consent (FPIC)** as an ongoing relationship, not a one-time transaction.
> Consent can be withdrawn, and terms must be transparently documented.

T2's partner categories include "Other Tribal Nations (when formal
agreement preferred over T1 protocols)" and "Federal agencies (subject to
FOIA considerations)". On T3, the same section says "The tier restricts
external access; internal governance determines internal access."

Section 5.1 says "Consent must be obtained from both" "The appropriate
Indigenous governing body" and "Individual knowledge holders", and "Clear
mechanisms must be in place to audit data access and use, ensuring
compliance with tier requirements and associated agreements." The OCAP
table gives T2 Possession as "Access-not-copies where possible".

The TSDF Tier Classification Decision Guide gives the access checks (quoted
with the decision tree's line wraps joined):

- T1: "Verify: Is requester a member of the authorized Indigenous network
  with standing reciprocal protocols?" Yes: "T1 access granted". No:
  "Access denied (T2 agreement required)".
- T2: "Verify: Is there an active, documented agreement that explicitly
  covers this data and this requester?" Yes: "T2 access per agreement
  terms". No: "Access denied (new agreement required)".
- T3: "No external access. Period."

The T2 classification record requires `auto_upgrade_on_expiry: true  # REQUIRED`,
and the guide's tier-change table says the T2 to T3 change "Auto-triggers
on agreement expiry". For derived data: "Aggregation does not automatically
reduce tier", "If any source data is T3, derived product is T3", and "AI/ML
model outputs trained on Tribal data retain source tier restrictions". The
guide allows that "Statistical products may be lower tier if truly
de-identified", and warns against "Conflating de-identification with tier
reduction".

The TSDF Federated Indigenous Data Protocol v1.0 (Apache 2.0), section 4.1,
defines the standard roles. The two that matter here:

```yaml
  - NETWORK_MEMBER:
      tier_access: [T0, T1]
      permissions: [read, create, update]
      scope: own_nation + network_shared

  - RESEARCH_PARTNER:
      tier_access: [T0, T2]
      permissions: [read]
      scope: agreement_defined
      agreement_required: true
```

Section 4.2 requires "a **built-in, non-bypassable filter** that
automatically enforces tier rules on every query", and gives the query
behavior by user type (transcribed; check marks written as yes and no):

| User type | T0 | T1 | T2 | T3 |
|---|---|---|---|---|
| Unauthenticated | yes | no | no | no |
| Network member | yes | yes (own network) | no | no |
| T2 partner | yes | no | yes (agreement scope) | no |
| Nation admin | yes | yes | yes (own nation) | yes (own nation) |

A network member sees T1 of its own network but not T2. A T2 partner sees T2
within agreement scope but not T1. The same viewers in the code today
(tier ceilings only; the value's own circle narrows further):

| Code viewer today | T0 | T1 | T2 | T3 |
|---|---|---|---|---|
| Anonymous (Public) | yes | no | no | no |
| Any non-member person (Network) | yes | yes (any group) | no | no |
| A partner under an agreement | no such path | | | |
| Member (Group) | yes | yes | yes | no |
| Owner (Private) | yes | yes | yes | yes (own records, never exported) |

Section 2.1 requires that "All T2 (Negotiated) and T3 (Sovereign) data must
be encrypted using keys that are" generated, managed and controlled by the
sovereign entity, with T2 agreement keys "Derived per-agreement,
time-limited". Section 2.3 says "Data from each tier carries explicit,
**machine-readable restrictions** on use for training AI/ML models", with
training, inference and embedding each marked "Per agreement only" at T2,
and asks to "Include `ai_ml_restriction` field in all data records".
Section 4.4 makes T2 partner credentials expire with the agreement
(`auto_revoke_on_expiry: true       # REQUIRED`) and lists "Consent
withdrawal" among the revocation triggers.

The isdGRAPH Schema Reference v1.0 defines an Agreement node ("T2
Governance") whose required fields are `"agreement_type", "title",
"parties", "effective_date", "expiration_date"`, with a `status` enum
`["draft", "active", "expired", "terminated", "renewed"]`, a `scope` with
`permitted_uses` and `prohibited_uses`, `ai_ml_terms`, `benefit_sharing`
(including `data_return_requirements` and `publication_requirements`), and
`auto_upgrade_on_expiry` described as "Data auto-upgrades to T3 when
agreement expires". It also defines a Consent Record node with
`consent_type` `["individual", "collective", "delegated", "blanket"]`, a
grantor and grantee, `expires_at` ("null = indefinite") and `revoked_at`
("null = active"). Its agreement-scoped query example names one agreement
(`$agreement_id`), filters `data.tsdf_tier IN ['T0', 'T2']` and
`datetime() < datetime(a.expiration_date)`: expiry is evaluated per
agreement against the current time on every read.

Stated plainly: **the code's linear ceilings keep the order of the tiers but
flatten their mechanisms. T1 has a ceiling but no membership, and T2 has a
ceiling but no agreement path, so TSDF's negotiated sharing cannot happen at
all.** The only way to show a T2 value to an outside partner today is to
make them a member of the group, which shows them every Group value.

### What the research adds

Each item names the digest finding. Where the design step is the digest's
own implication note rather than the finding, the item says so.

- R2-F6: a 2024 data roundtable of Indigenous Guardians programs named
  data-sharing agreements and MOUs as the practical way to put data
  sovereignty into effect. A bridge should carry a machine-readable
  agreement: scope, purpose, expiry, revocation.
- R8-F7: access as request, then a scoped grant valid between an issue date
  and an expiry date, revoked by appending another op. Expiry by default
  suits pilot-era bridges.
- R8-F6: the Kantara Consent Receipt records the parties, the purpose and
  the categories of personal data. The digest's note asks that every grant
  produce a receipt the person keeps.
- R7-F6: the Indigenous Navigator lets communities make published data
  private again. Serving as a bridge should be a revocable event in the op
  log, not a field set once.
- R8-F5: Indigenous genomics scholars say dynamic consent must sit inside
  wider community control, and many communities want consent asked again
  for each use. The two-keys design in D2 rests on TSDF standard section
  5.1 (consent from both the governing body and the individual); the
  digest's note on R8-F5 draws the same two keys.
- R3-F10: locally designed governance takes priority, with CARE as the
  floor. A bridge must apply the stricter of the two Nations' protocols, and
  the rules apply to data already in the log.
- R8-F9: contextual integrity specifies a flow by "sender, recipient,
  subject, information type, and transmission principle". A circle names
  only the recipient.
- R7-F1: TigerSwan built watch lists and dossiers on Indigenous activists
  and environmental groups and at times shared them with law enforcement.
  The digest's note reads this as a warning that a graph of who is
  connected to whom is what a hostile party wants, so edges should never be
  exportable in bulk. That step is the digest's inference, adopted here as
  design (D8).
- R7-F2: TigerSwan then pointed the same work at other movements. The
  digest's note concludes that every grant should name a purpose and
  expire, and that the system should be able to prove which projections
  were issued to whom (D9).
- R7-F3 and R7-F4/R7-F5: hidden funders made participatory mapping
  extractive, and in small populations both re-identification and privacy
  noise do harm, so suppression should depend on the viewer (D1 partner
  fields, D8 small-cell rule).
- R6-F12 and R3-F8: derived products (models, embeddings, maps, briefs)
  should carry the use conditions of their sources. Te Hiku Media's
  Kaitiakitanga License is their own text; it is a pattern to learn from,
  not text to reuse.
- R1-F2 and R1-F5: the USGS/BIA Tribal Community Resilience Liaison network
  ended in July 2026 and NOAA terminated the Northwest Climate Resilience
  Collaborative in 2025. Relationship knowledge that lived with grantees
  needs to live in Tribe-held records, and moving it between graphs needs
  terms the Tribes set.

## Decision

### D1. Permission records in cn-model

Bridging relations are permission primitives, so they are core types with
fixed shapes, not template kinds (ADR-001 D5). Each is modeled on TrustGrant
(cn-model trust.rs): a UUIDv7 id, a provenance envelope, timestamps, a
schema version, and, unlike TrustGrant, a `tier` of its own because the
record is itself data (I6). Revocation is a separate `revoked_at` that is
set once and absorbs every later write, as `revoke_trust_grant` does, so no
reordering or merge can un-revoke (D2). Default record tiers: agreement T2,
compact T1, bridge consent T3. Terms are immutable once created: changing
terms means revoking and creating a new record, so a value always points at
fixed terms.

**SharingAgreement (TSDF T2).**

- `grantor: GroupId` - the group whose log holds the agreement.
- `grantees: Vec<PersonId>` - persons only in Phase A. These are the access
  holders.
- `partner: { organization, party_type, partnership_level, funder }` -
  descriptive data about who the grantees act for. `party_type` is one of
  `TribalNation`, `TribalOrganization`, `FederalAgency`, `StateAgency`,
  `University`, `Ngo`, `Other`. `FederalAgency` and `StateAgency` put a
  public-records warning (FOIA or the state's law) on the creation and
  consent screens (standard section 3.1, T2 partner categories; R7-F3).
- `governance_decision_ref` (required text: the resolution or meeting
  record that approved the agreement) and `document_ref` (optional: the
  signed agreement), so the record is the "explicit, documented agreement"
  T2 asks for.
- `scope` - all lists, no wildcards, empty admits nothing:
  - `kinds: Vec<KindId>` - entity kinds whose presence may cross.
  - `attrs: Vec<(KindId, AttrId)>` - attributes named with their kind. An
    attribute id shared across kinds is in scope only for the kinds listed
    with it.
  - `edge_kinds: Vec<EdgeKindId>`.
  - `tiers: BTreeSet<SensitivityTier>` - default `{T2}`. T3 is rejected at
    construction. Including T1 requires `tier_authority_ref` (D4).
- `tier_authority_ref: Option<Text>` - the recorded decision of the tier
  authority (ATNI Climate for the pilot, D-034). Required when `tiers`
  includes T1 or `attrs` names an identity attribute (D4).
- `purpose: { code, text }` - an open-vocabulary code plus plain text.
- `permitted_uses`, `prohibited_uses: Vec<UseCode>` - a closed core enum:
  `View`, `Export`, `Routing`, `NetworkAnalysis`, `Publication`,
  `Commercial`, `AiTraining`, `AiInference`, `AiEmbedding`. Anything not
  permitted is prohibited, a prohibition wins over a permission, and `View`
  is implied by admission. The core enforces `View`, `Export`, `Routing`
  and `NetworkAnalysis` (D3, D8). The other five are terms the core
  carries on every agreement-served value and that only honest tools honor
  (D7). In Phase A, construction rejects an agreement that permits
  `Routing` or `NetworkAnalysis` until the tier authority sets a
  small-cell rule (D8, open question 12).
- `return_obligations: Vec<{ kind: DataReturn | Credit | Notice, text,
  due: Option<Timestamp> }>`. An agreement that permits `Export` must carry
  a `DataReturn` obligation due no later than `valid_until` (standard
  section 6.2, return or destroy per agreement terms).
- `valid_from: Timestamp` and a required `valid_until: Timestamp`, with
  `valid_until > valid_from` checked at construction.
- `export_key` - a per-agreement key for export pseudonyms (D8). It is part
  of the record but never projected to grantees.
- Lifecycle slots: `created_by: PersonId`; `accepted: BTreeMap<PersonId,
  Timestamp>` (one entry per grantee who accepted); `ratified_by:
  Option<MembershipId>`, `ratified_at`; `revoked_at`. Status is derived,
  never stored: Proposed until ratified, Ratified after, Revoked once
  `revoked_at` is set, and "expired" is evaluated against the injected
  `now` (D3).

**NetworkCompact (TSDF T1) - a proposal for Phase B, not settled here.**

- `members: Vec<{ group: GroupId, party_type, eligibility_ref }>` - the
  party groups, each with its party type and a reference to whoever attests
  that it is part of a trusted Indigenous network (open question 11).
- `reciprocity` - the reciprocity protocol as data (options in D5), with
  its text and its AI/ML terms (protocol section 2.3's T1 row requires
  network approval for training and embedding).
- Per-group ratification: `{ group, ratified_by, ratified_at, left_at }`.
  Each party records the compact and its own ratification in its own log,
  because an op for another group is quarantined (fold.rs). A compact is
  active between two groups only when both logs say so.
- `valid_from: Timestamp`, `valid_until: Option<Timestamp>` (a standing
  protocol may be open-ended).

**BridgeConsent.**

- `owner: PersonId` - the person whose records may cross.
- `under: Agreement(SharingAgreementId) | Compact(NetworkCompactId)`.
- `items: Vec<ConsentItem>`, where an item is `Attr { kind, attr }` or
  `EdgeKind { kind }`, with an optional `entities: Vec<EntityId>` that
  narrows it to named records of the owner. With no list, the item covers
  every record of that kind the owner owns.
- Each item also records `at_most: Circle` and `at_least: SensitivityTier`:
  the value's visibility and effective tier when consent was given. The
  client fills them from current state, and the submit path rejects bounds
  wider than current state. At read time an item covers a value only while
  the value's current visibility is no wider and its current tier no less
  restrictive than the item records (D4). Consent to any attribute of a
  record implies that record's presence, under the bounds of its presence
  visibility and entity tier at consent time.
- `granted_at`, an optional `expires_at` ("null = indefinite", as in the
  isdGRAPH Consent Record), `revoked_at` (set once, absorbing). A consent
  never outlives the agreement or compact it names. Asking again for each
  use (R8-F5) is owed to a later slice.

Mapping, so the records stay TSDF-legible:

| TSDF / isdGRAPH | Here |
|---|---|
| Agreement `parties` (`party_type`, `role`) | grantor group; `partner` (organization, party type) as descriptive data; `grantees` as the persons who hold access |
| Agreement `agreement_type`, `title`, documentation | `purpose`, `governance_decision_ref`, `document_ref` |
| Agreement `effective_date`, `expiration_date` (required) | `valid_from`, `valid_until` (required) |
| Agreement `status` | derived: Proposed, Ratified, Revoked; expired is evaluated at `now`; a renewal is a new agreement |
| Agreement `scope.data_elements`, `permitted_uses`, `prohibited_uses` | `scope`, use codes |
| Agreement `ai_ml_terms`; protocol 2.3 `ai_ml_restriction` | `AiTraining`, `AiInference`, `AiEmbedding`, and the D7 per-tier table |
| Agreement `benefit_sharing.data_return_requirements`, `publication_requirements` | `return_obligations` |
| `auto_upgrade_on_expiry: true` (required) | an analog, not an equivalent: expiry ends admission (D3), and every agreement-served value and derived product carries `tier_after_expiry: T3` (D7) |
| T1 record `network_scope`, `reciprocity_protocol_reference` | compact `members`, `reciprocity` (held as data; Phase B proposal) |
| Consent Record (`consent_type: individual`, `expires_at`) | BridgeConsent |

Read as contextual integrity (R8-F9): sender is the grantor group, recipient
is the grantees, subject and information type are the owner and the scope,
and the transmission principle is the purpose, the use codes, the validity
window and the consent.

Phase A implements SharingAgreement and BridgeConsent (naming agreements).
NetworkCompact lands with Phase B, because before D5 it would have no
effect.

### D2. Ops and authority

- `AgreementCreate`, `AgreementRatify`, `AgreementRevoke`: governance acts
  of the grantor group, authorized like `require_governance` (cn-perm
  authz.rs). The ratifier must be a governance member other than
  `created_by`, so no single member both drafts and ratifies. A k-of-n
  threshold is a later decision (open question 2; panel idea
  sovereign-architect-6, Council Seal).
- `AgreementAccept`: each grantee person accepts the terms. Authorized only
  when the submitter is in `grantees`; it is the one write a non-member
  grantee may make. By default admission needs the grantee's acceptance as
  well as ratification (open question 8 asks whether a grantor may waive
  it). The agreement record, terms included and `export_key` excluded,
  projects to its grantees so they can read what binds them.
- Revocation depends only on state. A record whose `revoked_at` is set
  admits nothing for any `now`, and `AgreementRevoke` is accepted at any
  time, including after `valid_until`. That makes revocation the grantor's
  stop that no caller-chosen `now` can undo. An `AgreementRatify` that
  sorts after a revoke is a no-op in the fold. A second revoke is rejected
  on the submit path.
- An agreement binds the grantor group's governance, not the individual
  ratifier. The submit path checks that the ratifier is active governance
  when ratifying. A ratifier's later departure does not end the agreement;
  governance revokes it explicitly. Membership lifecycle is current state
  only (`active_roles_for`), so the read-time check (below) can confirm
  only that the ratifier's governance membership exists in state.
- `CompactCreate`, `CompactRatify`, `CompactLeave` (Phase B): each party
  group's governance acts for its own ratification and its own leaving.
- `BridgeConsentCreate`, `BridgeConsentRevoke`: owner-only, authorized like
  `authorize_grant_create` and `authorize_grant_revoke` (authz.rs). No
  governance act can create, block or reverse a person's consent or
  revocation.
- Two keys (TSDF standard section 5.1: consent "from both" the governing
  body and the individual): a person's records cross only when both exist,
  an active group act (agreement or compact) and that person's own active
  consent naming it. Neither key alone admits anything.
- Declining is the absence of a consent; it needs no op. BridgeConsent
  records are T3 and project to their owner only. The record itself reaches
  no other viewer, but the fact of consent is not secret, and the consent
  screen says so:
  - The grantee sees what crossed. When an owner's in-scope attributes are
    Group-visible, the grantee sees exactly the set of owners who
    consented.
  - When an owned entity's presence is visible at the grantee's base circle
    (Network or Public), an in-scope attribute missing for it, while other
    entities show that attribute annotated as agreement-admitted, tells the
    grantee that this owner withheld it. Owners who would rather stay
    unseen can set presence to Group.
  - Whether grantor governance may audit who consented is open question 10.
    The interim default follows TSDF standard section 5.1 (audit) and the
    T3 text ("internal governance determines internal access"): governance
    may audit which records crossed to which grantee under which agreement.
    That audit arrives with receipts (D9); Phase A adds no governance read
    surface over consents.
  - Whoever holds the raw op log can read the ops. Encrypting circles is
    not in this ADR (D6).
- Quarantined BridgeConsent ops appear in governance's store report as
  counts only, with no owner, item or time, until open question 10 is
  answered. Today `redact_report` returns the full report to governance
  (cn-perm reports.rs), so this is a new redaction rule.
- Read-time validity (ADR-002 A-B2): because loaded logs fold without
  authorization, cn-perm re-checks at projection time that an agreement's
  ratifier holds a governance membership of the grantor group, that the
  ratifier differs from `created_by`, and that a consent's owner owns the
  records it covers. A record that fails is inert, not deleted. This keeps
  the rules consistent; it is not a defense against a forged log, which
  only D6 provides.
- cn-store gains the ops (op.rs `OpKind`), `apply_*` arms following
  `apply_trust_grant` (fold.rs), and FieldKeys.

### D3. Time- and use-aware projection

- `project` and `export_projection` gain two required parameters: `now:
  Timestamp` and `use: UseCode`. Making them required lets the compiler
  list every caller. On main at 7c45afa those include: cn-api projection,
  detail, search, paths, measures, export and `export_snapshot_impl`; the
  redaction inside `submit_ops_impl`; `redact_report` through its subject
  index; `intake_near_duplicates` and `intake_validate_record` (through
  `projection_for`); `graph_continuance_impl`,
  `suggest_introductions_impl` and `record_mirror_impl`; the cn-wasm entry
  points; the CLI export (core/cli/src/export.rs); the snapshot embed
  plugin (app/scripts/embed-snapshot-data.ts, which uses a fixed build
  clock); and the app worker (app/src/wasm/worker.ts). The core still
  reads no clock.
- Write-path checks keep the agreement-free predicate (`visible_for_owner`
  and `entity_projected` as they are): `authorize_edge_create`,
  `custody_target_visible`, `authorize_story_update`, and the report
  redaction. Agreement admission gives a member nothing new, because
  `min(base, Group)` is never wider than Group, and a non-member grantee
  fails `require_member` first. So the `Authorizer` needs no `now`, and a
  grantee sees less in reports, never more.
- Uses by entry point: display projection, entity detail and search take
  `View`; paths and neighborhoods take `Routing`; measures, continuance and
  introductions take `NetworkAnalysis`; exports take `Export`. Intake
  near-duplicates and validation stay agreement-free.
- An agreement is active for a grantee at `now` when it is ratified, that
  grantee has accepted (unless waived, open question 8), its `revoked_at`
  is unset, and `valid_from <= now < valid_until`. A compact is active
  under the same rule without acceptance, and without an upper bound when
  it has no `valid_until`. A consent is active while its `revoked_at` is unset and
  `now` is before its `expires_at`, if any.
- Cache keys. Both cn-api caches key on `(viewer_cache_key, use)`, and the
  graph index is built from cn-perm's projection for that use, never from
  the display projection. For Person viewers `viewer_cache_key` appends the
  sorted ids of the agreements active at `now` that name this viewer, and
  an expiry epoch: the earliest `valid_from`, `valid_until` or consent
  `expires_at` after `now` among records that name this viewer or cover a
  value admitted to them, or `-`. A projection or index built before an
  expiry is therefore never served after it, and one built for `Routing`
  is never served for `NetworkAnalysis`. The extension goes into
  `viewer_cache_key` only. `fingerprint_input` and the exported
  `viewer_fingerprint` stay as they are, so
  `cache_key_survives_fingerprint_collision` keeps its pinned pair. The
  Anonymous key is unchanged.
- `as_of`. cn-api stamps `as_of: now` on each returned projection and
  export after the cache lookup; the cached value carries none, so a
  projection served from cache never reports the time of its first build.
- Each agreement-admitted value carries the ids of every agreement that
  admitted it and the earliest of their `valid_until` (D7).
- Formats: `schemas/op-log.schema.json` `$defs.projection` gains `as_of`
  and the per-value annotations, cn-api `BOUNDARY_VERSION` bumps, and the
  app DTOs follow (I7).
- `now` is caller-supplied, like the viewer. cn-wasm already passes
  `now_ms` into writes and gains it on projection, detail, query and export
  entry points; cn-api passes it through instead of discarding it. A
  caller can assert a false `now` exactly as it can assert a false viewer
  (D-015). Key-bound viewers (D6) do not make a caller's clock
  trustworthy, so expiry is honored only by honest callers until expiring
  per-agreement keys exist (TSDF protocol section 2.1). Revocation does not
  depend on `now`.

### D4. Phase A: agreement admission

A viewer named as a grantee in an agreement active at `now` receives, for a
use the agreement permits, the in-scope and covered values of an effective
tier in the agreement's `tiers`, as if at the Group circle. Admission is per
`(agreement, use)`. Never T3. Everything else resolves exactly as today.

```
base = admissible_circle(state, viewer, owner)          // viewer.rs, unchanged
admitting = { A : active(A, now) and viewer in A.grantees and permits(A, use)
                  and in_scope(A, subject) and eff_tier in A.scope.tiers
                  and covered(A, subject) }
circle = if admitting is empty { base } else { min(base, Group) }
                                                        // Private < Trusted < Group < Network < Public
visible = value_visible(circle, visibility, eff_tier)    // rules.rs, unchanged
```

- In scope. Entity presence: the entity's kind is in `scope.kinds`. An
  attribute: its entity's kind is in `scope.kinds` and `(kind, attr)` is in
  `scope.attrs`. An edge: its kind is in `scope.edge_kinds`, both endpoints
  are already projected for this viewer and use, and the edge's own rule
  passes. Stories never cross in Phase A.
- Relation-bearing attributes never cross. The template marks an attribute
  `relational: true` when its value names other parties or ties (for
  example `connections`, `committee_memberships`,
  `organization_affiliations`, `origins`). cn-perm never admits one under
  an agreement, and construction rejects a scope that lists one. Once
  two-sided relations exist, admitting them needs `Routing` and every
  named party's consent; that is a later decision.
- Identity and affiliation attributes are out of scope by default. The
  template marks an attribute `identity: true` (for example `tribe`,
  `nation_or_org`). The tier-decision guide puts enrollment records at T3
  as "Highly sensitive, sovereignty issue", and these sit next to them. A
  scope may list one only with `tier_authority_ref`, and it then still
  needs the owner's consent item.
- T1 is out of scope by default (`tiers` defaults to `{T2}`). Because every
  pilot value is T1 (D-034), an agreement on pilot data admits nothing
  until the tier authority records a decision to include T1. The basis for
  allowing it at all is the decision tree's T1 branch, "Access denied (T2
  agreement required)", which names an agreement as the path. A T1 value
  shared that way is governed as T2 for that agreement, and that
  classification is the tier authority's act (D-034), so it needs
  `tier_authority_ref`.
- Covered, owned records: the owner holds an active BridgeConsent under A
  with an item that matches the value: `Attr { kind, attr }` for an
  attribute (and its entity, if the item names entities), or `EdgeKind`
  for an edge, for each owned endpoint. Edges carry no owner, so endpoint
  owners are the only persons whose consent can attach. The item covers
  the value only within its recorded bounds (D1). A governance AttributeSet
  that widens an owned value's visibility, or a governance TierSet that
  lowers its tier, therefore voids that owner's consent for it until the
  owner consents again. This is a read-time check in cn-perm.
- Covered, unowned records: only collective records are covered by the
  group's ratified agreement alone. ADR-008 adds a template field `subject:
  person | collective` per kind; a kind that does not declare one counts
  as `person`.
  - An unowned record of a person kind is never covered in Phase A, with or
    without the ADR-005 intake block. Section 5.1 needs both keys, and
    there is no owner to give the second.
  - An entity whose own provenance carries the ADR-005 D5 intake block is
    never covered by the group act alone, whatever its kind and whatever
    the provenance of an individual value says. `set_attribute` replaces
    the whole instance, provenance included, so a value's own provenance
    cannot be trusted to keep the block. The respondent's consent reads
    "Everything you share here stays here." and covers use "for this
    demonstration" (D-106). Such a record crosses only after it has an
    owner who consents, which needs an owner-claim path that does not
    exist yet (ADR-005 D7).
  - An edge crosses only when each endpoint is either covered by its
    owner's consent or is an unowned collective record that carries no
    intake block. An in-app edge to an intake person or to an unowned
    person record never crosses.
- T3 never crosses: T3's ceiling is Private (tier.rs), Group is never
  inside Private, so `value_visible` refuses it, and `tiers` cannot hold
  T3. Attribute-level tighten-only overrides (ADR-001 A-B3) apply
  unchanged. A governance TierSet that lowers an owned value out of T3
  voids that owner's consent (above). One that lowers an unowned
  collective value out of T3 lets it cross under the group act, which is a
  governance decision about the group's own record.
- Unchanged default: for every viewer named in no active agreement, the
  sets of visible entities, attributes and edges are unchanged. The only
  difference in their results is the added `as_of` field (D3).
  `tier_ceiling_cells` (cn-perm tests/blueprint.rs) keeps its assertions;
  its call site gains `now` and `use`.
- Divergence from protocol section 4.2, stated: the table shows a T2
  partner without T1. By default Phase A matches it, because `tiers` is
  `{T2}`. When the tier authority includes T1, Phase A admits in-scope T1
  for that agreement. Because the lattice is linear, Group admission
  includes the Network ceiling, and until Phase B every person already
  reaches Network, so this adds only in-scope T1 values whose own circle
  is Group. Open question 7 asks whether Phase B removes even that.
- Precondition for use outside tests: the human answers open question 5,
  and reviewed T2 wording replaces "Shared within this group only" in
  `TIER_PLAIN` (D-032; community-facing text needs human review). Until
  then Phase A sits behind a build flag (a cargo feature, off in the app
  build). With the flag off, cn-perm's admission path is compiled out and
  the submit path refuses the agreement and consent ops. Phase A runs only
  in tests and in fixtures that are not shown with tier prose.
- Why this is director work after acceptance: it only widens, only for
  named persons, only through records that two of the grantor group's
  governance members and the record owners create, and it narrows nothing.
  It is TSDF T2 made executable.
- Trust scope, stated: while viewers are caller-asserted (D-015), an
  agreement admits whoever asserts a grantee's PersonId. That is the same
  class as asserting the owner's PersonId today, but it reaches outsiders.
  Phase A is built and tested on synthetic fixtures only. Serving
  agreement-admitted real data to a real outside partner is sharing outside
  the machine. It stays behind the existing human gates, and it waits for
  receipts (D9) and key-bound viewer contexts (D6).

### D5. Phase B: Network means compact members (human gate)

- The Network circle for a value in group G admits a person only when that
  person holds an active membership in a group H, and H and G are ratified,
  not-left parties to a NetworkCompact active at `now`. Any other
  non-member, non-trusted person resolves to Public, not Network (TSDF
  T1: "Access denied (T2 agreement required)").
- Owned values still need the owner's BridgeConsent naming the compact (D2,
  two keys). Visibility choices made under today's meaning of Network were
  not choices about a specific compact (open question 9).
- The reciprocity rule is compact data. Options for the human:
  (a) membership alone: every active member of a party group is Network;
  (b) contribution-gated: a party group's members are Network only while
  that group itself shares at least one kind into the compact, so no party
  only reads; (c) a per-compact protocol: the compact picks its rule from a
  closed set the core can evaluate, with text for the rest.
- Party eligibility. TSDF T1 is a "trusted Indigenous network", and T2
  "governs collaboration with non-Indigenous entities". A compact with a
  non-Indigenous party group would put that group's members in Network
  (T1) without a T2 agreement. Who attests a party's eligibility is a
  sovereignty and identity question (open question 11). H's governance can
  also grow H's membership, and with it G's Network audience, without G
  knowing; Phase B needs a rule for G to review party rosters (open
  question 11).
- Evaluating H's membership while projecting G needs H's state.
  `project()` takes one GroupState; cn-api holds several groups. Any
  multi-state admission lives in cn-perm, never cn-api (I2). G's cache key
  includes the revision of every compact-party group consulted, or G's
  caches clear whenever any party session changes; today caches clear only
  on the session's own revision (cn-api session.rs, `bump_revision`). H's
  log was folded without authorization, so the narrowing half of Phase B is
  safe on its own, while the widening half rests on unverified logs until
  D6.
- Why it is a human gate: it changes tested behavior (`tier_ceiling_cells`),
  it changes what T1 means for every pilot record (D-034, ATNI Climate is
  the tier authority), and PROJECT_PLAN.md:258 names the network ADR as
  human-gated. It is also what makes the D-106 sentence "shared within the
  network and governed by the community it belongs to" true.

### D6. Phase C: identity, verified logs, transport (human gates G-NET and identity)

Out of scope here, and gated by G-NET (PLAN_1.0.md:304-315) and the
identity ADR (P7.4, D-027): cross-group identity (that a PersonId in another
graph is the same person); signed ops and verify-before-fold; key-bound
viewer contexts (D-015); transport (cn-sync is a placeholder; ADR-002 A-B8);
circle encryption and TSDF 2.1's per-agreement time-limited keys, which are
also what would make expiry hold against a dishonest local holder; portable
grants. This ADR defines only what a projection may admit. It chooses no
protocol, identity standard or token format.

### D7. Stricter of two, and what travels with a value

- What crosses with an agreement-served value, and nothing more: the
  value; its effective tier; its provenance origin class only (authored,
  ingested or derived); the ids of every admitting agreement; the merged
  use terms; the earliest `valid_until`; and `tier_after_expiry: T3`.
  Never the provenance envelope, custody, `recorded_by` or
  `responsible_human` ids, or the ADR-005 intake block. Projection rule 7
  (projections omit provenance envelopes) stands. The envelope stays in
  the grantor's store (I6).
- The binding rule is the stricter of the source and receiving rules: the
  more restrictive tier (`SensitivityTier::most_restrictive`, tier.rs),
  the union of prohibited uses, the intersection of permitted uses, the
  earliest `valid_until` (R3-F10). When several agreements admit one value,
  it carries all of them and their terms merge by the same rule. In Phase A
  the receiving side is any tool reading the export. In Phase C it is the
  receiving graph's importer.
- `tier_after_expiry: T3` is how the REQUIRED `auto_upgrade_on_expiry`
  reaches a copy. In the live projection only admission ends at expiry; the
  grantor's own viewers see no tier change, because this code's T2 is the
  Group ceiling, not TSDF's partner tier. That is a stated divergence, like
  the protocol 4.2 one in D4. Data co-produced with the partner is out of
  scope in Phase A (open question 13).
- Derived products (exports, graph measures, summaries, embeddings) inherit
  the maximum tier of their inputs and the use terms of every agreement that
  admitted an input (R6-F12; the decision guide's derived-data rules). This
  ADR never lowers a tier for de-identification. The guide allows it only
  for "truly de-identified" statistical products, and it warns against
  conflating the two. The core makes no summaries or embeddings today.
- AI/ML terms. Every export carries a per-tier `ai_ml_restriction` table in
  its header (protocol section 2.3): T3 all false; T2 false unless an
  admitting agreement permits the use; T1 training and embedding false
  pending network approval, inference false. Each exported value already
  carries its tier, so each record's restriction is machine-readable, and
  agreement terms override it per value. The interactive projection carries
  no table; the planned on-device AI must read the same rule when it lands
  (owed to that slice).
- Limit, stated: once disclosed, the core cannot enforce purpose. Terms
  bind honest tools and give the grantor a record of what was agreed.

### D8. Edges, relations and exports stay home

- No export contains an edge admitted only by an agreement. Agreement
  exports carry entities and attributes only. This follows the digest's
  reading of R7-F1 (D7 context): the edge list is what a profiler wants.
- Export is a use. `export_projection` and `export_snapshot_impl` drop
  every agreement-admitted value unless an admitting agreement permits
  `Export`. No agreement permits it by default. A grantee who may only
  `View` can read admitted values live but gets none of them in any export,
  which is the closest the core comes to TSDF's "Access-not-copies where
  possible".
- Agreement exports replace entity ids with per-agreement pseudonyms that
  cn-perm derives from the entity id under the agreement's `export_key`.
  Exports held by different grantees, or under two agreements, then do not
  join on stable UUIDv7 ids. Limit: a grantee who also reads the live
  projection sees stable ids there. When a scope leaves out names but
  includes quasi-identifiers (role, specialties, an identity attribute),
  the creation screen warns that small groups make people identifiable.
- Queries run over cn-perm's projection for the declared use (D3).
  Agreement-admitted values and edges enter a `Routing` or
  `NetworkAnalysis` projection only when an admitting agreement permits
  that use. The values are dropped in cn-perm, not cn-graph (I2). In Phase
  A no agreement may permit either use until the tier authority sets a
  small-cell rule for outputs that reach grantees (R7-F4, R7-F5; open
  question 12), so grantee queries run over the grantee's base circle
  only. A query that names an agreement runs under that agreement alone,
  as the isdGRAPH agreement-scoped query does.
- `edge_kinds` defaults to empty. Listing an edge kind discloses those edges
  to the grantee's client in the live projection; the export rule above
  stops onward bulk transfer by honest tools only.
- Once ADR-008 lands, an edge may cross only when ADR-008 derives it as
  Confirmed through an explicit `Accepted` from every owned endpoint. A
  grandfathered edge (ADR-008 D5) is never an acknowledgment for crossing.

### D9. Receipts and audit (deferred out of Phase A)

Receipts are not part of Phase A. They are a precondition for serving
agreement-admitted real data to an outside partner (D4). The receipt slice
must specify, at minimum:

- A receipt op kind with a system actor, a stated responsible human, a
  tier, an authz arm, and non-disclosability under the ADR-002 A-B5 export
  gate. Receipts are generated in the core only, and `submit_ops` denies
  receipt ops from callers.
- Persistence. cn-api has no durable store. Receipts are returned in the
  read envelope for the caller to persist (native: `OpLog::append_batch`;
  browser: a named store), and the slice states what happens when the
  caller drops them.
- Revision. A receipt append must not change the revision other viewers
  see, because the revision bump is itself a channel (23aacec names it).
- Coverage. Projection, entity detail, search, queries and exports that
  serve agreement-admitted values are receipted. Agreement activation,
  expiry, revocation and inertness are recorded too (protocol section 4.3
  lists "All agreement activations/expirations").
- Lines. A line for an edge with an owned endpoint is an owned line of each
  owned endpoint. Each owner reads the lines that touch their records.
- Governance reading follows open question 10. Under the interim default,
  governance reads which records crossed to which grantee, and the owner
  was told so at consent. If the human chooses owner secrecy from
  governance instead, the slice must hide per-change headers whenever the
  served set includes owned lines (a header that changes when only owned
  lines change would show consents and revocations), apply the same rule to
  any governance mirror, redact consent and receipt ops in governance's
  store reports, and state that this departs from standard section 5.1.
- Limits: a receipt proves what an honest instance served; a hostile
  instance holding the log can skip it (D6). TSDF 4.3 says "All actions
  must be logged to an **immutable audit trail**, including denied
  queries." The slice will not meet that by itself: ordinary reads and
  denials are not logged.

## Options considered and rejected

1. **Network as all loaded groups** (members of any group loaded in the
   same instance count as Network). Rejected: which groups are loaded is a
   local accident, not a standing reciprocal protocol, and a loaded log is
   unverified, so anyone who fabricates a group log would become Network.
2. **Adopting Keyhive or Meadowcap now.** Rejected for now. Keyhive is
   pre-alpha and unaudited; its own notebook says "DO NOT use this release
   in production applications" (R5-F1). Meadowcap is Final with a Rust
   implementation and maps well onto owned and communal namespaces (R5-F3),
   but adopting it commits an identity and namespace model, which is
   G-NET's. Both stay Phase C design input.
3. **Per-person TrustGrants only.** Rejected: TrustGrant is person to
   person with no purpose, expiry or use terms (trust.rs). Trusted sits
   inside Group, so it would expose more than intended. It has no group
   key (standard section 5.1), and unowned records cannot be granted at all
   because the grantor is a person.
4. **A consent flag only** (an attribute such as "serve as bridge").
   Rejected: a flag set once is not revocable publication (R7-F6). It names
   no recipient, purpose, scope or expiry (R8-F9), carries no group key, and
   as template data it cannot be a permission primitive (ADR-001 D5).
5. **Biscuit tokens now.** Rejected for Phase A: in Phase A the grantor,
   grantee and records share one local log and are checked in place, and a
   portable token means something only with key-bound identities and a
   transport (D6). Recorded as the leading option for portable,
   offline-verifiable, attenuable grants in Phase C (R5-F11: biscuit-auth
   6.0.0 was the latest stable release when checked on 2026-11-11,
   Apache-2.0).
6. **Agreements as a template entity kind** (panel idea
   emergency-coordinator-1, step 1). Rejected for the permission path:
   permission relations are core primitives (ADR-001 D5). A template may
   still describe a compact as a community object for display.
7. **Narrowing Network in Phase A.** Rejected for Phase A: it changes
   tested behavior and the meaning of every pilot T1 record (D-034), which
   is the human's call (D5).
8. **Bare attribute ids in scope.** Rejected: attribute ids are shared
   across kinds, so listing `display_name` for one kind would admit it for
   every kind (D1 uses `(kind, attr)` pairs).

## Consequences

Positive:

- TSDF T2 becomes executable: a named partner sees exactly the in-scope
  values, for a stated purpose, until a stated date, with the owner's
  consent, the acts of two governance members and the partner's own
  acceptance of the terms.
- For honest callers, expiry takes effect on the next projection, including
  from cache. Revocation takes effect for every caller, because it does not
  depend on `now`.
- T3 stays home by the existing lattice, not by a new special case.
- Relation-bearing attributes, identity attributes, T1, exports and
  analysis uses are all closed by default, so a careless agreement admits
  little.
- Phase B has a defined shape for making "shared within the network" true.

Negative or accepted:

- Persisted-format changes under I7:
  - New record types, op kinds and FieldKeys.
  - The model schema version moves one patch past ADR-008's activation
    version (0.1.3 if ADR-008 lands at 0.1.2; cn-model lib.rs).
  - `schemas/group-template.schema.json` gains `relational` and `identity`
    per attribute; `subject` per kind comes with ADR-008.
  - The op-log projection schema, the export format and `BOUNDARY_VERSION`
    gain `as_of`, per-value agreement ids, use terms, `tier_after_expiry`
    and the export `ai_ml_restriction` table.
  - At major 0, `accepts_schema` requires the same minor and accepts any
    patch, so a 0.1.x reader does not reject a newer 0.1.x log by version.
    It fails at parse on an unknown op variant. That rests on the same
    one-machine deployment assumption as ADR-005 D5 and ADR-008 D5.
- cn-perm gets an agreement-aware admission path in a new module rather
  than growing viewer.rs (I5). `admissible_circle` and `visible_for_owner`
  need the value's subject (kind, attribute id or edge kind), not only its
  owner. cn-perm also gets `now` and `use`, use-scoped projections, export
  gating and pseudonyms, the consent-bound check, the report redaction for
  consent ops, and new authz arms.
- cn-api and cn-wasm pass `now` and `use` through every entry point listed
  in D3, and cn-api's caches change keys.
- The record mirror gains agreement and compact audiences, limited to the
  viewer's own records. A line for an edge with an owned endpoint is that
  owner's line.
- Agreement, consent and acceptance screens are community-facing text and
  need human review before use. So does the T2 wording (D4 precondition).
- Agreements can multiply into a burden for owners and governance.
- Purpose limits are honored by honest tools only, expiry by honest callers
  only, and the raw op log stays readable by whoever holds it, until D6.
- Phase A has no receipts (D9), so it proves nothing about what was served.
  That is why it stays on synthetic data.
- Phase B will break `tier_ceiling_cells` on purpose; Phase A must not.

## Test obligations

In this order:

0. **Golden captures first.** Before any change, capture the projections of
   all four fixtures for every viewer context (anonymous, a non-member, a
   member, governance, each owner) at the pre-change commit, and commit
   them as test data.
1. **I7.** Serde round trip and version checks for every new record and
   op, the template-schema additions, and the projection schema; an
   unknown op variant fails at parse.
2. **Authority matrix.** Agreement ops are governance-only; a ratifier who
   is the creator is denied; `AgreementAccept` is grantee-only; consent ops
   are owner-only. A non-member grantee is denied every member write. A
   second revoke is denied. With the build flag off, agreement and consent
   ops are refused.
3. **Convergence.** A proptest reorders and duplicates create, accept,
   ratify and revoke ops; they converge, and a ratify after a revoke stays
   revoked. A ratifier leaving governance does not change admission.
4. **Projection property.** Extend `property_projection_sound_and_complete`
   (cn-perm tests/blueprint.rs) with soundness and completeness clauses for
   entities, attributes, edges (the edge's own rule, completeness) and edge
   attributes. Check it against an oracle written from this ADR's text, not
   from cn-perm's predicates. Generate agreements, consents with bounds and
   entity lists, overlapping agreements, every use, attribute ids shared
   across kinds, intake-marked entities, unowned person-kind entities,
   relational and identity attributes, all four tiers, and `now` at and
   around every `valid_from`, `valid_until` and `expires_at`.
5. **Expiry and revocation.** Nothing is admitted at `now < valid_from` or
   `now >= valid_until`. After a revoke, nothing is admitted for any `now`,
   including one before `revoked_at`. A revoke after `valid_until` is
   accepted.
6. **Cache and use.** In one session, a projection and an index built
   before `valid_until` are not served after it; the key differs across the
   epoch; the Anonymous key and `viewer_fingerprint` are unchanged, and
   `cache_key_survives_fingerprint_collision` passes unchanged. With a
   display projection cached first, a path query does not traverse an
   agreement-only edge. `graph_measures` after a path query ignores
   agreement-admitted edges. A projection served from cache carries the
   current call's `as_of`.
7. **Two keys and coverage.** An owned record with no consent does not
   cross under an active agreement, and a consent with no active agreement
   admits nothing. An unowned person-kind record does not cross, with or
   without the intake block. A member-edited attribute of an intake entity
   does not cross, and neither does an in-app edge to an intake person. A
   consent stops covering a value after governance widens its visibility or
   lowers its tier. An attribute id shared across kinds crosses only for
   the listed kind.
8. **Relations, identity and tiers.** A relational attribute never crosses,
   and a scope that lists one is rejected. An identity attribute crosses
   only with `tier_authority_ref` and the owner's item. T1 crosses only
   with `tier_authority_ref`. T3 never appears in any agreement-admitted
   projection or export, as an entity, attribute override or edge.
9. **Exports.** No agreement-admitted value is exported without `Export`.
   No agreement-only edge is in any export. A grantee's detail and export
   carry no custody, `recorded_by`, `responsible_human` or intake field.
   Exported values carry `tier_after_expiry: T3`, the merged terms and
   every admitting agreement id. Pseudonyms differ across agreements. A
   derived product carries the maximum input tier. The export header
   carries the `ai_ml_restriction` table.
10. **Hidden equals absent.** Detail and query calls on an out-of-scope,
    unconsented or missing id return the same `not_found` (pattern:
    `hidden_and_absent_are_identical_not_found`, cn-api
    tests/blueprint.rs).
11. **Declining limit, documented.** With presence at Network, an owner who
    withholds an in-scope attribute is visible to the grantee as missing
    it; with presence at Group, the record is absent.
12. **Unchanged default.** `tier_ceiling_cells` keeps its assertions. For
    every viewer named in no agreement, the golden projections from step 0
    are identical except for `as_of`.

Receipt tests move with D9.

## Open questions (each answerable in one line)

1. Reciprocity rule default for compacts: membership alone,
   contribution-gated, or per-compact protocol?
2. Who ratifies for a group: two distinct governance members (the Phase A
   floor here), or a k-of-n quorum set per group?
3. May Phase A name grantee groups, which needs cross-group membership proof
   (D6), or only persons?
4. Default maximum agreement duration (for example 90 days, one year, or no
   cap beyond the required `valid_until`)?
5. Should the TSDF mapping read "T2: Group, or agreement-admitted"
   (tier.rs), and what reviewed T2 wording replaces "Shared within this
   group only" (D-032)? This is the Phase A precondition.
6. Do facilitators count as governance for creating agreements, given that
   facilitator authority is write-only and never read-path today
   (`is_facilitator_or_governance`, viewer.rs)?
7. Once Phase B lands, should agreement admission drop in-scope T1 even
   when the tier authority included it, to match protocol section 4.2?
8. May a grantor waive grantee acceptance (`AgreementAccept`), which is
   required by default here?
9. In Phase B, does an owner's Network visibility count as the person key
   for every compact, or must the owner name each compact (the default
   here)?
10. May grantor governance, or a designated data-steward role, audit which
    records crossed to which grantee (the interim default, following
    standard section 5.1), or is an owner's consent secret from governance?
    To be answered with ATNI Climate (D-034), together with ADR-008 open
    question 1.
11. In Phase B, who attests that a party group belongs to a trusted
    Indigenous network, and how does a group review changes to another
    party's roster?
12. What small-cell threshold, set by the tier authority, must `Routing`
    and `NetworkAnalysis` outputs meet before they reach a grantee?
13. Data co-produced with a partner: in scope for a later agreement slice,
    and does it move to T3 on expiry as the decision guide says?

## References

Repository:

- ADR-001 D5, D7, A-B2, A-B3; ADR-002 A-B2, A-B5, A-B8; ADR-003 D3, A-B4;
  ADR-005 D5, D7; ADR-008 (activation version, `subject`, Confirmed).
- DECISIONS.md D-015, D-027, D-030, D-032, D-033, D-034, D-106;
  docs/design/intake-consent-text-2026-09-14-v3.md.
- docs/blueprints/cn-perm.md; docs/PROJECT_PLAN.md:258;
  PLAN_1.0.md:304-315 (G-NET), :860 (P5.6), P7.4.
- Code on main at 7c45afa, by function: cn-perm viewer.rs
  (`admissible_circle`, `viewer_cache_key`, `fingerprint_input`,
  `active_roles_for`), rules.rs, projection.rs (`project`,
  `export_projection`, `edge_visible`, `visible_for_owner`,
  `entity_detail_metadata`), authz.rs (`authorize_op`,
  `require_governance`, `authorize_owner_or_member`, `authorize_tier`,
  `authorize_grant_create`, `authorize_grant_revoke`), reports.rs, mirror.rs,
  tests/blueprint.rs; cn-model tier.rs, trust.rs, time.rs, provenance.rs,
  lib.rs; cn-store fold.rs (`fold`, `set_attribute`, `apply_trust_grant`,
  `revoke_trust_grant`), op.rs; cn-api wire.rs, lib.rs, session.rs,
  analysis.rs, tests/blueprint.rs; cn-wasm lib.rs; core/cli/src/export.rs;
  app/src/ui/format.ts, app/src/wasm/worker.ts,
  app/scripts/embed-snapshot-data.ts; schemas/op-log.schema.json,
  schemas/group-template.schema.json.

TSDF (public text): TSDF Indigenous Data Sovereignty Standard v1.0
(CC-BY-SA 4.0), sections 3.1, 3.2, 5.1 and 6.2 and the OCAP table; TSDF
Tier Classification Decision Guide (decision tree; tier changes; Aggregated
/ Derived Data; Co-Produced Data; documentation requirements for T1 and T2;
common mistakes); TSDF Federated Indigenous Data Protocol v1.0 (Apache
2.0), sections 2.1, 2.3, 4.1-4.4; isdGRAPH Schema Reference v1.0 (Agreement
node, Consent Record node, T2 agreement-scoped query). The framework's
public overview is linked from the consent statement (D-106).

Research findings (docs/research/relatives-discovery-2026-11-11.md):

- R1-F2 USGS Climate Adaptation Science Centers, Supporting Tribal
  Resilience (liaison network ended 2026-07-17).
  https://www.usgs.gov/ecosystems/climate-adaptation-science-centers/tribal-resilience-liaisons
- R1-F5 CIG update, termination of the Northwest Climate Resilience
  Collaborative (2025-05).
  https://cig.uw.edu/2025/05/cig-update-termination-of-the-northwest-climate-resilience-collaborative
- R2-F6 ELOKA, Indigenous Guardians data roundtable (2024-03-04).
  https://eloka.nsidc.org/our-work/stories-events/eloka-co-hosts-indigenous-data-guardians-data-roundtable-northwest-territories-canada
- R3-F8 Te Hiku Media, Kaitiakitanga License.
  https://github.com/TeHikuMedia/Kaitiakitanga-License/blob/tumu/LICENSE.md
- R3-F10 GIDA communique, CARE Directs Us Home (2024).
  https://fnigc.ca/wp-content/uploads/2025/09/GIDA-Communique-CARE-Directs-Us-Home-Prioritizing-Indigenous-Peoples-Community-Standards-final.pdf
- R5-F1 Ink & Switch, Keyhive lab notebook.
  https://www.inkandswitch.com/keyhive/notebook/
- R5-F3 Meadowcap specification (Final, 2025-11-21).
  https://willowprotocol.org/specs/meadowcap/index.html
- R5-F11 biscuit-auth versions (6.0.0, 2025-07-16; checked 2026-11-11).
  https://crates.io/api/v1/crates/biscuit-auth/versions
- R6-F12 IEEE Spectrum, Indigenous AI voice models (2026-05-21).
  https://spectrum.ieee.org/indigenous-ai-voice-models-maori
- R7-F1 Grist with The Intercept, TigerSwan documents (2023-04-13).
  https://grist.org/accountability/tigerswan-documents-dakota-access-pipeline-standing-rock-surveillance/
- R7-F2 The Intercept, TigerSwan expanded surveillance (2017-06-21).
  https://theintercept.com/2017/06/21/as-standing-rock-camps-cleared-out-tigerswan-expanded-surveillance-to-array-of-progressive-causes/
- R7-F3 Upside Down World, military-backed mapping project in Oaxaca
  (2009). https://upsidedownworld.org/archives/mexico/military-backed-mapping-project-in-oaxaca-under-fire/
- R7-F4 Federal Reserve Bank of Minneapolis CICD Working Paper 2023-05.
  https://minneapolisfed.org/research/cicd-working-paper-series/new-2020-census-rules-make-it-harder-to-navigate-native-american-data
- R7-F5 CSTE Tribal epidemiology toolkit, Data Collection.
  https://tribal.cste.org/?page_id=49
- R7-F6 Indigenous Navigator, privacy and consent.
  https://indigenousnavigator.org/node/288
- R8-F5 Entwined Processes: Rescripting Consent and Strengthening Governance
  in Genomics Research with Indigenous Communities (2020).
  https://cambridge.org/core/journals/journal-of-law-medicine-and-ethics/article/entwined-processes-rescripting-consent-and-strengthening-governance-in-genomics-research-with-indigenous-communities/E722581A3EAEB00EC5F40AD9A223DE75
- R8-F6 Kantara Consent Receipt, W3C vocabulary workshop paper (2018).
  https://www.w3.org/2018/vocabws/papers/lizar.txt
- R8-F7 Inrupt, Access Requests and Grants.
  https://docs.inrupt.com/security/authorization/access-requests-grants/
- R8-F9 Contextual Integrity Up and Down the Data Food Chain (2019).
  https://nissenbaum.tech.cornell.edu/papers/Contextual%20Integrity%20Up%20and%20Down.pdf

Panel ideas (2026-11-11): sovereign-architect-1 (Kinship Compact: bridge
consent and sharing agreements as executable records), sovereign-architect-3
and individual-at-center-3 (two-sided relations), sovereign-architect-6
(Council Seal), knowledge-keeper-5 (Honorable Ask), individual-at-center-1
(See Me As), individual-at-center-4 (Receipts I Keep),
emergency-coordinator-1 (Dormant Kin Compacts). The judges' shared
requirements carried here: a time-aware projection with an injected `now`
and a cache epoch; any federated projection inside cn-perm; verify-before-fold
before any compact is honored from a peer log; edges never bulk-exportable.

## Review log (2026-11-11)

One review round ran on the first draft. Three lenses, each reading the
ADR against the code and the TSDF text, plus a Jev citation check:

- **P - permission soundness and non-leakage** (main at 6db3c24 plus the
  relatives/perm commits). Verdict: not acceptable as written.
- **S - TSDF fidelity, sovereignty and harm.** Verdict: revise before
  acceptance.
- **C - fit to the codebase and its invariants** (main at 0048e77, moved
  during review). Verdict: revise before acceptance; land ADR-008 first.
- **Jev check** of the ADR's quotations and research citations (model
  jev-1.14.0, 7 fair controls passed first). 18 ADR-007 claims scored
  supported (0.95 to 0.99), 2 more (R1-F5, R5-F1) passed at 0.98 before
  they were swapped out of the batch, and 2 scored weak.

Every blocking finding is applied. Each finding's disposition follows.
Findings marked "both" also appear in ADR-008's log.

Lens P:

- P1 (blocking) Scope and consent named bare attribute ids. Applied: D1
  scope and consent items use `(kind, attr)`; consent items may name
  entities; test 4 shares attribute ids across kinds; rejected option 8.
- P2 (blocking) The intake exception keyed on value provenance, and unowned
  person records crossed on the group act. Applied: D4 keys the intake
  exclusion on the entity, excludes edges touching an intake or unowned
  person record, and never covers unowned person-kind records; test 7.
- P3 (blocking) Receipt headers were a consent oracle for governance.
  Applied: receipts leave Phase A (D9). Whether governance may audit
  consents is open question 10, with the TSDF interim default disclosed to
  owners at consent. If the human picks secrecy, D9 requires hiding
  headers that change with owned lines. Edge lines with an owned endpoint
  are owned lines, and the mirror follows.
- P4 (blocking) The cache key left out the use. Applied: D3 keys both
  caches on `(viewer key, use)` and builds the index from the use-scoped
  projection; test 6.
- P9 (advisory, both) The tests could not catch these violations. Applied:
  test 4 adds edge and edge-attribute soundness and completeness against
  an oracle written from the ADR text, with the listed generators.
- P10 (advisory) Expiry and revocation gaps. Applied: revocation depends
  only on state and is allowed after expiry; `as_of` is set outside the
  cache; D3 says expiry holds only for honest callers until expiring keys
  exist. Declined in part: rejecting a `now` earlier than the latest op
  time would let any viewer probe the time of the latest op, hidden ops
  included, and state-only revocation already gives a `now`-independent
  stop.
- P11 (advisory) Admission was "exists A" and Export had no enforcement
  point. Applied: admission per `(agreement, use)`; exports gated on
  `Export`; values carry every admitting agreement with merged terms;
  queries may name their agreement (D4, D7, D8).
- P12 (advisory) Receipt write path and forgery. Applied as requirements of
  the deferred receipt slice (D9): core-generated, denied on `submit_ops`,
  returned for persistence, detail and search covered, no visible revision
  bump.
- P13 (advisory) Governance could widen the person key. Applied: consent
  items record visibility and tier bounds, and cn-perm voids them at read
  time when someone else widens the value (D1, D4); test 7.
- P14 (advisory) "Declining looks like absence" was overstated. Applied:
  D2 states the limit and the presence-Group remedy; test 11.
- P15 (advisory) The Phase B cache ignored other groups' state. Applied:
  D5 keys G's cache on the party groups' revisions or clears it on their
  change.
- P18 (advisory, both) Both ADRs claimed the next patch. Applied: ADR-008
  takes 0.1.2 as a named activation constant, and this ADR takes 0.1.3
  (header, Consequences).

Lens S:

- S1 (blocking, both) Unowned person records were treated as group data.
  Applied: the template `subject` field (from ADR-008); unowned person-kind
  records never cross in Phase A; the intake exclusion works at entity
  level (D4).
- S2 (blocking) Relation-bearing and identity attributes bypassed D8.
  Applied: the template marks `relational` and `identity`. Relational
  attributes never cross. Identity attributes need a tier-authority
  reference and the owner's consent (D4); test 8.
- S3 (blocking) Export was called enforceable but was not enforced.
  Applied: exports drop agreement-admitted values without `Export`;
  `Export` needs a `DataReturn` obligation; D1 names which codes the core
  enforces and which honest tools honor (D1, D8); test 9.
- S4 (blocking) The grantee could infer who consented while governance
  could not audit it. Applied: open question 10 for the human and ATNI
  Climate, paired with ADR-008 open question 1. The interim default
  follows standard section 5.1 and is disclosed at consent. D2 states that
  the grantee sees the consenting set. The secrecy branch is specified in
  D9, and consent ops are redacted to counts in governance reports.
- S5 (blocking) Phase A made the app's T2 wording false. Applied: the
  human's answer to open question 5 and reviewed T2 wording are a
  precondition, and Phase A sits behind a build flag until then (D4).
- S6 (advisory) Collective authority was thin. Applied: the ratifier must
  differ from the creator; the agreement carries a governance decision
  reference, a document reference and partner fields with party type; the
  mapping row is corrected (D1, D2).
- S7 (advisory) Phase A was not purely additive, and test 9 could not be
  met. Applied: the cache-key extension stays out of `viewer_fingerprint`;
  the claim is narrowed to "unchanged except `as_of`"; the obligation reads
  "assertions unchanged" (D3, D4, test 12). The suggestion to keep a
  3-argument `project()` is declined: a required `now` and `use` let the
  compiler list every caller (lens C, C8).
- S8 (advisory) Pilot T1 would flow to outsiders. Applied: scope gains
  `tiers` with default `{T2}`; T1 needs a tier-authority reference; the
  decision-tree text is cited as the basis (D4).
- S9 (advisory) AI/ML restrictions were on agreement exports only.
  Applied for exports: every export header carries a per-tier
  `ai_ml_restriction` table (D7). Deferred: the interactive projection
  table belongs to the on-device AI slice.
- S10 (advisory) Stable ids allow joins, and small groups allow
  re-identification. Applied: per-agreement export pseudonyms, a
  quasi-identifier warning, and no `Routing` or `NetworkAnalysis` for
  grantees until a small-cell rule exists (D8; open question 12).
- S11 (advisory) Compact parties had no party type, and the gate framing
  was unclear. Applied: the status says the human accepts the ADR; the
  NetworkCompact shape is a proposal with party type and eligibility; open
  question 11 covers eligibility and roster review (D5).
- S12 (advisory) The `auto_upgrade_on_expiry` mapping was an analog.
  Applied: the mapping says so; co-produced data is out of scope (open
  question 13); expiry and inertness are receipted events in D9; an
  agreement binds the body, so a ratifier's departure no longer ends it
  silently (D2).
- S13 (advisory) Consent was too coarse. Applied: consent items may name
  entities; consent has `expires_at`; asking per use is owed; the
  agreement's terms project to grantees; `AgreementAccept` is required by
  default (D1, D2; open question 8).

Lens C:

- C2 (blocking) "Keeps its provenance envelope" would expose custody and
  intake ids to grantees. Applied: D7 lists exactly what crosses (origin
  class only, never the envelope) and keeps projection rule 7; test 9.
- C5 (blocking, both) The version bumps were not coordinated, and the
  reader rule was misstated. Applied: 0.1.2 for ADR-008, 0.1.3 here; the
  `accepts_schema` rule is stated correctly (Consequences).
- C6 (blocking) Receipts could not be built as specified. Applied: D9
  moves out of Phase A with the slice's requirements listed; the receipt
  claims are dropped from Consequences.
- C7 (blocking) The REQUIRED `auto_upgrade_on_expiry` was presented as met.
  Applied: every agreement-served value and derived product carries
  `tier_after_expiry: T3`, and the live-projection behavior is a stated
  divergence (D7, mapping table).
- C8 (advisory) The list of places needing `now` was incomplete. Applied:
  `now` and `use` are required parameters, D3 lists every caller found on
  7c45afa, and write-path checks keep the agreement-free predicate.
- C9 (advisory) D8's use-scoped projections conflicted with D3's single
  cache key. Applied with P4: D3 assigns a use to each entry point.
- C10 (advisory) The cache key change would move `viewer_fingerprint`,
  break the collision test and change the projection shape. Applied:
  `viewer_cache_key` only; `as_of` outside the cache; `BOUNDARY_VERSION`,
  projection schema and DTO updates listed (D3).
- C11 (advisory) Agreement status precedence was unspecified, and a
  ratifier's departure made agreements inert. Applied: absorbing
  `revoked_at`, ratify after revoke is a no-op, the agreement binds the
  body, and a convergence test is added (D1, D2; test 3).
- C19 (advisory) Gate framing. Applied in the status line (see S11).
- C20 (advisory, both) Stale anchors. Applied: code is cited by function on
  7c45afa, and the record mirror is described as built.
- C21 (advisory, both) Missing test cases and order. Applied: golden
  captures come first, then the listed order, with the missing cases
  (index cache by use, grantee write paths, detail and provenance
  exposure, revoke and ratify convergence, ratifier departure). The
  receipts-survive-reload case moves with D9.

Jev check:

- J1 (weak, 0.39) "A flow can need two keys" was cited to R8-F5. Fixed:
  D2 now rests the two keys on TSDF standard section 5.1, and the research
  list says the two-keys step is the digest's note on R8-F5, not the
  finding.
- J2 (weak, 0.31) "A hostile party wants exactly the edge list" was cited
  to R7-F1. Fixed: the research list states the finding (watch lists and
  dossiers) and marks the edge-list step as the digest's inference adopted
  as design. R7-F2's purpose-and-proof step is marked the same way.
- J3 (manual caveat) The `auto_upgrade_on_expiry` mapping was weaker than
  the source. Fixed with C7.
- J4 (manual caveat) D7's max-tier rule did not mention the guide's
  de-identification allowance. Fixed: D7 says this ADR never lowers a tier
  for de-identification, and why.
- J5 (manual caveat) The biscuit-auth version can go stale. Fixed: option
  5 and the reference carry the date it was checked.
- The other 20 claims (the TSDF quotations, R1-F2, R1-F5, R2-F6, R5-F1,
  R5-F3, R5-F11) scored supported and are unchanged. The research checks were made
  against the digest, not the primary sources.
