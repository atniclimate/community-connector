# Integration Plan: Applying the Graph-Networks Research to Community Navigator

> Produced 2026-07-06 by an adversarial multi-model design panel, then reviewed
> by Opus 4.8 at maximum reasoning effort. Feeds plan v3 (the D-040 sitting).
> This is a recommendations + technical-spec document, NOT yet an accepted ADR;
> the two human gates in Section 9 must be answered before the affected work
> starts. Conventions: hyphens, never em dashes.
>
> Reading order: Section 0 (the recommendation), Section 3 (the keystone), then
> Sections 4-6 for the buildable detail. Companion: the research report at
> docs/research/graph-networks-report-2026-07-06.md; Execution Plan v2 at
> docs/PROJECT_PLAN.md section 3.

## 0. The recommendation in one paragraph

Build the convention pilot on the **minimalist base**: the routing and explore
hero workflows are mostly wired already, so the pilot's real code is small and
bounded. But the panel found four things the first-cut plan got wrong or
skipped, and all four must be fixed before implementation:

- **Vocabulary.** The capability vocabulary must be **authored by ATNI Climate
  in its own words first**, not seeded from a US settler taxonomy - which also
  dissolves a real software-license conflict (Section 3).
- **Routing.** The routing workflow is **not** "UI-only" - it needs a genuine
  "need-term + asker to candidate paths" contract and a **contactability-consent
  gate enforced in the query layer** (not the UI) so it never overpromises or
  exposes someone who did not consent (R3, spec 6.2).
- **Ingestion.** It must be **idempotent under event-sourcing** via a versioned
  intake contract with deterministic, source-derived identity for entities,
  edges, and custody events - or same-day re-imports silently duplicate data
  (R2, spec 6.1).
- **Comprehension.** The **assembly comprehension layer** - a flat/list reveal
  projection, a "how to read this" primer, a facilitator script, and seeded
  Stories - is the difference between "they see it work" and a beautiful graph
  nobody in the room can read, and it is currently nobody's line item (R6).

The sibling tools (cap-assessor, TCR-policy-scanner, GeoBase,
engagement-database) integrate **after** the pilot; the pilot's job is to build
the one CSV importer and the one shared vocabulary that make cap-assessor a
near-free follow-on.

## 1. How this was produced (the adversarial panel)

- A codebase-exploration pass produced a shared **integration surface map**
  (`.codex/panel-codebase-map.md`): the real crates, seams, and constraints.
- Two proposers argued opposite stances from the same ground truth: **MIN**
  (pilot-first minimalist, `.codex/panel-proposal-min.md`) and **MAX**
  (ambitious spine integrator, `.codex/panel-proposal-max.md`).
- Two critics attacked both proposals: **Codex** (gpt-5.5, high effort, session
  019f3a59-91c9-7830-97f3-823a0c21069c, `.codex/panel-codex-critique.md`) on the
  engineering, and a **Fable skeptic** on sovereignty, delivery-risk, and
  fidelity to the human's decisions.
- The director (Fable) synthesized this document; **Opus 4.8 at max effort**
  reviewed it for articulation and technical-spec completeness before it was
  committed.

## 2. The verdict: MIN is the base; MAX contributes one design note

Both critics independently concluded MIN is the safer base: it hugs the human's
decisions (D-019..D-040), hits the convention date with two bounded code items,
and maps almost 1:1 to the D-022 win condition. MAX's genuine contribution is a
single warning, absorbed here as a design note on the importer: **name the
forward-compatible seam so v0.1 does not foreclose the spine, but do not build a
general interchange DTO for the pilot.** MAX's ranking of engagement-database as
a near-first integration is explicitly rejected - it creates schedule and PII
pressure toward data the human walled off (D-030 membership = form respondents
only, D-031 real-data migration is human-gated, and the AGENTS.md red-data
rule; D-026 is separately the no-remotes/backup decision).

## 3. The keystone: sovereignty and licensing are the same fix

This is the most important finding, and it changes D-023's direction. Both
proposals seeded the capability vocabulary from **Open Eligibility**, a US
health-and-social-care taxonomy. Two independent objections converged on it:

- **Sovereignty (Fable skeptic).** The research report's Section 3 does not say
  "extend" answers the ontology-can-colonize critique; it says interoperability
  must be "negotiated, scoped, or **refused**," with refusal a first-class
  operation. Baking a settler vocabulary in *before ATNI Climate sits* grants a
  **vocabulary authority the human never delegated** - and D-034 already
  reserves the analogous *tier* authority to ATNI Climate, not the developer.
- **Licensing (Codex).** Open Eligibility is CC BY-SA (a ShareAlike license -
  the exact version is unconfirmed and does not matter here, since the conflict
  holds for any BY-SA version); the project license is PolyForm Noncommercial
  1.0.0. ShareAlike + attribution obligations, and CC BY-SA's bar on additional
  restrictions, make embedding adapted Open Eligibility JSON into a PolyForm
  repo a likely license conflict.

**Both dissolve with one move.** The report itself (Section 3-bis) notes Open
Referral's **HSDS is taxonomy-agnostic by design** - you adopt the
`provider -> program -> service -> tag` *structure* without adopting anyone's
*terms*. So:

> **Sovereignty-first vocabulary.** ATNI Climate names its capability categories
> in its own words first (facilitator-elicited, under the D-034 collective FPIC
> checkpoint). We adopt the HSDS *structure* (a controlled offer/need tag set +
> a situation/eligibility facet), not Open Eligibility's *terms*. Any later
> mapping to Open Eligibility (for FHIR/interop) is a refusable, back-office,
> separately-licensed concern - never a precondition. Refuse/replace is the
> default posture; extend is second.

This reserves vocabulary authorship to ATNI Climate exactly as D-034 reserves
tiering, keeps the repo cleanly PolyForm, and still leaves the FHIR interop
door open for later - without betting the pilot on a vocabulary the community
did not choose.

## 4. The v0.1 pilot integration plan (ranked recommendations)

Each recommendation: the seam (crate:file from the map), the smallest correct
version, the commitment/decision it serves, and effort routing. "grind" =
Codex mechanical implementation from a blueprint; "director" = Fable designs
first.

**R1 - Author the ATNI template with ATNI's own capability vocabulary.**
Seam: `schemas/group-template.schema.json` + a new `fixtures/templates/
atni-climate.template.json`, modeled on the routing skeleton already in
`fisheries-committee.template.json` (`need` + `skill_resource` kinds joined by a
`need_met_by` edge). The offer and need attributes MUST draw from **one shared
`tags` vocabulary** (Section 6.2 explains why the fisheries template's untagged
`need` breaks routing). Code-free (`enum`/`tags` value types already exist - no
ADR-001 D4 amendment). Serves commitment #5, D-023 (redirected per Section 3),
and unblocks both heroes + the importer column map. Routing: director authors
the structure; the *terms* come from ATNI Climate (gate G1). This is the
keystone artifact.

**R2 - cn-ingest as a narrow, versioned, idempotent importer.**
Seam: `core/crates/cn-ingest` (empty). Build one contract, `AtniIntakeBatchV0_1`
(spec in 6.1), NOT a general `IngestBatch` DTO. It carries `schema_version`
(I7), a stable **source-row id** per record, and **idempotent re-import
semantics**: a re-seen source id emits `AttributeSet`/edge-update ops, never a
second `EntityCreate` (Section 6.1 explains why the event-sourced fold makes the
naive version silently duplicate data). It stamps `Origin::Ingested{source}` +
`SensitivityTier::T1` (D-034) and appends an `Imported` custody event (I12).
CLI: `cn ingest` + `cn snapshot` only. Serves the pilot arc (ingest -> reveal)
and D-030 (same-day QR joiners). Routing: director blueprint (the semantics are
subtle) -> grind implements -> adversarial round. This is the one genuine
critical-path build.

**R3 - Complete the routing capability properly (not "UI-only").**
Seam: `core/crates/cn-graph/query.rs` + `cn-api` + the frontend. Codex proved
the gap is more than UI: `PathRequest{from,to,constraints}` needs concrete
endpoints and `search` returns attribute hits, not "who can help with need X."
The minimum correct version (spec in 6.2): needs and offers share a searchable
`tags` attribute (R1); the frontend resolves a chosen need-term to
capability-bearers via `search`, then calls `query_paths` with explicit
`allowed_edge_kinds`; the returned **path is the result** (commitment #3, no
opaque score, no LLM). Two hard gates from the skeptic (and Opus): (a) endpoint eligibility is gated on
**contactability consent** (D-023 principle 4) - a person who consented only to
facilitator-mediated contact must never be returned as a directly reachable
endpoint. Per I2 and ADR-001 A-B1 this gate is **structural, in the query
layer** (cn-graph/cn-api candidate resolution), NOT a UI render rule - a
non-consenting person is still present in the projection and would otherwise be
returned by `search`/`query_paths` and leak through the flat list (R4) and any
export (spec 6.2). (b) Every path is framed in the UI as "a possible pathway to
explore, brokered by the facilitator," and **no "need-met/closed" state is
built** (caution #2; NCCARE360 88%->30% on funding). Routing: director designs
the contract + the structural consent gate -> grind implements the cn-graph/
cn-api candidate filter + the frontend.

**R4 - Explore surface, including a flat reading projection for the room.**
Seam: the frontend over already-wired `search`/`entity_detail`. A search box
over `cn-graph search` (offer/need tags are searchable) + a detail panel showing
the **TSDF code primary** (D-032) with a **one-line provenance** string for
members (D-033). Plus - new, from the skeptic - a **flat list/table reading
projection** for the general-assembly reveal, because a pure-3D reveal to a room
is the documented failure mode (Net-Map flattened 3D to 2D for group settings;
report Section 5). This flat projection is also the down-payment on the deferred
accessibility parallel-DOM (D-035), so it is not throwaway. Serves commitment
#1 and #4. Routing: grind from a director sketch.

**R5 - GroupRole::Facilitator with an explicit authority matrix.**
Seam: `cn-model/group.rs` + `cn-perm` (`viewer.rs`, `authz.rs`, `rules.rs`).
Codex showed "one variant + one predicate" is understated: a third role touches
membership serialization, viewer reach/fingerprints, and every write gate. The
minimum SAFE version (spec in 6.3): facilitator can create/import unowned pilot
records and author stories, but cannot govern membership, loosen visibility,
lower a tier, or bypass owner controls - with an explicit cn-perm authority
matrix, exhaustive tests, and projection-cache fingerprint updates. Serves
D-028 (role now, creator-governance handoff later). Routing: director blueprint
-> grind HIGH -> mandatory adversarial round (permission-adjacent).

**R6 - The assembly comprehension layer as a named critical-path deliverable.**
Seam: mostly frontend + docs, plus Stories (core already silent-elides steps).
The graph-literacy caution is measured in a community like ATNI's; the D-022
win *depends* on the assembly landing. Fund four things as critical path, not
polish: (a) an in-product **"how to read this" primer**; (b) the **flat reading
projection** (R4); (c) a **facilitator assembly-reveal script** as a written
deliverable (choreography of the reveal); (d) **seeded and tested Stories**
(D-037) drawn from intake-form story material - guided tours are the *measured*
remedy for lay-audience illiteracy (report Section 5, NetworkNarratives).
Serves commitment #2 and the win condition directly. **Definition of done
(observable, not asserted):** a lay-reader dry-run / facilitator rehearsal of
the full reveal against the primer, the flat projection, and the seeded Stories,
before the convention - since the D-022 win depends on the assembly landing, it
must be rehearsed, not hoped.

## 5. Must-resolve issues and their resolutions

| # | Issue (source) | Resolution in this plan |
|---|---|---|
| 1 | Vocabulary sovereignty violation (skeptic) | Section 3 keystone + R1 + gate G1 |
| 2 | Open Eligibility CC BY-SA vs PolyForm license conflict (Codex) | Section 3 keystone (don't embed terms) + gate G2 |
| 3 | Routing is not "UI-only"; no term-to-path contract (Codex) | R3 + spec 6.2 |
| 4 | Routing overpromise / non-consented contactability (skeptic) | R3 consent gate + facilitator-brokered framing |
| 5 | Re-ingest not idempotent under event-sourcing (Codex) | R2 + spec 6.1 (source-row id, AttributeSet-on-reimport) |
| 6 | Assembly comprehension layer unbudgeted (skeptic) | R6 (promoted to critical path) |
| 7 | Custody event not appended on import (Codex) | R2 stamps an `Imported` custody event (I12) |
| 8 | Snapshot budget hand-waved (Codex) | Spec 6.4 (taxonomy data out of the critical bundle) |
| 9 | General DTO over-reaches the pilot (skeptic) | R2 builds `AtniIntakeBatchV0_1`, not a spine DTO; MAX's warning kept as a design note only |

## 6. Technical specifications

### 6.1 `AtniIntakeBatchV0_1` - the intake contract
- **Location:** `core/crates/cn-ingest/src/lib.rs`; consumed by `core/cli` `cn
  ingest`; emits `cn_store::Operation`s through `cn-api` load/submit.
- **Envelope (serde JSON):** `{ schema_version: "0.1.0", source: string,
  imported_by_human: PersonId, rows: [IntakeRow] }`. `schema_version` satisfies
  I7 (readers reject unknown majors).
- **IntakeRow:** `{ source_row_id: string (STABLE per person across re-imports),
  kind: KindId, attributes: {AttrId: rawvalue}, edges: [{edge_kind, target_
  source_row_id, weight?}] }`. `source_row_id` is the idempotency key - e.g. a
  stable hash of the form-response id, NOT the person's name.
- **Deterministic identity (the correctness core; corrected per Opus review).**
  There is NO home for a `source_row_id -> EntityId` map in group state (Entity
  has no `external_id`, no `OpKind` variant records such a map, `GroupState`/
  `SnapshotParts` have no such field). So identity is **derived, not stored**:
  compute all ids as **deterministic UUIDv5** via `Id::from_uuid(uuid_v5(
  namespace = group_id, name = source_row_id))` for entities, `uuid_v5(group_id,
  edge_kind ++ from_source_row_id ++ to_source_row_id)` for edges, and a
  first-sight-only deterministic id for the `Imported` custody event. (Note:
  this is **UUIDv5, deliberately NOT the UUIDv7** that `EntityId::new` produces
  for interactive ops - the importer needs reproducibility, not time-ordering.)
  First-vs-subsequent sight is then decided purely by whether that id already
  exists in the folded projection - no external table required.
- **Re-import semantics.** The fold dedups only by `op_id` and quarantines a
  second `EntityCreate`/`EdgeCreate` on a repeated `EntityId`/`EdgeId` as
  `DuplicateCreate`, and dedups custody by `event.id` (`cn-store/fold.rs`).
  Therefore, uniformly for entities, edges, AND custody: **first sight of a
  derived id** -> `EntityCreate`/`EdgeCreate` + `AttributeSet`s + the single
  `Imported` custody append; **subsequent sight** -> `AttributeSet` only for
  attributes whose value differs from the **prior imported snapshot**
  (see baseline below), edge create/remove diffs by derived `EdgeId`, and
  **no second `EntityCreate`, no re-appended custody event**. This is what makes
  "fast idempotent re-run" (D-030) actually idempotent under LWW.
- **Attribute-diff baseline.** The importer diffs each re-import against the
  **prior imported source snapshot for this `source`** (the CSV as last
  imported), NOT against current folded state. This is essential: field-LWW by
  HLC sort_key means diffing against folded state would let a re-import silently
  clobber an intervening member/facilitator edit. Only genuinely changed source
  cells emit `AttributeSet`.
- **Provenance/tier:** each entity constructed with `ProvenanceEnvelope::new(
  Origin::Ingested{ source }, recorded_by = the importer actor, responsible_
  human = imported_by_human)` (valid per `provenance.rs` - agent recorder with a
  separate responsible human is permitted), tier `T1` (D-034), plus the single
  first-sight `Imported` custody event (I12).
- **Validation:** every op is re-validated by cn-schema at fold; the importer
  surfaces the `ValidationReport` (rejected/quarantined rows) to the facilitator
  rather than failing silently.
- **Acceptance criterion (required test):** a round-trip property test -
  import a batch, then re-import the identical batch, and assert the folded
  state is byte-identical: **zero** new `EntityCreate`, **zero** duplicate
  edges, **zero** custody growth. This test pins the semantics above and joins
  the existing per-crate `tests/blueprint.rs` property-test culture.
- **Explicitly deferred to ADR-005 (Session C):** fuzzy dedup, merge-vs-link,
  split/tombstone. v0.1 handles only exact `source_row_id` identity.

### 6.2 Routing query contract - the missing seam
- **Problem:** `query_paths` needs concrete `from`/`to` entity ids;
  "who can help with need X" has neither a `to` nor a way to find candidates.
- **First decide the offer-modeling question (R1 owns this).** `search` returns
  the tag-bearing entity, so the candidate endpoint depends on where the offer
  tag lives: **offer-as-tag-on-Person** (the person is the endpoint, the readout
  and the consent gate attach directly) versus **offer-as-resource-node** (the
  fisheries template's `skill_resource` pattern - the endpoint is a resource
  node and you must resolve the offering Person behind it before rendering or
  gating). Recommendation for the pilot: **offer-as-tag-on-Person** for the
  routing hero (simplest, person is the endpoint), keeping resource nodes
  optional for richer asset modeling later. 6.2 assumes Person endpoints.
- **v0.1 contract (no new Rust if the template cooperates):** (1) R1 gives every
  offer and need a shared `tags` vocabulary on Person; (2) the frontend calls
  `search` with the chosen need-term over offer-bearing Persons to get
  **candidate endpoints**; (3) for each candidate it calls `query_paths(from =
  asker, to = candidate, constraints{ allowed_edge_kinds = [collaborates,
  member_of, need_met_by, ...], max_hops })`; (4) it renders the returned node
  sequence as "A -> B -> C" plus the 3D path highlight (reuse focus-mode dual
  color buffers). If profiling shows the per-candidate loop is too slow, add ONE
  cn-graph function `paths_to_capability(asker, tag, constraints)` (director
  blueprint) - a bounded, well-scoped addition, not a redesign.
- **Consent gate (mandatory, STRUCTURAL).** A Person whose `contactability`
  attribute is "no" or "facilitator-only" (D-023 principle 4) is **never**
  returned as a directly reachable candidate endpoint. Per I2 (permission logic
  lives only in the engine; app-layer permission logic is a blocking finding)
  and ADR-001 A-B1 (projection is the only read path), this filter lives in the
  **cn-graph/cn-api candidate-resolution step, not the UI** - because the person
  is still present in the projection and would otherwise be returned by `search`
  and leak through the R4 flat list and exports. Model it as a routing-query
  concern (a candidate-eligibility predicate on contactability), so there is a
  single structural chokepoint. The UI stays presentation-only. (For v0.1's
  single-facilitator/single-laptop reality a UI-only gate would be operationally
  survivable, but the "never" guarantee earns its structural home.)
- **Framing (mandatory):** label results "a possible pathway to explore -
  the facilitator can help make the connection." No auto-contact, no "need-met"
  state, ever (caution #2).

### 6.3 `GroupRole::Facilitator` authority matrix
- **Add:** `GroupRole::Facilitator` (`cn-model/group.rs`) + predicate
  `is_facilitator_or_governance` beside `is_governance` (`cn-perm/viewer.rs`),
  threaded through `authorize_op` (`cn-perm/authz.rs`).
- **READ reach (state it explicitly - the no-leak property is a READ property).**
  `is_active_member` returns true for any active role and `admissible_circle`
  grants `Circle::Group` to any member (`cn-perm/viewer.rs`), so a Facilitator
  **inherits member-level read (Group circle) by default** and is **NOT**
  `is_governance` - therefore no governance-only counts and no full-custody
  chains leak to a facilitator (D-033: full chain is governance-only). Because
  facilitator read == member read, the fingerprint work below is a safety belt,
  not a new read tier.
- **CAN (v0.1 write):** create/import unowned pilot entities and edges;
  author/edit stories; set attributes on facilitator-created records.
- **CANNOT (reserved to Governance, per D-028 "creator-governance later"):**
  add/remove memberships; grant roles; loosen a value's visibility; lower a
  tier (tightening only, and tier authority is ATNI Climate per D-034); bypass
  owner controls on member-owned data. **Note - custody is a real tightening:**
  the current rule lets any member append custody to any visible target
  (`cn-perm/authz.rs`); restricting facilitator custody-append to
  facilitator-created records is therefore NEW role/ownership logic the matrix
  must specify, not an existing rule to inherit.
- **Required with it:** an explicit authority-matrix doc (role x op-kind),
  exhaustive authz tests + property-strategy update (the no-leak property must
  still hold with three roles), and a fix to `active_role_names`
  (`cn-perm/viewer.rs`) - today a **non-exhaustive two-arm match** on
  `GroupRole`, so adding `Facilitator` forces a new arm or a member/facilitator
  fingerprint collision - plus confirming `viewer_fingerprint` distinguishes the
  role so the projection/index cache keyed by it (`GroupSession` in
  `cn-api/src/session.rs`) does not serve a facilitator a member's cached
  projection. This is why it is grind HIGH + an adversarial round, not a
  one-liner.

### 6.4 Snapshot budget discipline (I8)
- The Rust CLI importer does not touch the WASM snapshot, but seeded vocabulary
  JSON, embedded Stories, and demo ops do. Rule: **third-party/seed taxonomy
  data and full custody chains stay out of the critical HTML bundle** - the
  snapshot embeds only the projected, member-visible data. Every seed-data or
  Stories recommendation is gated by `npm run build:snapshot` +
  `scripts/check-size.mjs` before it counts as done (existing I8 discipline).
- **Which viewer's projection is baked in (load-bearing for size AND
  sovereignty).** The projection is viewer-scoped and is the only read path, so
  the shipped single-file snapshot must embed the projection for a **specific,
  deliberately chosen viewer context - the Anonymous/Group-member view, never
  the facilitator's or any Trusted/Private-bearing view** - or a room-shown file
  could embed attributes above the audience's circle. State the snapshot's
  viewer scope explicitly in the build.
- **Rough budget (so the rule has a number).** Pilot scale is ~120-300 Persons
  x ~10-15 attributes + edges + one-line provenance summaries (full custody
  stays out per above). At the base renderer's measured 0.55MB for 120 entities,
  a 2-3x participant increase plus Stories and the flat-list data is comfortably
  under 5MB - but confirm with `check-size.mjs` once the ATNI template and real
  participant count are known; do not assume.

## 7. Sequencing against Execution Plan v2

| Rec | Execution Plan v2 home | Change from v2 |
|---|---|---|
| R1 template + vocabulary | FORM deliverable | Redirect: ATNI authors terms first (was: seed Open Eligibility) |
| R2 importer | S4-A | Add: versioned idempotent contract + custody (was: "CSV importer") |
| R3 routing | S3-B | Add: term-to-path contract + consent gate (was: "routing UI") |
| R4 explore + flat projection | S3-B | Add: flat reading projection for the assembly |
| R5 facilitator role | Session B | Add: explicit authority matrix + fingerprint work |
| R6 comprehension layer | S3-C + FORM | Promote: from "author stories" to a named critical-path deliverable set |

No new phases; the changes are corrections and one promotion (R6) within v2's
existing sessions. Session C (ADR-005 dedup) absorbs the deferred merge/link
semantics R2 explicitly leaves out.

## 8. Sibling-tool integration roadmap (post-pilot)

Explicitly **none** integrate during the pilot. The pilot builds the two things
that make the first integration near-free: the R2 importer and the R1 shared
vocabulary.

1. **cap-assessor (first, v0.2).** Richest structured source of needs/capacity;
   emits extracted rows through the R2 importer (same contract), stamped
   `Origin::Ingested` + tier per ATNI-Climate authority (D-034), FPIC-gated
   (D-031). Precondition: it emits into the ATNI-authored vocabulary (R1), not
   its own.
2. **TCR-policy-scanner (v0.2+).** T0 public program/eligibility data ->
   Funder/Program nodes + `eligible_for` edges; a new routing target ("who is
   eligible for program X"). Derivative - sequences after the need side exists.
3. **engagement-database (v0.2+, consent-gated).** A pre-built edge set, but the
   pilot's form-respondents-only rule (D-030) and the OUTBOUND_GATE make it an
   enrichment/outreach source under the Session E migration recipe, not an
   auto-ingested identity set. Never fed during the pilot.
4. **GeoBase (v0.2+).** The geography bridge; presupposes the Place-kind +
   proximity work deferred from the pilot. CN place-nodes resolve against
   GeoBase layers; shared TSDF tiers. H3/S2 privacy aggregation is the later
   lever.

## 9. Open gates for the human (new, from the panel)

Both are genuinely human-reserved (sovereignty + license), so they park here
rather than being decided autonomously:

- **G1 - Vocabulary authority.** Confirm that ATNI Climate authors the
  capability categories (offer/need/situation terms) in its own words, the way
  D-034 reserves tier authority - with the facilitator eliciting them under the
  collective FPIC checkpoint. **How, in practice** (the FORM deliverable should
  specify): a facilitator-led co-construction session - a card-sort or
  structured interview in the community's own language - converted into template
  JSON by the facilitator/developer, timed *before* the intake form goes out.
  This is exactly the Net-Map participatory-mapping method the research endorses
  (report Section 5): the vocabulary is built *with* the committee, not
  presented to it. Default if unanswered: the FORM deliverable drafts a
  *structure* (HSDS-shaped, empty of imposed terms) and parks the terms until
  ATNI Climate fills them; no settler vocabulary is committed.
- **G2 - Open Eligibility / FHIR mapping.** If a later mapping from ATNI's terms
  to Open Eligibility (for FHIR/interop) is wanted, it must be isolated as
  separately-licensed (CC BY-SA) third-party data with attribution and a NOTICE,
  kept out of the PolyForm-licensed code. (Verify the exact CC BY-SA version
  before this reaches counsel; the conflict holds for any version.) Default: no
  mapping in v0.1; revisit when interop is actually needed.

## 10. What we deliberately are NOT doing for the pilot (and why)

- **A general interchange/spine DTO** - over-reaches the pilot (skeptic); build
  the narrow `AtniIntakeBatchV0_1` and keep the seam name only.
- **Geography as structure / Place kind / spatial index / H3-S2 / GeoBase** -
  needs code + an ADR; keep the `Geo` attribute; v0.2 (both proposers agree).
- **Link prediction / embeddings / GNNs / community detection** - caution #2;
  incomplete graphs yield uninsightful predictions; `shortest_path` + tag-search
  is the honest v0.1.
- **LLM natural-language-to-query** - graph query first, LLM second and
  parameterized-only; the pilot uses a structured taxonomy picker, no model.
- **Any external graph engine or Cypher/GQL/SPARQL surface** - caution #3 (Kuzu
  archived mid-2025); the Rust core is the engine.
- **Sibling-tool feeds, personal mode, identity, deep accessibility, tier-
  enforcement tooling** - deferred per D-026/027/029/034/035.

## Appendix: panel artifacts

Held in `.codex/` (gitignored scratch); the durable record is this document.
- `panel-codebase-map.md` - the integration surface map.
- `panel-proposal-min.md`, `panel-proposal-max.md` - the two proposals.
- `panel-codex-critique.md` - Codex gpt-5.5 critique (session
  019f3a59-91c9-7830-97f3-823a0c21069c).
- The Fable skeptic critique and the **Opus 4.8 max-effort review** are folded
  into this document. The Opus review caught three must-fix errors in the first
  synthesis (an unimplementable idempotency mechanism in 6.1, a nonexistent file
  reference in 6.3, and a consent gate placed at a layer I2 forbids in 6.2), all
  corrected above; its verdict on the corrected document is
  "commit-after-must-fixes."
