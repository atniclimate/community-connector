# Relatives across graphs: what Community Navigator is for, what works, what remains, and what becomes possible

> Written 2026-11-11 by the director at the end of an exploration session the owner
> opened with this charge: take a deep look at the tool, name its primary goals,
> treat the convention goals (remote capture, intake) as important but not the core,
> and explore what a system of networks of practice, organizations, institutions,
> Tribes, and affinities could become when each person is the center of their own
> view, each entity shares as little or as much as it chooses, and consenting
> entities serve as interlinkages between graphs, so that connections are made as
> relatives. The owner relaxed the sovereignty guards for the exploration. Evidence
> for every research claim below is in
> `docs/research/relatives-discovery-2026-11-11.md` (finding ids such as R1-F2).
> Code receipts are path:line at commit `1abedc2` unless a later commit is named.
> The two decisions this document proposes are written up as ADR-007 and ADR-008
> (both Proposed). Docs use hyphens, never em dashes (I10).

## How this was made

Two workflows ran on 2026-11-11.

- **Discovery** (24 agents, about 28 minutes): five read-only audits of the code
  and the plans; eight web research topics (Tribal coordination needs, intertribal
  network exemplars, Indigenous relational frameworks, network science of
  collective action, sovereign federation technology, the AI-on-graphs frontier,
  products and harms from shared network data, and consent design for the
  individual); six ideation lenses (an emergency coordinator, a knowledge keeper,
  a network weaver, a sovereign data architect, a frontier strategist, and an
  individual at the center of their own graph) that produced 48 ideas; three
  judges (community value, architecture fit, sovereignty and harm); and two Jev
  batches that fetched each of 40 load-bearing sources and checked whether the
  passage supports the claim. 36 held; four were weak and are corrected in the
  register (R2-F4, R3-F10, R5-F11, R7-F4).
- **Build** (see "What this session built" below): the confident improvements,
  each implemented, verified by an independent claim-verifier, and committed, plus
  the two ADRs drafted, adversarially reviewed through three lenses, and checked
  by Jev for fidelity to the TSDF text.

A fresh baseline before any change: Rust 287 tests passed, 0 failed, 1 ignored
(41 test binaries); the app 207 of 207.

## 1. What the tool is for

Community Navigator exists to make the connections between people, places,
knowledge, and need visible and usable, in a way that strengthens sovereignty
instead of eroding it (`docs/THE_STORY.md`). The convention work proved the
pieces: a permission-filtered projection as the only read path, an event-sourced
log, typed templates, a facilitator intake path, and a sealed remote form. Those
are the on-ramp. The destination is a web of relatives that can find each other
across institutions.

Three facts from this session's research explain why that destination is urgent
now, not someday.

- **The people who did this work by hand are gone.** The BIA and USGS Tribal
  Community Resilience Liaison network, whose liaisons "served as extension agents,
  facilitating research, linking Tribal needs to available resources", ended on
  July 17, 2026 after twelve years (R1-F2, R4-F12). NOAA terminated the Northwest
  Climate Resilience Collaborative, whose partners included ATNI, effective May 5,
  2025 (R1-F5). Indian Affairs lost a net 11 percent of its staff in the first half
  of 2025 (R1-F7). Tribal programs facing funding bottlenecks report not knowing
  whom to contact (R1-F8).
- **Tribes have already said what they need.** Northwest coastal Tribes asked for
  agencies to coordinate so that Tribes face fewer disparate entities, and for
  better communication among the Tribes themselves (R1-F1). The Fifth National
  Climate Assessment names staffing and grant burdens as the binding constraint
  (R1-F3).
- **The relationship knowledge must move into Tribe-held records.** Grant-funded
  partnership webs vanish with their funding, and the contacts go with them
  (R1-F5). An event-sourced graph held by Tribes and intertribal bodies keeps the
  network when funders and staff leave.

And one fact from the owner's own framework reframes the whole project. The
Tiered Sovereign Data Framework defines its Network tier this way: "T1 embodies
reciprocity - the foundational Indigenous value that relationships are mutual and
ongoing, not transactional. Network membership itself implies shared
responsibilities." Connection as relatives is not a feature to add on top of the
system. It is the definition of the tier the code labels Network and has never
activated.

## 2. What works today

The core is a strong single-group, single-viewer reader. Everything below was
read from the code and confirmed by the baseline test run.

| Area | Status | Evidence |
|---|---|---|
| Op log, fold, HLC, idempotent durable append | Working and tested | 18 op kinds (`core/crates/cn-store/src/op.rs:103-173`); order-independent fold with quarantine to fixpoint (`fold.rs:139-247`); `DurableOpIndex` (`seam.rs:27-221`) |
| Permission-filtered projection | Working and tested | Circle lattice and tier ceilings (`core/crates/cn-perm/src/rules.rs:18-30`, `core/crates/cn-model/src/tier.rs:20-32`); soundness proptest (`core/crates/cn-perm/tests/blueprint.rs:813`) |
| Per-attribute visibility, tighten-only tiers | Working and tested | `core/crates/cn-model/src/attribute.rs:172-216`; owner-only `VisibilitySet` (`core/crates/cn-perm/src/authz.rs:138-149`) |
| Provenance envelope and custody chain | Working and tested | `core/crates/cn-model/src/provenance.rs:105-183` |
| Graph queries | Working and tested, with one routing defect | paths, neighborhood, degree, betweenness, eccentricity, Jaccard, search (`core/crates/cn-graph/src/query.rs`) |
| JSON and wasm boundary | Working and tested | `core/crates/cn-api/src/wire.rs:13-25`; `core/crates/cn-wasm/src/lib.rs` |
| 3D app, presenter mode, flat table, search, detail | Working | one group and one hard-coded viewer per load (`app/src/main.ts:166-180`) |
| Facilitator intake wizard | Working, logic tested | `app/src/ui/intake/`; Chromium only (directory picker) |
| Remote intake (form, relay, pull, apply) | Built and accepted, not deployed | D-089; deploy bar D-059.8 unmet |

The honest summary of the gaps:

- **Routing exists but nobody can reach it.** `client.queryPaths` and
  `client.queryNeighborhood` have no caller outside a test
  (`app/src/wasm/client.ts:89-103`). The R6 need-to-solution routing core is built
  and invisible.
- **The person at the center cannot see their own world.** There is no viewer
  switcher, and an unprivileged person on the ATNI fixture sees only their own
  entity, because people there hold no memberships (`SESSION_ROSTER.yaml:418-420`).
- **There is one graph at a time.** An entity has one group (`entity.rs:16`), an
  edge's endpoints must live in the same fold (`fold.rs:425`), and `project()`
  takes one state (`projection.rs:173`).
- **The Network circle has no members.** It is the fallback for any person who is
  not the owner, not trusted, and not a member (`viewer.rs:34`), and viewers are
  asserted by the caller (`wire.rs:31-33`, D-015). In practice Network means anyone
  who supplies a UUID.
- **Relationships have no consent.** Any member may link any two entities, with no
  endpoint consent (`authz.rs:34`, `:84-90`), and because edges have no owner,
  Trusted and Private edges are unreachable by everyone, including their creator
  (`projection.rs:266-268`).

## 3. What clearly remains

**The convention tail.** Nothing in the repo records how the A9 reveal went, and
the D-052 post-convention retrospective was never held; HANDOFF and the plans
stopped on 2026-09-14. The deploy bar is still unmet (keygen ceremony, D-023
sign-off of the whole form page, the D-089 real-browser smoke, a Pages workflow,
a guard against shipping the localhost relay origin, and placeholder ids in
`relay/wrangler.toml`). The S-E2 verdict is owed, the S-E2 branch exists only in
this clone (its worktree is gone), and on main the QR form built from template
0.1.2 cannot complete apply against the ATNI fixture, whose embedded template is
0.1.1; regenerating the fixture at the S-E2 merge resolves it. The promised
post-conference deletion needs a named owner (D-106, D-059.11).

**The deferred spine.** Phase 6 personal mode (R4), Phase 7 network readiness
(R5: `cn-sync` is a four-line placeholder), Phase 8 accessibility and governance
tooling, and Phase 9 reusability and sibling integration were all deferred to the
retrospective that did not happen. Smaller gaps: `cn ingest` and `cn snapshot` are
stubs; stories are carried by the core but dropped by the app's projection type;
the snapshot build ships a data-free shell by default.

**Defects found by the audit** (fixed this session unless noted): weighted routing
with a hop limit could miss a valid route (`query.rs:478`, `:506-510`); edge
creation never checked the edge's group id (`fold.rs:418-447`); edge submissions
could reveal whether a hidden entity exists (`core/crates/cn-api/src/submit.rs:88-93`);
the permission spec still carried a pre-errata comparison
(`docs/specs/permission-model.md:86`); no test proved that hidden entities leave
measures unchanged. Not fixed and recorded for later: loaded op logs fold without
authorization (`fold.rs:139-149`), so a federated log would be trusted wholesale;
one governance member can widen what a whole group shares (`authz.rs:429-438`);
trust grants do not go inert when the grantor leaves (ADR-002 A-B2).

**Planning drift.** PLAN_1.0 and TRACE keep all 64 task boxes unticked, including
finished ones, so they cannot serve as status. The phase numbering for personal
mode differs between CLAUDE.md and PLAN_1.0. D-106 describes the TSDF consent link
as built when it was not (built this session). The owner's 2026-10-19 cadence edits
are uncommitted in the working tree and have no DECISIONS entry; they are the
owner's to land.

## 4. The idea: relatives, not contacts

A contact is a fact one party holds about another. A relative is a relationship
both hold, with responsibilities that outlast any one task. The research makes
that difference concrete enough to build.

- **Two-sided.** A tie should exist only when both ends acknowledge it. Survey
  atlases publish one-sided claims (R7-F12). A no must be final, private, and
  carry no penalty: "if we're going to ask permission, we have to listen for the
  answer as well" (Kimmerer, R3-F5).
- **Responsible.** Kyle Whyte defines being in a relationship as holding
  "reciprocal (though not necessarily equal)" responsibilities (R3-F1). A request
  that is answered should leave a return owed: thanks, results, credit, notice.
- **Redundant.** Whyte names ecological redundancy, many knowledge holders and many
  chances to meet a responsibility, as a root of collective continuance (R3-F1).
  So the most valuable analysis is not "who is most central" but "what rests on one
  person", phrased as an invitation to share the load.
- **Introduced, then direct.** Brokers who keep people apart burn out; brokers who
  join people do not (R4-F7, R4-F8). A mutual contact raised new-tie formation by
  35 percent (R4-F10), and a 90-minute purpose-bound session raised grant
  co-application by 75 percent (R4-F9). An introducer should step out once the two
  people hold their own tie.
- **Carried by the person, kept by the seat.** A person keeps their relatives for
  life while affiliations are dated ties that institutions vouch for (the ORCID
  pattern, R8-F10). The working relationships of a role stay with the seat when its
  holder changes, which answers the turnover the cuts produced (R1-F7).
- **More than human.** Rivers, runs, foods, and places are relatives with their own
  caretakers. SIKU lets each user set ownership and stewardship per post (R2-F1);
  the InterTribal Buffalo Council moves buffalo, framed as sacred beings, across
  Nations through an intermediary (R2-F11); Bodin's work shows collaboration works
  better when actors on interconnected parts of an ecosystem are tied (R4-F1). The
  CTUIR First Foods approach is the Umatilla Tribes' own frame (R2-F4); it can
  inspire a template, but a Nation-specific frame ships only when that Nation
  authors it.
- **Paced by protocol and season.** The Tribal Climate Adaptation Menu puts
  relationship strategies first and warns that climate urgency clashes with the
  time relationships need (R3-F12); Whyte describes a relational tipping point
  passed before the ecological one (R3-F2). Trust should widen slowly, through
  repeated reciprocity, never through an instant request.
- **Consented twice.** Individual consent sits inside community governance as two
  keys, the person's and the governing body's (R8-F5). Each Nation's own protocols
  outrank CARE, which is the floor where none exist (R3-F10).
- **Seen is not used.** Seeing a record does not grant the right to reuse it
  (R2-F1), and a license and tier should follow derived products such as maps,
  summaries, and embeddings (R3-F8, R6-F12).

A caution the community-value judge raised and this document adopts: name the
source of every frame, and do not borrow another people's words as product names.
A hash-linked op feed should not be called a whakapapa chain in an ATNI tool.

## 5. Three consents: how graphs interlink

Interlinkage needs consent at three levels, and the owner's framework already
defines two of them.

| Level | What it decides | Today | Proposed |
|---|---|---|---|
| Person | Which of my values each audience receives; whether my records may be carried across a bridge | Per-attribute circles exist; no bridge consent; no way to see the result | The record mirror (built this session); bridge consent in ADR-007 |
| Relation | Whether a tie between two relatives exists, and who may see it | Any member can assert any tie; private ties are unreachable | Two-sided acknowledgment, final private refusal, ties visible to their two ends (ADR-008) |
| Nation or collective | What a group shares with whom, for what purpose, until when | Nothing; T2 data can never leave the group | TSDF T2 agreements and T1 reciprocal compacts as executable, expiring records with receipts (ADR-007) |

How ADR-007 maps to TSDF. TSDF's access checks are two questions: for T1, "Is
requester a member of the authorized Indigenous network with standing reciprocal
protocols?", and for T2, "Is there an active, documented agreement that explicitly
covers this data and this requester?" (`tier-decision.md`). The code keeps the
order of the tiers but flattens the mechanisms: T1 has a ceiling and no members,
and T2 has a ceiling and no agreement path. ADR-007 restores both, in phases.

- **Phase A, director work once the human accepts the ADR, and purely additive:**
  a named outside person in an active, ratified agreement receives in-scope values
  up to T2 as if a group member, when the owner's bridge consent (or, for unowned
  records that are not person records, the group) covers them. Never T3. Unowned
  person records, intake records, and relation-bearing attributes never cross on
  the group's act alone. The projection takes an injected time and a declared use
  (view, routing, analysis, export), so expiry is real and each use is admitted
  separately, and caches are keyed on viewer and use. Receipts the owner can read
  moved to a later slice (ADR-007 D9) after review found that receipt headers
  would show governance who consented and when.
- **Phase B, a human gate:** the Network circle comes to mean members of groups
  that are ratified parties to an active compact, TSDF T1 made literal; the
  reciprocity rule is the compact's data.
- **Phase C, human gates:** the same relative in two graphs, signed and verified
  logs before fold, and a transport stay out of scope until the identity and
  network choices are made.

Before any of this reaches real data across graphs, the red-team judge's
preconditions hold: logs must be verified before they fold, the viewer must be
bound to a key instead of asserted, circles above Group should be sealed by
encryption so that a device holding a raw log cannot read above its own circle,
and a Nation needs a single collective exit that severs all its bridges at once.

**What relaxing the guards revealed.** TSDF is more open than the code. It allows
network-wide sharing under reciprocal protocols and negotiated sharing with named
partners, and the code can express neither. The way to share more boldly is to
implement TSDF's own sharing mechanisms, with expiry, purpose, and receipts, not to
remove checks. The checks that would make federation safe do not exist yet, so they
are the work.

## 6. What a broadly shared bridge cluster could generate

When many entities choose broad sharing and consent to serve as bridges, the
cluster can produce things no single graph can. Each item names who it serves, the
evidence for the need, the sharing posture it requires, and the harm to guard
against. Items marked (built) have a first slice in this session.

### For a Nation's own staff

1. **Who can help with this (built).** Type a need; see the relatives who offer
   it and the chain of relatives that reaches them, with an introducer named only
   if they said they are willing. This is the liaison's job, done with the Nation's
   own records (R1-F2, R1-F8; human brokers in the loop work best, R2-F2). Posture:
   the viewer's own projection, one group. Harm: the same few people get every
   request, so capacity and rest season are respected and resting people sort last.
2. **One Telling.** Record needs and capacities once, then give each county, state
   office, FEMA region, or funder a signed, scoped, expiring slice instead of
   answering the same questionnaire again (R1-F1; selective disclosure is now a
   standard, RFC 9901, R5-F9). Posture: agreement-scoped (ADR-007 Phase A). Harm:
   purpose drift once disclosed, so the slice carries use terms and a receipt.
3. **Continuance check before each hazard season (built, in part).** Show which
   lifelines rest on one person, one agency contact, or one bridge, and which ties
   have gone untended since the cuts (R3-F1, R1-F7). Posture: a person sees what
   rests on them; the full picture is governance-only and never exported, because a
   list of a Nation's irreplaceable people is a poaching and targeting list.
4. **Dormant mutual-aid compacts.** Promises agreed in calm times between Nations,
   counties, and clinics that wake for a declared smoke, flood, heat, or fire event
   and sleep when it closes, then ask for the return gift. The Lummi Nation boated
   clinic staff in when floods cut every road (R1-F12); Washington's Tribal and
   local health mutual aid agreement already exists on paper (R2-F12); the
   Navajo and Hopi relief effort showed how fast a relationship graph must activate
   (R2-F10). Posture: sealed until a quorum declares the event; auto-expiring.
5. **Check-in circles for elders and the isolated.** During a declared heat or
   smoke event, each person who wants it has two or three relatives committed to
   check on them; the emergency manager sees coverage gaps as counts, never names
   outward. The need is plausible and urgent but not yet documented in the register
   (R1 open question on the 2021 heat dome); verify it with Tribal emergency
   managers before building. Posture: within one Nation, T2 or tighter.
6. **A printed relatives sheet.** A dated, expiring, viewer-scoped page of who to
   call for what, for offices with no connectivity, elders who do not use apps, and
   floods when the power is out (R1-F12). Cheap, and likely the most used.

### For networks of practice and their weavers

7. **Table setter for convenings.** Before a gathering, propose small tables where
   what one person seeks meets what another offers across Tribes, printed with the
   reason for each seat and sent nowhere automatically. The convention form already
   collects seeking and offering, and nothing reads them yet (R4-F9).
8. **Walked this path.** Connect a Tribe starting an adaptation plan, a relocation,
   a co-stewardship agreement, or a frozen grant with relatives one or two steps
   ahead who have offered their plan or template (Tribal Climate Camp's culture of
   gifting plans, R2-F8; agreement capacity limits, R1-F4).
9. **Introductions you could ask for (built, read-only).** Two-hop relatives you
   are not yet tied to, through a mutual relative who is willing to introduce, with
   the shared ground stated; the introducer steps out once you are tied (R4-F8,
   R4-F10).
10. **Counterpart circles.** Seasonal circles of five to seven people who carry the
    same responsibility in different Nations, convened by a weaver who introduces
    them and steps out.
11. **Next stewards.** Learning ties between Tribal youth or early-career staff and
    mentors across Nations, carried as acknowledged ties, to replace the cohort
    structure lost with the Tribal Stewards program (R1-F5).
12. **Network memory and ripple stories.** Fold the log as of two dates to show what
    the network gained, what went quiet, and what an introduction led to, as ripple
    stories with measures each Nation defines (Ripple Effects Mapping, R4-F11;
    network evaluation pillars, R4-F6; the AIHEC framework puts evaluation in the
    Tribe's hands, R3-F11). The op log can finally answer the open research question
    of whether brokered ties last (R4 open questions).

### For many Nations together

13. **Coalition in a day.** From organizations that consented to serve as bridges,
    assemble a proposal-ready partner roster for one funding call, asking each
    partner once, for that purpose, with a receipt. NOAA's Climate Resilience
    Regional Challenge funded a consortium of 31 federally recognized Tribes through
    the Bristol Bay Native Association (R7-F11).
14. **Shared-limbo finder.** When a grant is paused or terminated, find the other
    Nations in the same limbo without anyone revealing their portfolio, using private
    set intersection over program identifiers (R1-F10, R5-F12), then open a bridge
    only by consent.
15. **Opportunity routing.** When the TCR Policy Scanner finds a funding call, notify
    only the Tribes who opted in and whose stated needs match, through each Nation's
    designated contact. Broadcasting does not route (the Pacific Northwest Tribal
    Climate Change Network lesson, R1 exemplars).
16. **The river is the bridge.** Let a river, a salmon run, a buffalo herd, or a
    First Food be the shared node that joins many Nations' graphs. Each Nation keeps
    its own relationship to that relative at home, and the federation shows only
    who else is related to it and where caretaking has gaps. An observation on the
    relative, a fish kill or a low-flow alarm, reaches everyone who cares for it
    (LEO Network routes each observation to experts, R2-F2; co-managers already
    publish a merged fish-distribution reference, R2-F7).
17. **Ask-once commons brief.** One consented picture of what each part of the
    network works on, what is unmet, and what capacity is shareable, exact for its
    owners and thresholded for outsiders. Small-number suppression has been used to
    deny Tribes data about their own members (R7-F5), and census privacy noise erased
    detail for small Tribal areas (R7-F4), so the threshold depends on who is
    looking, and who sets it is a sovereignty decision.
18. **Regional readiness atlas.** Counts, never names or locations, of what each
    fire shed, airshed, and watershed can lend and lacks, drawing on the sibling
    hazard and geospatial tools.
19. **Relational accounts.** From Nations that consent to publish their
    institution-facing ties, an aggregate account of each agency, university, funder,
    and NGO: what it asked for, what it returned, and where the relationship runs one
    way. Extraction by outside researchers is the commonest documented harm (R7-F3).
    High value, high risk: structured facts about commitments kept, never free-text
    reviews.
20. **Common ground.** Rank needs and proposals by how many different Nations and
    institutions stand behind them, not by how often one office repeats them
    (bridging-based ranking, R6-F10), and use the same signal to allocate pooled
    staff time or a small regional fund (connection-oriented cluster matching, R6
    exemplars). No Tribal deployment of either was found; it would be a first.

The judges' ranking put the person-centered and consent work first (the mirror,
two-sided relations, the routing panel, the kinship compact) and held the
cross-graph aggregates, private set intersection, and AI agents until identity and
network gates are settled. This document agrees, and the sequence in section 9
follows it.

## 7. What is newly feasible, and where this could lead

Status as of November 2026, from the R5 and R6 research (Jev-checked where marked
in the register).

| Capability | Status | What it enables here |
|---|---|---|
| Selective disclosure: SD-JWT (RFC 9901, November 2025), OpenID4VP 1.0 Final (July 2025), VC 2.0 Recommendation (May 2025) | Standards, production wallets coming | One Telling slices; Tribe-vouched credentials a person carries between institutions (identity choice is a human gate) |
| Capability tokens: Biscuit 6.0 (July 2025, Apache-2.0) | Mature Rust crate | A portable, offline-verifiable encoding of agreements for ADR-007 Phase C |
| Local-first access control: Keyhive (pre-alpha, "DO NOT use"), Meadowcap (final spec, November 2025) | Designs to learn from | The shape of delegated, time-bounded read capabilities |
| Transport: Iroh 1.0 (June 2026), compiles to WASM | Production-ready | A candidate `cn-sync` adapter (network choice is a human gate) |
| Private set intersection (voprf, OpenMined PSI), differential privacy (OpenDP, work in progress), anonymous membership (Semaphore) | Usable but narrow | Discover shared relatives without exposing rosters; thresholded aggregates |
| On-device AI: WebGPU in all four major browsers (November 2025); WebLLM at up to 80 percent of native speed; EmbeddingGemma under 200MB | Practical on a 32GB laptop | Semantic matching across community vocabularies and plain-language path explanations with nothing leaving the machine |
| Bridging-based ranking and cluster-matched funding | Open algorithms | Priorities and pooled funds that reward agreement across Nations |

Two cautions travel with the AI rows. First, the graph already exists, so the
expensive step of having a language model extract a graph from text is unnecessary;
the core does the retrieval and a small model only explains (R6-F6, R6-F8). Second,
an assistant that sees private data and untrusted input and can reach the outside
is the "toxic combination" (R6-F9): any assistant reads only one viewer's
projection, treats intake free text as untrusted, and has no outward channel. TSDF
already requires machine-readable AI and ML restrictions per tier (T3 prohibited
for training, inference, and embedding; T1 needs network approval), and those
fields must exist before any AI feature reads data.

Where Community Navigator could lead: no system found in this research combines
these. It could be the first Indigenous-governed, local-first relational graph in
which consent is structural at three levels; bridging between sovereign graphs is
negotiated as executable agreements that implement TSDF T1 and T2; discovery across
graphs can happen without exposing rosters; collective voice is released exactly to
its owners and thresholded to outsiders; an AI helper is a declared relative under
each community's protocol rather than an invisible service (the Indigenous Protocol
and AI position paper, R3-F9); and relationship memory survives funding cuts and
turnover because it lives in Tribe-held, event-sourced logs.

## 8. What this session built

Twenty commits on main after `1abedc2`, all fully synthetic, none pushed. Each code
unit was checked by an independent claim-verifier; every verdict was pass, with
advisories only. The core units were built in two worktrees (`relatives/perm`,
`relatives/graph`) and cherry-picked onto main without conflicts; both branches are
fully reconciled (`git cherry` shows every commit as already on main).

| Unit | What it does | Commits |
|---|---|---|
| Research register | 8 topics, 40 load-bearing claims Jev-checked, 4 corrected | `f07e1bc` |
| Relatives-demo fixture | 57 synthetic entities where every person is a member and owns their record; planted liaison, bridge, introductions, sole holder, resting person, seat change | `9777031` |
| TSDF consent link | "Tiered Sovereign Data Framework" is a link on both consent paths; the digest text is unchanged | `ef3ee77`, `6db3c24` |
| Defect fixes | Foreign group id edges quarantine; edge submits no longer reveal hidden entities; permission spec matches code | `49a51cb`, `23aacec`, `3ce9823` |
| Record mirror | What each audience (public, network, group, trusted) receives of a person's own record, from the projection's own rules | `c75f904` |
| Analysis scope | Governance sees all continuance and introduction results; everyone else sees only their own records | `f0e76a6` |
| Graph split | `query.rs` split into index, measures, and query modules (I5), public API unchanged | `0436d19` |
| Routing fix | Weighted paths honor `max_hops` without dropping valid routes | `f68b38e` |
| Invariance tests | Measures, paths, neighborhoods, search, continuance, and introductions are identical with or without entities the viewer cannot see | `4aa654a` |
| Continuance | Articulation points and bridge ties over the viewer's projection | `5be76c8` |
| Introductions | Two-hop suggestions through mutual relatives, introducer named only when willing | `0048e77` |
| cn-api and wasm | `graph_continuance`, `suggest_introductions`, `record_mirror` through cn-api, cn-wasm, and the worker client | `7c45afa` |
| App state | Routing, neighborhood, continuance, introductions, and mirror requests in the state machine | `8efb181` |
| Relatives panel | Who can help, introductions you could ask for, what rests on you, see your record as | `8f8e203` |
| Ring layout | "My relatives" ring layout centered on the viewer, beside "Whole network" | `130e20d` |
| ADR-007 (Proposed) | Network circle and bridging, TSDF T1 compacts and T2 agreements, phased | `118f1be` |
| ADR-008 (Proposed) | Two-sided relations: acknowledgment, final private refusal, grandfathering | `b151878` |

Both ADRs went through three adversarial review lenses (permission soundness;
TSDF fidelity, sovereignty, and harm; fit to the codebase; each lens judged both
"revise before acceptance") and a Jev fidelity check of their TSDF citations. Every blocking finding
was applied in the committed text, and all but two advisory sub-parts (declined with
reasons); each ADR ends with its review log. Neither is accepted; acceptance is the
human's.

**How to try it.** From `app/`, run `npm run dev` and open the dev URL with
`?group=relatives-demo`. The viewer is the ordinary member `...127699`, the seeker in
`fixtures/README.md` structure (b). The Relatives panel sits between the detail panel
and the flat list: search "eelgrass" under Who can help to see the route through the
willing introducer, and press My relatives for the ring layout.

**Integration.** `pwsh scripts/check-all.ps1` at `130e20d`: 12 of 12 members passed
(Rust 317 tests at the core merge, app suite green, snapshot 0.85MB of 5MB, PII scan
clean on 401 files). Outside check-all, the form suite passed 81 of 81 and the relay
suite 47 of 47. The run used the owner's uncommitted edits to `scripts/check-all.ps1`,
`scripts/pii-scan.ps1`, and the hook scripts, so it shows HEAD plus those edits. No
browser, real-GPU, or real-device check was done.

**Follow-ups and verifier advisories that matter.**

- **EdgeCreate rule change.** Members and facilitators are now denied `target_missing`
  when linking to an entity outside their projection, including archived or retracted
  entities they could see before. This is what makes a hidden endpoint look exactly
  like an absent one. Governance keeps the old quarantine behavior. The
  `s-e2-intake-edges` branch and any app flow that links to such entities must be
  checked against it at integration.
- **StoryCreate oracle still open.** Story steps naming a hidden entity quarantine
  `MissingTarget`, so a member can still probe hidden ids that way. Not fixed.
- **UI copy is draft.** All panel and toggle wording is developer language and needs
  ATNI vocabulary and community-language review (D-051, D-023) before anyone outside
  the team sees it. The mirror shows raw circle names.
- **Human-only checks owed (I9).** No one has seen the panel or ring layout in a
  browser: keyboard order, 375px layout, screen-reader announcements, reduced motion,
  and the TSDF link on both consent paths.
- **Smaller items.** The mirror's audience-to-circle mapping is a second copy of the
  `admissible_circle` rule, guarded only by a four-viewer test; one failing path call
  fails the whole routing request; the viewer's center is the first owned entity
  (shortcut comment); choosing an already-focused result does not light its path; the
  camera is not refit on a layout switch; `willing_to_introduce` matches
  case-sensitively; the ring order uses `localeCompare`; `fixtures/README.md` still
  says fixture vocabulary stays in the research and fisheries domains (D-044.4), which
  this fixture and the ATNI fixture both stretch; `cn-api/src/lib.rs` (664 lines),
  `app/src/viz/index.ts` (725), and `cn-store/src/fold.rs` (1279) exceed I5.
- **Stale worktree records.** The `s-e2-intake-edges` worktree is prunable (its folder
  is gone); the branch itself is intact at `6c92eed`.

## 9. Recommended sequence

1. **This session's slice:** defect fixes, the record mirror, routing reachable
   from the app, continuance and introduction suggestions, and a synthetic fixture
   where every person is a member (section 8).
2. **Two-sided relations (ADR-008)** before the S-E2 branch or any form answer
   creates person-to-person edges. In the same slice: an owner claim so that a
   remote respondent's record can belong to them (approvals land unowned today,
   ADR-005 D7, so none of the person-centered features reach them), and a viewer
   bound to a key instead of asserted.
3. **ADR-007 Phase A:** the injected time and declared use, agreements naming
   outside persons, and bridge consent; receipts follow as their own slice (D9).
4. **Verified logs and collective consent:** signed, hash-linked ops verified before
   fold, and a k-of-n governance seal for widening and ratification.
5. **Gated by the human:** Network compacts (Phase B), the same relative across
   graphs, transport, private set intersection, and thresholded aggregates.
6. **In parallel, any time:** a template studio so a Nation's own staff can author
   its kinds, relatives, labels, own-language names, and default tiers without
   editing JSON (every "their own frame" idea depends on it); the convention tail.

## 10. Safeguards that stay on even with relaxed guards

- Tier ceilings stay structural; T3 never leaves; widening needs a governance act,
  never a consent flag alone.
- Edge lists are never exported in bulk, even when the nodes are public. A watch
  list of who is tied to whom is exactly what was built against water protectors at
  Standing Rock (R7-F1), and it was reused against other movements (R7-F2).
- Results that name keystone people, elders, harvest sites, or alliances are
  governance-only and unexportable; a person sees what rests on them.
- A refusal is visible only to the one who refused; "not confirmed" looks the same
  as unanswered.
- Every grant names its purpose and expires; every crossing leaves a receipt.
- The graph is not an enrollment record: membership in a graph is never citizenship,
  and templates should be unable to carry lineage or blood-quantum attestations.
- Real data enters only after the recorded ATNI Climate committee checkpoint, and
  synthetic fixtures stay synthetic.

## 11. Questions for the owner, one line each

1. How did the A9 reveal land, and should the D-052 retrospective be recorded now?
2. Does Phase A of ADR-007 (agreements naming outside persons, expiring, with
   receipts) go ahead next, or does ADR-008 (two-sided relations) go first as
   recommended?
3. For Network compacts: is network membership alone enough, or must a party share
   to see (contribution-gated reciprocity)?
4. Who ratifies for a group: one governance member as today, or a quorum?
5. Should facilitators (network weavers) see the full continuance picture, or only
   governance?
6. May the S-E2 branch be backed up off this machine (it exists only in this clone)?
7. Which Nation, if any, would co-author a more-than-human relatives template?

## References

- Evidence register: `docs/research/relatives-discovery-2026-11-11.md`
- Proposals: `docs/adr/ADR-007-network-circle-and-bridging.md`,
  `docs/adr/ADR-008-two-sided-relations.md`
- Prior research this builds on: `docs/research/graph-networks-report-2026-07-06.md`
- The owner's framework: the Tiered Sovereign Data Framework
  (github.com/atniclimate/TieredSovereignDataFramework)
- Decision record: DECISIONS.md D-107
