# ADR-008: Two-sided relations - acknowledged edges, a final refusal, and ties visible to their two ends

- Status: Proposed (2026-11-11). Revised the same day after one review
  round: three review lenses and a Jev citation check (see Review log). Not
  accepted. This ADR changes the authority matrix (a new owner-only op),
  adds a fold rule that is not last-writer-wins, and changes persisted
  formats, so it needs its adversarial round closed before acceptance
  (CLAUDE.md architecture-stance rule; D-056.2 names owner-bound authority
  as an authority-matrix change).
- Date: 2026-11-11
- Phase: 5 (personal mode, R4). It must land before any person-to-person
  edge is created from intake, and before ADR-007.
- Drivers: R4 (individuals own their record; every view is a
  permission-filtered projection), R6 (routing must run over consented
  ties), R10/I6 (provenance on every record), I2 (permission logic only in
  cn-perm), I3 (no silent failure), I7 (versioned persisted formats), the
  real-data gate (only consenting people enter the graph, D-030), TSDF
  standard section 5.1 (consent from both the governing body and the
  individual), and the D-105 flag on the convention form's question about
  connections.
- Depends on: two commits now on main. 49a51cb (`fix(store): quarantine
  edges whose group_id is not the group's`) adds the missing `valid_group`
  check to `apply_edge_create` (cn-store fold.rs). 23aacec (`fix(perm):
  edge submissions no longer reveal whether a hidden entity exists`) makes
  `authorize_edge_create` (cn-perm authz.rs) require a non-governance
  submitter to have both endpoints in their projection, and deny a hidden
  or absent endpoint the same way (`target_missing`). Governance still
  reaches any target.
- Order: lands first, at the edge-consent activation version (model schema
  0.1.2). ADR-007 follows at 0.1.3.
- Anchors: code is cited by file and function on main at 7c45afa. Line
  numbers are left out because main moved twice during review.
- Out of scope, reserved: ADR-006 is the dedup ADR (PLAN_1.0 P5.6).

## Context

Edges between people are claims about people, and today nobody asks them
before the claim is made.

1. **Anyone in the group can link anyone they can see.** `authorize_op`
   sends EdgeCreate to `authorize_edge_create`, which checks
   `require_member` and, for non-governance submitters, that both endpoints
   are in the submitter's projection. The authority matrix test pins the
   cells (cn-perm tests/authority_matrix.rs, `edge_create_cells`). The
   owners of the two endpoints are never asked. Seeing a person is not
   their consent to be linked.
2. **Edges have no owner, so the two narrowest circles cannot be reached
   for edges.** `edge_visible` passes `None` as the owner (cn-perm
   projection.rs), and so does `visible_edge_attributes`. With no owner,
   `admissible_circle` never returns Private or Trusted (viewer.rs). The
   blueprint records the consequence: "Values with NO owner (owner: None)
   use the entity's group as owner authority: Private unreachable, Trusted
   unreachable, so their floor is Group behavior" (docs/blueprints/cn-perm.md).
   A Trusted or Private edge is invisible to every viewer, its creator
   included. No tie can be held just between two people.
3. **No endpoint can refuse.** Only governance can archive an edge
   (`EdgeLifecycleSet` goes to `require_governance`), and only governance
   can change an edge's visibility, since an edge has no owner
   (`authorize_visibility`). A person cannot hide or decline a tie about
   themselves.
4. **The fold cannot express a final no.** Writes are last-writer-wins per
   field (cn-store fold.rs, `lww`), with one clock per object and FieldKey.
   Stored as an ordinary field, a refusal would be overturned by any later
   accept. The clocks (`field_clocks`) are private to `GroupState`, so
   cn-perm cannot read when an edge was created or answered.
5. **Intake is about to create edges.** The unmerged `s-e2-intake-edges`
   branch (tip 6c92eed) turns committee and organization answers into
   `member_of` and `affiliated_with` edges at `Circle::Group`, tier T1,
   from a person entity that lands unowned (ADR-005 D7). D-105 flagged
   that the form's sixth question ("Who are you connected with") invites
   names of third parties who did not consent. Turning those answers into
   person-to-person edges needs a consent handshake first.
6. **Real respondents have no owner, and the core cannot tell a person
   record from a committee.** Approved intake persons land unowned (ADR-005
   D7), so a protection that asks only owners protects no real respondent
   today. The template schema has no marker for which kinds describe a
   person (schemas/group-template.schema.json), and `GroupState` holds the
   template (`state.template`), so a marker there is readable by cn-perm.
7. **Introductions are built.** `suggest_introductions` (cn-graph
   introductions.rs, 0048e77) computes Adamic-Adar suggestions from the
   viewer's projection and graph index, and 7c45afa wires it through cn-api
   (`suggest_introductions_impl`) and cn-wasm. A non-governance viewer may
   only ask about their own entity; governance may ask about any
   (`analysis_scope`).
8. **The fixtures already hold person-to-person edges.**
   `fixtures/groups/atni-convention.ops.jsonl` holds 306 EdgeCreate ops,
   all at Group circle and T1: 60 `connected_to` edges between two owned
   person entities, 180 `member_of` edges from an owned person to an
   unowned committee, and 66 `affiliated_with` edges from an owned person
   to an unowned organization. The presenter beats filter on
   `connected_to`. The other fixtures hold more: `relatives-demo` has 40
   `connected_to` and 6 `learned_from` edges between owned people, and
   `research-network` has 60 `coauthored` edges. All are at Group circle,
   in ops stamped schema 0.1.0 or 0.1.1, and in every fixture the op stamp
   and the edge's own `schema_version` agree. `fisheries-committee` has no
   edge between two owned entities. Whatever this ADR does must leave all
   of those projections unchanged.

The research and the panel point the same way:

- Ecosystem atlases record one-sided claims ("A says it works with B") and
  publish them openly. A tie asserted by A and confirmed by B would be a
  real advance (R7-F12).
- SIKU, an Inuit-led platform, lets each user set control per post:
  "Privacy, sharing, stewardship and ownership can be controlled on a
  post-by-post basis". It also says that seeing a post is not the right to
  reuse it (R2-F1).
- Seed keepers keep a relationship visible while withholding its contents
  (R2-F9). D6 defers sealed contents. Per-attribute visibility on edge
  attributes, which D4 makes reachable, serves part of this now.
- The Honorable Harvest asks permission and listens for the answer: "if
  we're going to ask permission, we have to listen for the answer as well"
  (R3-F5). A no must be final, carry no penalty, and not be asked again
  and again.
- Trust between Tribes and institutions accrues through repeated
  reciprocal acts, not instant friend requests (R3-F2).
- Brokers who bring people together burn out less than brokers who keep
  people apart (R4-F8). The digest's note reads this as a design of
  introduce-and-step-out; that step is the note's, not the finding's. A
  tie to a mutual contact raises the rate at which new ties form (R4-F10).
  The digest's caution note on R4-F10 adds that the study measured whether
  ties formed, not whether they lasted. Either way, an unconfirmed tie
  should not count as a tie.
- Panel ideas: Two-Sided Relations with sealed contents
  (sovereign-architect-3), Asked and Answered (individual-at-center-3),
  and Honorable Ask (knowledge-keeper-5) propose this mechanism. The three
  judges' notes from the same discovery's scoring say: land two-sided
  consent before any S-E2 person edges (community value); fix edge
  ownership and consent before any feature (architecture feasibility);
  build the mirror, then two-sided relations, then signed ops, then the
  council seal, then compacts (sovereignty harm). The mirror is built
  (cn-perm mirror.rs, c75f904).

The owner's Tiered Sovereign Data Framework sets the bar this ADR must
meet (TSDF Standard v1.0, CC-BY-SA 4.0; Federated Indigenous Data
Protocol v1.0, Apache 2.0; the tier decision guide and the isdGRAPH schema
reference ship beside them):

- Standard section 5.1: "Free, Prior, and Informed Consent (FPIC) must be
  treated as an **ongoing dialogue**, not a one-time transactional event.
  It is a process that can evolve, and includes the right for consent to
  be withdrawn at any time."
- Standard section 5.1: "Consent must be obtained from both" "The
  appropriate Indigenous governing body" and "Individual knowledge
  holders".
- Standard section 5.1: "All grants of consent, including specific
  conditions or limitations, must be formally documented and linked to the
  associated data."
- isdGRAPH schema, core design principle 4: "No data without documented
  consent linkage".
- Standard section 3.1, T1: "relationships are mutual and ongoing, not
  transactional".
- Standard section 3.1, T3, on strategic invisibility: "the choice to
  remain unseen is itself an exercise of sovereignty".

## Decision

### D1. EdgeAcknowledge: an endpoint owner's answer, and only theirs

A new op records one endpoint's answer to one edge:

```
OpKind::EdgeAcknowledge { edge: EdgeId, endpoint: EntityId, answer: EdgeAnswer }
EdgeAnswer::Accepted { up_to: Circle }
EdgeAnswer::Refused  { quiet_until: Option<Timestamp> }
```

- **Endpoint classes.** The template gains `subject: person | collective`
  per kind, a 0.x template-schema addition (I7). A kind that does not
  declare one counts as `person`, so an undeclared kind fails closed. Each
  endpoint of an edge is then one of three things:
  - an **owned** endpoint (the entity has an owner), whose owner can
    answer;
  - an **unowned person** endpoint (no owner, a person kind), which cannot
    answer and never counts as having said yes;
  - an **unowned collective** endpoint (no owner, a collective kind, such
    as a committee, an organization or a place), whose authority is the
    group's, as for every unowned value (cn-perm.md), and which needs no
    answer.
- **Who may answer.** cn-perm authorizes EdgeAcknowledge only when
  `endpoint` is `edge.from` or `edge.to`, the submitter owns `endpoint`,
  and the edge appears in the submitter's own projection (in
  `Projection.edges` or in their `asks`, including `you_declined`). There
  is no governance, facilitator or delegate path in this slice. An absent
  edge, a hidden edge, an entity that is not an endpoint, and a submitter
  who is not the owner all get the same denial, `target_missing`, the code
  23aacec uses for hidden and absent EdgeCreate endpoints. So an answer
  cannot probe whether a hidden edge exists (ADR-003 A-B4, D-015). Limit:
  every applied op bumps the session revision every viewer can see, a
  channel that exists for all ops (23aacec names it) and that this ADR
  does not close.
- **`up_to` is the widest circle this endpoint agrees to.** The client
  fills it with the edge's current visibility, and the owner may choose a
  narrower one. A later governance VisibilitySet cannot widen a tie past
  what its ends agreed to. Today one governance member can widen an edge
  alone (`authorize_visibility` with `require_governance`).
- **Answers live beside the edge, not inside it.** `GroupState` gains
  `edge_acks: BTreeMap<(EdgeId, EntityId), EdgeAck { answer, op_id,
  sort_key, answered_by: PersonId }>`. `sort_key` is the winning answer's
  op sort key, and `answered_by` is that op's `responsible_human`. `Edge`
  (cn-model edge.rs) is unchanged, so an EdgeCreate payload cannot carry
  acknowledgments, and a creator cannot pre-fill someone else's yes.
- **Answers are checked against the current owner at read time.** An
  answer counts only while `answered_by` is the endpoint's current owner,
  the read-time validity pattern of ADR-002 A-B2. An answer by someone who
  does not own the endpoint now is inert: the endpoint counts as not
  answered, and a quiet period it set lapses. No op changes an entity's
  owner after creation on main today; the rule is fixed now so that the
  owner-claim primitive (D-056.2) inherits it.
- **No implied yes.** The asker's own side is an explicit `Accepted`,
  submitted in the same batch as the EdgeCreate. `cn_store::submit`
  authorizes and applies each op in order (cn-store authz.rs), so the
  answer authorizes against the edge created just before it. No answer is
  ever inferred from who created an edge, with one exception: an EdgeCreate
  whose provenance carries a respondent's own ADR-005 intake block counts
  as that respondent's `Accepted { up_to: Group }` for their own endpoint
  only, and only when the block's `record_id` matches the endpoint
  entity's own intake block. The respondent stated the tie themselves. The
  in-app submit path refuses an EdgeCreate whose provenance carries an
  intake block; the ADR-005 approval path is meant to be its only writer
  (to be verified against cn-ingest's approval path when S-E2 lands). The
  TSDF requires that every grant be "formally documented and linked to the
  associated data". The op is that record: its envelope carries the actor,
  the responsible human and the time (cn-store op.rs), and `EdgeAck.op_id`
  links the answer back to it.
- **The asker** of an edge is the `responsible_human` of its EdgeCreate op.
  cn-perm denies an EdgeCreate whose payload `provenance.responsible_human`
  differs from the op's (`provenance_mismatch`), so no member can label an
  ask as someone else's.

### D2. Status and projection

cn-perm derives each edge's status at read time from current state,
following the read-time validity rule of ADR-002 A-B2. Let P be the
endpoints that are owned or unowned person (the endpoints that stand for a
person), and O the owned ones.

- **Refused** if any endpoint in O has a counting Refused (D3).
- **Confirmed** if every endpoint in P has a counting Accepted (including
  the intake self-acceptance in D1; trivially so when P is empty), or the
  edge is grandfathered (D5).
- **Suppressed** if it is not Confirmed and falls inside a refusal's quiet
  period (D3).
- **Standing** if P is exactly one owned endpoint that has not answered,
  and the other endpoint is collective: the opt-out case below.
- **Pending** in every other case.

The order above is the precedence. Refused outranks Confirmed, so a
withdrawal ends a tie. Confirmed, grandfathered included, outranks
Suppressed, so a refusal never hides a tie that was already confirmed.
Suppressed outranks Standing, so a quiet period also holds for an edge to
a body.

Projection rules:

- **Two person endpoints, pending.** The edge never enters
  `Projection.edges` for any viewer. It projects only to its endpoint
  owners and its asker, as an entry in a new per-viewer list,
  `Projection.asks` (`ProjectedAsk { edge, kind, from, to, status }`),
  and only when both endpoints are in that viewer's entity projection,
  the same rule edges already follow (`project_edges`). This holds for an
  edge governance creates, too: governance may still link any two
  entities, but a tie between two people it creates waits for both of
  them like any other. cn-graph builds its index only from
  `Projection.edges` (cn-graph index.rs, `GraphIndex::build`), so an
  unconfirmed tie never routes, never counts in a measure, and never
  seeds a suggestion. A tie to an unowned person record stays pending,
  because that endpoint cannot answer. Intake therefore creates no
  person-to-person edge until owner-claim exists (Consequences).
- **One person endpoint, owned** (for example, an owned person `member_of`
  a committee). A Standing edge projects as it does today, at Group circle
  or narrower (the cap below), and its owner may refuse it (D3). It is not
  pending. After the owner's Accepted it is Confirmed.
- **One person endpoint, unowned** (for example, an intake respondent
  `member_of` a committee). Nobody can refuse it, so it is Confirmed only
  through the respondent's own intake block (D1) or grandfathering (D5).
  Otherwise it is pending and projects only to its asker. S-E2's edges
  therefore project when they carry the respondent's own block, and an
  in-app edge that names a respondent does not.
- **Above Group needs every owned endpoint's yes.** The effective
  visibility of an edge with P non-empty is `min(edge.visibility, cap(e)
  for each e in P)`. `cap(e)` is the `up_to` of e's counting Accepted. It
  is Group when e has not answered, when e is an unowned person, and when
  the edge is grandfathered (D5). A Network or Public edge with an
  unanswered owned endpoint is therefore capped at Group, and so is every
  grandfathered edge until a real Accepted. The tier ceiling still applies
  on top (`required_circle`, cn-perm rules.rs).

### D3. Refusal is final and private

- **Refused is monotonic.** For one `(edge, endpoint)`, an answer is
  ranked by `(rank, sort_key)` in lexicographic order, where Refused
  outranks Accepted. The fold keeps the maximum. Within one rank, the
  later sort key wins, so a refuser can change `quiet_until` and an
  accepter can change `up_to`. Nothing moves an answer from Refused back
  to Accepted. At the edge level, one counting Refused from any owned
  endpoint makes the edge refused, and no later Accepted from anyone
  revives it. Accepted followed by Refused is a withdrawal, which the TSDF
  requires be possible "at any time", and it is final too. Because the
  order is total, the fold result does not depend on op order or
  duplication. This is a new fold rule beyond per-field last-writer-wins,
  and it gets its own convergence property test.
- **A refused edge projects to no one but the refuser.** It is absent from
  `Projection.edges` for every viewer. The refuser alone sees it, in their
  own `asks` list, as `you_declined`. Governance can still archive the
  edge, but no op un-refuses it. Whether governance may see refusals for
  abuse review is open question 1.
- **Everyone but the refuser sees the same thing for refused and
  unanswered.** For every viewer except the refuser, `ProjectedAsk.status`
  is one of two values: `awaiting_you` (the viewer owns an endpoint that
  has not answered) or `not_confirmed` (everyone else, including an
  endpoint owner who has accepted). A refused ask and an unanswered ask
  produce the same entry, with no reason, no answer time and no refuser
  identity. An asker who cannot see a confirmed edge, because its ends
  accepted it only up to a circle the asker is outside, keeps a
  `not_confirmed` entry too, so a narrow yes looks like no answer. When a
  confirmed tie is withdrawn, it leaves the projection, and its
  counterpart sees it return to `not_confirmed`. That change is
  observable, and the withdrawal right makes it unavoidable. No reason is
  ever shown.
- **One owned endpoint: a refusal is visible as absence.** A Standing edge
  (one owned person and a collective) has no pending state, so its refusal
  shows to the group as the edge's absence. Privacy of a refusal holds only for ties between
  two people. This is weaker protection, recorded as such.
- **A quiet period, set by the refuser.** `quiet_until` (`None` means no
  end) binds the pair (counterpart endpoint, refusing endpoint). An edge
  between those two entities is suppressed when all of these hold:
  - its EdgeCreate sorts after the refusing answer (cn-perm compares the
    edge's create sort key with `EdgeAck.sort_key`);
  - its EdgeCreate's HLC wall time is no later than `quiet_until`;
  - its asker is not the refuser, so the refuser stays free to ask the
    other side later;
  - it is not Confirmed or grandfathered.
  A third-party introducer therefore cannot route around the no
  (knowledge-keeper-5), and a tie that existed before the refusal is
  untouched. Suppression is not a denial, because a denial would tell the
  asker they had been refused. The new edge folds normally.
- **A suppressed edge looks unanswered to everyone but the refuser.** For
  a tie between two people, it projects to every viewer other than the
  refuser exactly as an unanswered pending edge would: the asker sees
  `not_confirmed`, and an unanswered counterpart, including an asker who
  did not self-accept, sees `awaiting_you`. The refuser gets nothing: no
  `awaiting_you` and no entry. For an edge that would otherwise be
  Standing, a suppressed edge stays out of `Projection.edges` for everyone
  and shows to its asker as `not_confirmed`; as above, refusals of that
  kind are not private from the group. An ask made after the quiet period ends is a
  fresh ask.
- **Clock bounds on the submit path.** The comparison uses claimed clocks:
  the core reads no clock (cn-model time.rs), and submitters choose their
  own sort keys. The submit path therefore rejects an EdgeCreate or
  EdgeAcknowledge whose HLC wall time lies outside the caller's `now_ms`
  plus or minus a stated skew constant (`clock_out_of_range`, a typed
  error, I3). cn-api passes `now_ms` through for this instead of discarding
  it. Limit: a caller who also chooses `now_ms` controls both, which is
  the D-015 class (Consequences).
- **Introduction suggestions withhold a pair only from those who know of
  the ask.** cn-perm computes, per viewer, the pairs with an unconfirmed
  ask (pending, refused or suppressed) of which that viewer is the asker
  or an endpoint owner, and `suggest_introductions` gains that withheld
  set as an input. A viewer who knows of no ask between a pair gets exactly
  the suggestions they would get if the ask did not exist. Governance in
  particular cannot learn of an ask by computing the expected candidates
  from its own `Projection.edges` and seeing one missing. A third party who
  acts on a suggestion is stopped by the quiet period, not by the
  suggestion list.
- **Governance store reports redact answers.** `redact_report` returns the
  full report to governance today (cn-perm reports.rs). A quarantined
  EdgeAcknowledge op appears in governance's report as a count only, with
  no edge, endpoint, answer or responsible human, until open question 1 is
  answered. EdgeAcknowledge adds no custody event (D-033), so the custody
  chain governance sees carries no answer.

### D4. edge_visible passes the endpoint owners

For a Confirmed or Standing edge, `edge_visible` and
`visible_edge_attributes` (projection.rs) stop passing `None`. They compute the viewer's admissible
circle over the owned endpoints:

- **Private** if the viewer owns any owned endpoint.
- Otherwise, the most restrictive of `admissible_circle(viewer, owner)`
  over the owners of the owned endpoints. Trusted then needs an active
  TrustGrant to the viewer from every owner: two keys, not one. Group and
  Network resolve as today.
- With no owned endpoint, `None`, exactly as today.

A Private edge is visible only to its two ends, and a Trusted edge to its
two ends plus the people both of them trust. This makes the two narrowest
circles reachable for edges at last, and it makes Private edge attributes
reachable by the two ends (R2-F9 in part). The both-endpoints rule
(`project_edges`) still applies, and so does the effective-visibility cap
from D2. The cn-perm blueprint text on owner-less values (cn-perm.md) is
amended to match.

### D5. Migration: edges created before activation are grandfathered, capped at Group

- **Version.** The model schema version that every op envelope carries
  (cn-model lib.rs, `MODEL_SCHEMA_VERSION`, now 0.1.1) moves to **0.1.2**,
  and cn-model gains a named constant, `EDGE_CONSENT_ACTIVATION = 0.1.2`.
  That is a patch bump, following the ADR-005 D5 precedent (661b02c).
  `accepts_schema` admits only the same minor at major zero, and the same
  check gates template schema versions (cn-schema validate.rs) and every
  submitted op's `template_version` (cn-api wire.rs), so a minor bump
  would reject every existing log, template and fixture. ADR-007 takes the
  next patch, 0.1.3, so no edge created before this ADR is active can carry
  the activation version. The atni-convention template's own version is
  also 0.1.2 (D-105). That is a template field and is never compared here.
- **The marker is the edge's own stamp.** `GroupState` holds edges, not
  ops, so the grandfather check reads `edge.schema_version`, never the op
  stamp and never `template_version`.
- **No back door.** On the submit path, cn-perm denies an EdgeCreate whose
  `edge.schema_version` differs from `op.schema_version` or is below
  `EDGE_CONSENT_ACTIVATION` (denial `pre_activation_edge`). This applies to
  the in-app path and to the ADR-005 durable-seam preflight. Loaded logs
  fold without authorization today (`fold`). A hand-built log can forge an
  old stamp as easily as it can forge an answer, so this ADR adds no
  fold-time guard for a hole it cannot close (see Consequences).
- **The versioned rule.** An edge is grandfathered when its
  `edge.schema_version` is below `EDGE_CONSENT_ACTIVATION` and the group's
  template declares `synthetic: true` (a 0.x template-schema addition that
  rides with `subject`, I7). A grandfathered edge is Confirmed, with every
  person endpoint capped at Group (D2). Every fixture edge is at Group, so
  every existing projection is unchanged, including the 60 `connected_to`
  edges between owned people in the ATNI convention fixture and the
  presenter beats built on them. No history is needed: the cap does not
  depend on what the edge's visibility used to be. Widening a grandfathered
  tie above Group takes a real `Accepted` from each owned endpoint, so a
  later governance VisibilitySet to Public still projects at Group.
  Grandfathering supplies a yes for compatibility. It does not remove the
  right to say no: a grandfathered edge's endpoint owner may still Refuse
  it (D3). It is never an acknowledgment for crossing graphs (ADR-007 D8).
- **Real data.** On main no real ingestion has run (HANDOFF.md), and
  approved intake creates no edges, so grandfathering touches only
  synthetic data today. The TSDF's guidance for historical data is the
  conservative direction: "For data collected before TSDF
  implementation:" "Default to T3" and "Document original collection
  context and consent status" (tier-decision.md, Historical Data). In a
  group whose template is not flagged `synthetic`, a pre-activation edge
  is not grandfathered: it is Pending until its person endpoints answer.
  Governance, or ATNI Climate as the tier authority for the pilot (D-034),
  may review such edges, but a review cannot supply a person's yes.
  Facilitators have no read-path authority (D-028), so they are not the
  reviewers. Limit: the flag is template data; the real-data gate
  (CLAUDE.md) must check that no real-data template sets it.
- **Formats (I7).**
  - `schemas/op-log.schema.json` gains the EdgeAcknowledge variant.
  - `GroupState` gains `edge_acks` and `edge_created: BTreeMap<EdgeId,
    SortKey>`, the sort key of the EdgeCreate the fold applied, which the
    quiet-period rule reads.
  - `FieldKey` gains `EdgeAck(EntityId)` under `ObjectKey::Edge`,
    recording the winning answer's sort key within its rank.
  - `schemas/group-template.schema.json` gains `subject` per kind and
    `synthetic` at the top level.
  - The cn-store state snapshot (cn-store log.rs, `SNAPSHOT_SCHEMA` 0.1.0)
    gains the new maps and clocks, so it takes a patch bump too. The
    loader rejects a snapshot outside major 0, minor 1, so the bump stays
    inside 0.1. A snapshot whose payload cannot be parsed is discarded and
    refolded from the log.
- **Fixtures, generators and test states.**
  - The four fixture templates gain `synthetic: true` and a `subject` for
    every kind, and their GroupCreate ops, which embed the template, are
    regenerated. The golden projections (test 0) prove nothing else moved.
  - The generators pin their stamps (app/scripts/generate-atni-ops.mjs at
    0.1.1, generate-relatives-ops.mjs at 0.1.0). Those pins stay
    deliberate. A generator that moves an edge to the activation version
    must write explicit synthetic `Accepted` answers for its edges between
    two people, or they turn pending and the presenter beats change. The
    S-E2 merge already owes a regeneration of the ATNI fixture.
  - Rust test helpers build edges with `Edge::new`, which stamps
    `model_schema_version()`, so after the bump every helper-built edge
    between two owned entities is post-activation and pends. The helpers
    in cn-perm tests/blueprint.rs and cn-api tests (blueprint.rs,
    hidden_invariance.rs) add explicit answers where a test needs a
    confirmed tie.
  - One synthetic scenario, built in tests, runs at the activation version
    with explicit EdgeAcknowledge ops, so the handshake itself is
    exercised.

### D6. Sealed contents and per-relation key destruction are a later slice

This ADR does not seal edge contents. Encrypting an edge payload to both
endpoint keys with a salted public commitment (sovereign-architect-3), and
releasing a tie for good by destroying its key (crypto-shredding: "Encrypt
the sensitive attributes, with a different encryption key for each
resource (such as a customer)", R8-F11), are a separate decision. That
decision needs per-subject key custody, key-bound viewers, and an answer
to ADR-005's open question on what removal means. The projection rules
here hold whichever way it goes.

## Options considered and rejected

1. **Owner-asserted edges only** (an edge counts once either endpoint's
   owner asserts it). Rejected. One side still asserts a tie about the
   other, which is exactly the one-sided claim R7-F12 describes. It also
   leaves third-party introductions (R4-F8, R4-F10) with no consented
   form.
2. **Governance approval only** (governance reviews each new edge).
   Rejected. It replaces individual consent with collective consent,
   where the TSDF says "Consent must be obtained from both" the governing
   body and individual knowledge holders. It cannot hold a person's
   private no. It loads review work onto staff who are already thin
   (R1-F3). And it makes governance a permanent gatekeeper broker, the
   pattern R4-F8 links to burnout. Collective consent for widening (the
   Council Seal, sovereign-architect-6) is a separate guard that composes
   with this one, not a substitute for it.
3. **An edge owner field set to its creator.** Rejected. It makes Private
   and Trusted reachable, but for the wrong person: the asker, who may be
   neither end. It gives the asserting party sole control over a tie
   about others. And adding a persisted field to Edge changes the
   EdgeCreate payload that the side map in D1 leaves alone.
4. **Answers as ordinary last-writer-wins fields.** Rejected. A later
   Accepted, or a forged later clock, would silently reverse a no (R3-F5).
5. **Denying re-asks at submit with a typed error.** Rejected. The denial
   itself would tell the asker they were refused. Suppression keeps
   refused identical to unanswered.
6. **Grandfathering keyed on the op stamp, with `up_to` from the edge's
   pre-activation visibility.** Rejected. `GroupState` holds neither the
   op stamp nor the visibility history, and reading the unchecked payload
   stamp would let an old-stamped edge skip the handshake (D5).
7. **Denying edges to unowned person records at submit.** Rejected. Such a
   denial would tell the submitter whether a visible record has an owner.
   Keeping the edge pending carries no signal and holds it for owner-claim.

## Consequences

Positive:

- For owned people, it closes non-consensual linking between two people
  and makes Trusted and Private edges reachable. A tie between two people
  asserts nothing to anyone until both agree.
- For unowned person records, including every real respondent today, a tie
  to another person stays pending, and a tie to a body projects only when
  the respondent stated it themselves through intake.
- A no is final, private between two people, and costs the refuser nothing
  (R3-F5). For a tie between two people, every viewer other than the
  refuser sees the same thing whether the refuser said no or said nothing.
- Unconfirmed ties never reach cn-graph, so routing, measures and
  suggestions run only over confirmed ties.
- Existing projections, fixtures and presenter beats are unchanged (D5).
- Only a tie confirmed by an explicit yes at every owned endpoint should
  ever cross a group boundary. A later cross-group bridge (ADR-007; the
  Kinship Compact, sovereign-architect-1) inherits that rule rather than
  inventing one.

Negative, accepted, or owed:

- **I7.** The model schema moves 0.1.1 to 0.1.2, with the activation
  constant. `OpKind` gains EdgeAcknowledge; `GroupState` gains `edge_acks`
  and `edge_created`; `FieldKey` gains `EdgeAck(EntityId)`; the template
  schema gains `subject` and `synthetic`; the cn-store snapshot schema
  bumps; op-log.schema.json gains the variant. A 0.1.1 build reading a
  0.1.2 log fails loudly on the unknown variant only if the log contains
  an answer. A 0.1.2 log with pending edges and no answers would parse
  under a 0.1.1 reader, which would show those edges. This ADR rests on
  the same deployment assumption as ADR-005 D5: one workspace commit, one
  machine, no old reader. Once logs travel between builds (R5 sync), this
  has to become a minor bump with an explicit predecessor-read rule.
- **These rules are courtesies until identity is bound.** The rules are
  enforced on the authorized submit path and in projection, but the
  submitter, the viewer and the clocks are caller-asserted (D-015;
  `parse_viewer`), and loaded logs fold without authorization. A forged
  answer, a forged clock within the skew bound or with a forged `now_ms`,
  a forged intake block, or a forged owner defeats the handshake for
  anyone who can write the log or choose the clock. Signed ops and
  key-bound viewers close this. This ADR does not.
- **A record of refusals now exists.** If it ever leaked, it would be
  relationally explosive (knowledge-keeper-5). It lives in the op log,
  which anyone holding `ops.jsonl` can read, and in the local cn-store
  state cache beside it. EdgeAcknowledge is non-disclosable under the
  ADR-002 A-B5 export gate: it never enters an export, a sync frame or a
  baked offline snapshot (R8), and `Projection.asks` is omitted from
  `export_projection` and from every baked snapshot.
- **S-E2.** Intake `member_of` and `affiliated_with` edges connect an
  unowned respondent to an unowned committee or organization. Under D2
  they are Confirmed only when each edge's provenance carries the
  respondent's own intake block, matching the person entity's block. This
  was not checked against 6c92eed. If S-E2 does not stamp the block on its
  edges, they stay pending until it does. At the Group circle S-E2 uses,
  a confirmed one projects as today.
- **Intake person-to-person edges wait.** An unowned person endpoint
  cannot answer, so intake creates no person-to-person edge (the
  connections answer, D-105) until both endpoints are owned. That requires
  the owner-claim primitive, which is an authority-matrix change with its
  own adversarial round (D-056.2). Until then, the connections answer
  stays a tag attribute, which ADR-007 marks `relational` and never lets
  cross.
- **An unowned respondent cannot refuse.** Until owner-claim exists, the
  only way to withdraw an edge about an unowned respondent is a governance
  archive. That path is owed, not solved.
- **Inside the group, an owned person's edge to a body is opt-out.** An
  edge from an owned person to an unowned collective (membership,
  affiliation) projects at Group without that person's yes, and they can
  refuse it, but the refusal is visible to the group as absence. A tie
  between two people is opt-in and its refusal is private. This split is
  deliberate: the first is a claim about participation in a body the group
  governs, and the second is a claim about two individuals. It is weaker
  protection for the first kind and is recorded as such.
- **New surfaces.** `awaiting_you`, `not_confirmed` and `you_declined`
  need app surfaces that meet I9. Their wording is community-facing text
  and needs human review before use (CLAUDE.md real-data gate).

## Test obligations

In this order:

0. **Golden captures first.** Before any change, capture the projections
   and introduction suggestions of all four fixtures for every viewer
   context (anonymous, a non-member, a member, governance, each owner) at
   the pre-change commit, and commit them as test data.
1. **I7.** EdgeAck, the snapshot bump and the template-schema additions
   round-trip; the activation constant matches the model version at
   activation.
2. **Authority matrix (cn-perm).** EdgeAcknowledge cells: the owner of the
   named endpoint is allowed when the edge is in their projection. The
   other endpoint's owner, governance, a facilitator, and a plain member
   are denied. An absent edge, a hidden edge incident to the submitter, a
   non-endpoint entity, and a non-owner all return the same
   `target_missing`. On the submit path, an EdgeCreate stamped below the
   activation version, and one whose `edge.schema_version` differs from
   `op.schema_version`, return `pre_activation_edge`. A payload
   `responsible_human` that differs from the op's returns
   `provenance_mismatch`. An in-app EdgeCreate carrying an intake block is
   refused. An HLC wall time outside `now_ms` plus or minus the skew
   returns `clock_out_of_range`.
3. **Convergence (cn-store).** A proptest (proptest is already a cn-perm
   dev-dependency, core/crates/cn-perm/Cargo.toml, and becomes one for
   cn-store) generates random interleavings of Accepted and Refused, with
   varied `up_to` and `quiet_until`, for both endpoints. Shuffled and
   duplicated, the sequences fold to the same `edge_acks`, the same
   `edge_created` and the same derived status. Any Refused yields refused.
   Refused then Accepted yields refused, and so does Accepted then
   Refused. The existing `convergence_over_reordering_and_duplication`
   (cn-store tests/blueprint.rs) gains answer ops.
4. **Projection property (cn-perm).** A proptest generates random answers,
   quiet periods, grandfathered and post-activation edges, unowned person
   and collective endpoints, multi-ask histories between the same pair,
   third-party introductions, and every viewer class. Its main assertion:
   for every viewer other than the refuser, the edges, `asks` and
   introduction suggestions are byte-identical whether the refuser refused
   (with or without a quiet period) or never answered. It also checks D2's
   status order and cap against an oracle written from this ADR's text.
5. **Pending.** A pending tie between two people is absent from
   `Projection.edges` for every viewer: both owners, the asker, a member,
   governance, and anonymous. It appears in `asks` only for the owners and
   the asker, and only when both endpoints project to that viewer. A tie
   to an unowned person record stays pending. An intake block counts only
   for the respondent's own endpoint and only when its `record_id` matches.
6. **Indistinguishability examples.** Two asks between the same pair; a
   third-party introduction after a refusal; a suppressed re-ask where the
   asker did not self-accept (the asker sees `awaiting_you`, as when
   unanswered); a confirmed-narrow tie (the third-party asker keeps
   `not_confirmed`). Governance's and any non-party's suggestions are
   identical with and without a pending or refused ask. Governance's store
   report shows a quarantined EdgeAcknowledge as a count only.
7. **Cap and widening.** A Network edge with one unanswered owned endpoint
   projects at Group. After `Accepted { up_to: Network }` it projects at
   Network. After `Accepted { up_to: Group }`, a governance VisibilitySet
   to Public still projects it at Group. A grandfathered edge, then a
   governance VisibilitySet to Public, still projects at Group.
8. **D4.** A confirmed Private edge is visible to both ends and to no
   member. A Trusted edge is visible to a viewer trusted by both owners,
   not by one. A Private edge attribute is visible to both ends.
9. **Quiet period.** In a relatives-demo pair with two edges, refusing a
   new ask leaves the existing grandfathered `connected_to` in place.
   Within `quiet_until`, a re-ask from the same asker or a third party is
   authorized and suppressed: the refuser sees nothing new. The refuser's
   own ask toward the counterpart is not suppressed. An ask after
   `quiet_until` reaches the refuser as `awaiting_you`. A forward-dated or
   backdated HLC beyond the skew is rejected at submit.
10. **Authorship.** In a test state where an endpoint's owner differs from
    the answer's `answered_by`, the answer is inert and its quiet period
    lapses.
11. **Migration.** Every viewer context's projection of each fixture log
    (atni-convention, fisheries-committee, relatives-demo,
    research-network) matches the golden capture from step 0. A
    grandfathered edge can still be refused. A pre-activation edge between
    two owned people in a group not flagged `synthetic` is pending. The
    activation-version scenario confirms a tie only after both explicit
    answers.
12. **No leak into graph or exports.** Paths, degrees and betweenness are
    identical with and without a pending or refused edge. This pairs with
    the hidden-node invariance tests already on main (4aa654a). Exports
    and baked snapshots contain no EdgeAcknowledge op and no `asks`.

## Open questions

1. Should governance see refusals for abuse review: never, as counts only
   (the interim default here), or in full under a recorded reason? To be
   answered together with ADR-007 open question 10.
2. What quiet period applies when the refuser sets none: no end, one
   season, or a fixed term?
3. Does an organization endpoint need its governance's acknowledgment, as
   the TSDF's "Consent must be obtained from both" suggests, and how does
   that compose with a Council Seal?
4. May the asker withdraw a pending request, and must that withdrawal look
   to the counterpart like any other `not_confirmed`?

## References

- Code on main at 7c45afa, by function: cn-perm authz.rs (`authorize_op`,
  `authorize_edge_create`, `authorize_visibility`, `require_governance`,
  `require_member`), projection.rs (`project_edges`, `edge_visible`,
  `visible_edge_attributes`), viewer.rs (`admissible_circle`,
  `analysis_scope`), rules.rs (`required_circle`), reports.rs
  (`redact_report`), mirror.rs; cn-perm tests/authority_matrix.rs
  (`edge_create_cells`), tests/blueprint.rs; cn-store fold.rs (`fold`,
  `apply_edge_create`, `lww`, `GroupState`), authz.rs (`submit`), op.rs,
  log.rs (`SNAPSHOT_SCHEMA`), tests/blueprint.rs; cn-schema validate.rs;
  cn-model edge.rs (`Edge::new`), lib.rs (`MODEL_SCHEMA_VERSION`,
  `accepts_schema`), time.rs; cn-api wire.rs (`parse_viewer`,
  `reject_unsupported_ops`), lib.rs (`submit_ops`), analysis.rs
  (`suggest_introductions_impl`), tests/hidden_invariance.rs; cn-graph
  index.rs (`GraphIndex::build`), introductions.rs
  (`suggest_introductions`); docs/blueprints/cn-perm.md;
  schemas/group-template.schema.json; fixtures/groups/*.ops.jsonl;
  app/scripts/generate-atni-ops.mjs, generate-relatives-ops.mjs; commits
  49a51cb, 23aacec, c75f904, 0048e77, 7c45afa; branch `s-e2-intake-edges`
  6c92eed.
- Decisions and ADRs: D-015, D-028, D-030, D-033, D-034, D-056.2, D-105;
  ADR-002 A-B2, A-B5, A-B6; ADR-003 A-B4; ADR-005 D5 and D7; ADR-007 D8;
  HANDOFF.md.
- Research findings (docs/research/relatives-discovery-2026-11-11.md):
  R7-F12, R2-F1, R2-F9, R3-F5, R3-F2, R4-F8, R4-F10, R8-F11, R1-F3.
- Panel ideas: sovereign-architect-3 (Two-Sided Relations with sealed
  contents), individual-at-center-3 (Asked and Answered),
  knowledge-keeper-5 (Honorable Ask), sovereign-architect-1 (Kinship
  Compact), sovereign-architect-6 (Council Seal); the three judges'
  requirements in the 2026-11-11 discovery scores.
- TSDF (owner's framework): Standard v1.0 (CC-BY-SA 4.0) sections 3.1 (T1,
  T3) and 5.1; Tier Classification Decision Guide, Historical Data;
  isdGRAPH Schema Reference, core design principles; Federated Indigenous
  Data Protocol v1.0 (Apache 2.0) section 3.3 (ConsentRecord, the
  documented-consent shape the op envelope plays here).

## Review log (2026-11-11)

One review round ran on the first draft. Three lenses, each reading the
ADR against the code and the TSDF text, plus a Jev citation check:

- **P - permission soundness and non-leakage** (main at 6db3c24 plus the
  relatives/perm commits). Verdict: not acceptable as written.
- **S - TSDF fidelity, sovereignty and harm.** It checked this ADR's TSDF
  quotations against the reference text and found them accurate. Verdict:
  revise before acceptance.
- **C - fit to the codebase and its invariants** (main at 0048e77, moved
  during review). It confirmed the fixture census and found the refusal
  fold rule sound and convergent. Verdict: revise before acceptance; land
  this ADR first.
- **Jev check** (model jev-1.14.0, 7 fair controls passed first). Both of
  this ADR's research claims scored supported (R4-F8 0.93, R4-F10 0.98),
  with two manual caveats. This ADR's TSDF quotations were not in the Jev
  batch.

Every blocking finding is applied. Each finding's disposition follows.
Findings marked "both" also appear in ADR-007's log.

Lens P:

- P5 (blocking) The quiet period had no lower bound and hid ties that
  predated the refusal. Applied: an edge is suppressed only when its
  create sorts after the refusing answer; Confirmed and grandfathered
  outrank Suppressed (D2, D3); test 9.
- P6 (blocking) Suppression told the counterpart they were refused.
  Applied: a suppressed tie projects to everyone but the refuser exactly
  as an unanswered one, `awaiting_you` included (D3); tests 4 and 6. For
  an edge with one owned endpoint, the literal rule would republish a
  refused claim to the whole group, so such a suppressed edge stays out of
  `Projection.edges` and refusals of that kind are stated as not private.
- P7 (blocking) Grandfathering could not be built, and one build opened a
  back door. Applied: the marker is `edge.schema_version`; the submit path
  denies a mismatched or pre-activation stamp; `edge_created` is a new
  state map listed under I7; the history-based `up_to` is dropped (D5;
  option 6); test 2.
- P8 (advisory) D2 and D5 disagreed on whether grandfathered ties are
  capped. Applied: grandfathered person endpoints are capped at Group
  until a real Accepted (D2, D5); test 7.
- P9 (advisory, both) The tests were too weak. Applied: test 4 is a
  projection proptest over random answers, quiet periods, grandfathering
  and viewer classes, with the refused-versus-unanswered assertion across
  multi-ask histories.
- P16 (advisory) The asker's identity and the answer path could be bent or
  probed. Applied: the asker is the op's `responsible_human` and a
  mismatched payload is denied; EdgeAcknowledge needs the edge in the
  submitter's projection; `EdgeAck.answered_by` is checked against the
  current owner at read time (D1); test 2. The revision-bump channel is
  stated as a limit, not closed.
- P17 (advisory) The protection was narrower than claimed. Applied:
  unowned person endpoints cannot answer, so ties to them stay pending;
  suggestions are withheld only from those who know of the ask; the
  Consequences claims are narrowed (D1, D2, D3).
- P18 (advisory, both) Both ADRs claimed the next patch. Applied:
  `EDGE_CONSENT_ACTIVATION = 0.1.2`, with ADR-007 at 0.1.3, and generators
  write explicit answers once they move to the activation version (D5).

Lens S:

- S1 (blocking, both) Unowned person records were treated as group data,
  so no real respondent was protected. Applied: the template `subject`
  field, a fail-closed default, unowned person endpoints that count as
  never answering, and the intake self-acceptance for the respondent's own
  endpoint only (D1, D2). The governance-archive withdrawal path for
  unowned respondents is recorded as owed (Consequences).
- S14 (advisory) Grandfathering as Accepted conflicted with TSDF's
  historical-data default. Applied: grandfathering applies only in groups
  whose template is flagged `synthetic`; real pre-activation edges are
  pending; governance or ATNI Climate may review; a grandfathered tie is
  never an acknowledgment for crossing graphs (D5).
- S15 (advisory) Answers keyed by entity were never re-checked against the
  owner. Applied with P16 and C13 (D1).
- S16 (advisory) Governance store reports carried refusals unredacted.
  Applied: quarantined answers appear as counts only until open question 1
  is answered, and answers add no custody event (D3); test 6.

Lens C:

- C1 (blocking) The grandfather rule keyed on an op stamp that state does
  not hold. Applied with P7.
- C3 (blocking) Withholding suggestions from every viewer leaked asks to
  governance, and "not built" was false. Applied: the withheld set is per
  viewer and covers only the asker and the endpoint owners; it is a new
  input to `suggest_introductions`; context item 7 records that
  introductions are built and wired (D3); tests 0 and 6.
- C4 (blocking) The quiet period had no lower bound and needed times
  cn-perm cannot read. Applied with P5: `EdgeAck.sort_key` and
  `edge_created` give both times from state (D1, D5); test 9 uses a
  relatives-demo pair with two edges.
- C5 (blocking, both) The version bumps were not coordinated. Applied:
  the named activation constant, ADR-007 at 0.1.3, and a statement that
  the check reads `edge.schema_version`, never `template_version` (D5).
- C12 (advisory) "The visibility the edge holds under its pre-0.1.2 ops"
  cannot be derived. Applied: dropped in favor of the Group cap (D5).
- C13 (advisory) Answer authorship was never re-checked, and the sentence
  claiming otherwise was backwards. Applied: `answered_by` and the
  read-time check; the sentence is replaced (D1); test 10.
- C14 (advisory) A forward-dated HLC escaped the quiet period. Applied:
  the submit path bounds HLC wall time by `now_ms` plus or minus a skew,
  and cn-api passes `now_ms` through; the enforcement claim is narrowed to
  match (D3, Consequences); tests 2 and 9.
- C15 (advisory) A third-party asker could tell a narrowly confirmed tie
  from a refused one. Applied: such an asker keeps `not_confirmed` (D3);
  test 6.
- C16 (advisory) The protection excluded every real pilot person. Applied
  with S1. Denying such edges at submit was declined, because the denial
  would reveal whether a visible record has an owner (option 7).
- C17 (advisory) Migration omitted test helpers and generators. Applied:
  D5 lists the fixture templates, generator pins, Rust test helpers and an
  activation-version scenario; test 11.
- C18 (advisory) The facilitator was named as the reviewer of real
  historical edges. Applied: governance or ATNI Climate reviews, and the
  `synthetic` flag makes the rule enforceable in code (D5).
- C20 (advisory, both) Stale anchors. Applied: code is cited by function on
  7c45afa; Depends-on names 49a51cb and 23aacec on main; the mirror is
  described as built.
- C21 (advisory, both) Missing test cases and order. Applied: golden
  captures first, then I7, authority matrix, convergence, projection and
  leak tests, with the six missing cases added (tests 2, 6, 9, 10).

Jev check:

- J3 (caveat on R4-F8, supported 0.93) "then step out" is in the digest's
  note, not the finding. Fixed: the research list attributes the
  introduce-and-step-out reading to the note.
- J4 (caveat on R4-F10, supported 0.98) The durability caution is the
  digest's note, not the finding. Fixed: the research list attributes it
  to the note.
- No claim in this ADR scored weak or unsupported.
