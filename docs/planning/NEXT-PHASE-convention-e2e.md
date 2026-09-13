# Next phase: convention-ready end-to-end intake, edges, and presenter controls

Status: PLANNING ONLY. Nothing in this document is built. Written 2026-09-12,
after the reveal chain (R0/R2/R4a/R4b/R7) landed and check-all reached 12/12 at
HEAD `10dbc78`. This phase turns the human's request - "QR scan to a simple
ATNI-branded form, collected and assigned proper edges/nodes/clusters, into the
graph; simple presenter commands (c/o/t/s) and a few on-screen buttons; prepare
for tests of the system" - into sessions, against the hard gates that stand
unchanged (CLAUDE.md, D-059.8, D-090c).

Companions: `SESSION_ROSTER.yaml` (new sessions `S-E1..S-E5`), `DECISIONS.md`
(D-089/D-090/D-097/D-098 read for this plan), `docs/research/discovery-2026-09-12.md`
(candidate decisions D-090c..D-096c, still open).

## 1. Goal and the two outcomes

Every session below builds and rehearses on SYNTHETIC data (`fixtures/templates/
atni-convention.template.json`, `fixtures/groups/atni-convention.ops.jsonl`).
Nothing here decides whether the convention itself goes live remotely - that
is D-090c, a human decision, not an engineering one. Both outcomes below are
served by the SAME engineering work; only the deploy step differs:

- **Synthetic demo path (default, always available):** the reveal runs
  in-app facilitator entry (`docs/runbooks/live-entry.md`) or the local,
  non-deployed remote-intake rehearsal (`docs/runbooks/e2e-remote-intake.md`)
  against synthetic data, on the presenter's own laptop. No git remote push
  of secrets, no Pages/Workers deploy, no real PII. This is fully rehearsed
  by session E4 below regardless of any human gate.
- **Live-at-convention path (conditional on the human):** the QR code
  resolves to the real deployed form and real Workers relay, real attendees
  submit real data. This requires every row in the table below to be TRUE.

### Go-live checklist

| Item | Owner | Status | Receipt | Blocking session |
|---|---|---|---|---|
| ADR-005 accepted | eng | DONE | `docs/adr/ADR-005-remote-intake.md:1` (D-068, 8 rounds) | - |
| Intake pipeline works end to end (synthetic) | eng | DONE | D-089; `docs/runbooks/e2e-remote-intake.md` | - |
| Real-browser Playwright smoke of the built form | eng | OWED | no test exists under `form/` or `scripts/` (confirmed this session) | E4 |
| GitHub Pages deploy workflow | eng | OWED | `.github/` absent (confirmed this session, `ls -la .github` style check via Glob) | E5 (extends S-R1) |
| D8 post-deploy hash verification step | eng | OWED | `docs/adr/ADR-005-remote-intake.md:1030` D8 spec exists; no CI step | E5 (extends S-R1) |
| R4-5 build guard against shipping localhost relay origin | eng | OWED | D-089 "Deferred / recorded LIMITATIONS", R4-5 | S-R1 (unchanged) |
| Form builds against the ATNI template, not research-network | eng | OWED | `form/vite.config.ts:47-54` defaults to `research-network.template.json`; overridable via `CN_FORM_TEMPLATE_PATH` (`:60`) but no script/CI wires the ATNI path by default | E1 |
| ATNI dark-mode brand theme (colors, fonts) | eng + human | OWED (needs human assets) | template already carries `theme.mode: "default-dark"` and 3 role colors (`fixtures/templates/atni-convention.template.json:53-60`); no ATNI-specific palette or font files exist anywhere in the repo | E1 (token slot) + human (assets) |
| Approved submissions produce edges (member_of/affiliated_with), not just a bare node | eng | OWED (new work) | see research question (a) below | E2 (adversarial round required) |
| Keygen ceremony EXECUTED | human | NOT DONE | off-repo by design; `cn intake selftest --dry` is the rehearsal | S-R1 rehearses; human executes |
| D-023 sign-off on consent/form text | human | NOT DONE | `docs/design/intake-consent-text-draft-2026-07-24.md:1` still DRAFT | human |
| GitHub Settings Pages source = "GitHub Actions" | human | NOT DONE | UI click, not a commit | human, after E5 |
| Workers Paid tier decision (D-092c, spend) | human | NOT DONE (candidate D-092c) | `docs/research/discovery-2026-09-12.md:197` | human |
| Recorded ATNI Climate committee approval of network activity | human | NOT DONE | CLAUDE.md real-data gate, `HANDOFF.md` gate authority | human |

Until every row reads DONE, the convention runs on the synthetic demo path.
Nothing in this phase moves that default.

## 2. Research questions (answered from the code, with receipts)

### (a) Does an approved intake record create edges, or only an EntityCreate?

**Only an EntityCreate. Confirmed, not assumed.** `plan_approval` in
`core/crates/cn-ingest/src/approval.rs:602-624` builds exactly ONE
`Operation` with `kind: OpKind::EntityCreate { entity }` per approved
submission; there is no second op and no `OpKind::EdgeCreate` anywhere in the
function. The blueprint that specified this is explicit about why:
`docs/blueprints/intake-pipeline.md` (`plan_approval` bullet, section 2)
says "ONE EntityCreate carrying the FULLY POPULATED unowned entity per
submission ... EdgeCreates deferred to merge-by-hand." So this is a known,
recorded gap, not an oversight - but it means "assigned proper edges/nodes/
clusters" from the human's ask is UNBUILT today.

It goes one level deeper than the durable-owner gap: the ATNI template
itself gives a submitter no way to assert committee or organization
membership. `fixtures/templates/atni-convention.template.json:16-25` lists
the person kind's attributes - `display_name, tribe, role,
areas_of_interest, specialties, events_of_interest, contact_email,
contact_preference` - none of which references a committee or organization
id, and the schema's attribute type vocabulary
(`schemas/group-template.schema.json`) has no "reference to another entity"
type (only text/number/enum/tags/date/geo/link/media). `form/src/model.ts`
renders whatever kinds and attributes the template defines
(`formModel()`, `:73-80`), so today the form CANNOT collect "which
committees are you on" even as a UI matter, independent of what
`approval.rs` does with the answer.

Building this needs two coordinated pieces:
1. A template addition giving the person kind a way to name committees/orgs
   (simplest: a `tags`-typed attribute, e.g. `committee_memberships`, whose
   values are constrained to the 15 committee display names - or a new
   attribute type if a real reference type is wanted; the tags approach is
   additive and needs no new schema machinery, just a version bump, per the
   D-090 precedent of PATCH-bumping `schema_version` rather than a real
   semver minor).
2. `plan_approval` extended to also emit `EdgeCreate` ops
   (`member_of`/`affiliated_with`) resolving the named committees/orgs to
   their fixed-instance entity ids in the target group, when such a field
   is present and validates.

This touches the durable owner (`approval.rs`, `plan_approval`, the seam)
directly - it is exactly the surface D-056.1/D-056.2 named as
permission-adjacent. **Session E2 below carries this and its mandatory
adversarial round, following the same pattern already used for S-R3's term
normalization** (which also touches `approval.rs`).

### (b) Can `form/` build against `atni-convention.template.json` today?

**Mechanically yes; usefully, only partway.** `form/vite.config.ts:47-54`
defaults `DEFAULT_TEMPLATE_PATH` to `research-network.template.json`, but
line 60 reads `process.env["CN_FORM_TEMPLATE_PATH"] ?? DEFAULT_TEMPLATE_PATH`
- so `CN_FORM_TEMPLATE_PATH=fixtures/templates/atni-convention.template.json
npm run build` (from `form/`) builds today with zero code changes.
`form/src/model.ts`'s `formModel()` iterates `template.kinds` generically
(`:73-80`) and would render a kind picker across person/committee/
organization plus every attribute widget for whichever kind is selected -
so the ATNI person fields (tribe, role, interests, contact) DO render.

What does NOT render, and cannot until (a) above is built: any way to
select which committees or organizations the submitter belongs to, because
no such field exists in the template. So the form is buildable against the
ATNI template now, but a submission through it today produces a
disconnected person node with no committee/org ties - the same limit as
(a), from the other end of the pipeline.

Not verified this session: whether `scripts/build-form.ps1` (the
human-facing build script) exposes a way to set `CN_FORM_TEMPLATE_PATH`,
or whether it hardcodes the research-network default. Session E1 checks
this and adds an explicit `-Template atni-convention` (or similar) flag if
absent.

### (c) How does a newly applied entity reach the running app - is a live refresh feasible under "the graph never listens"?

Today: manually, by a facilitator action, never automatically.
`docs/runbooks/live-entry.md` step 4 and `CONTROLS.md`'s "Live entry loop"
step 4 both say "Reload the group in the app to see the applied entries" -
a full reload of the load path, after running `cn intake apply` from a
terminal. `docs/blueprints/intake-pipeline.md` section 5 (facilitator
wizard) already anticipates something smoother: "the dashboard surfaces
staged decisions with the `cn intake apply` instruction and, after an
apply, prompts a group reload" - but this is described, not built; no
in-app "reload" affordance exists yet (not found in `app/src/ui/intake/`
by this session's reading).

ADR-005 D1 is explicit and binding: "The graph and core never listen on
any network... no component of this system accepts an inbound connection."
A facilitator-triggered pull is NOT a violation of that - it is the same
shape as every existing load: the app already does one outbound fetch of
group ops at boot (`app/src/main.ts` `loadDevDemo`/group loader, via the
wasm client, no socket). Re-running that SAME fetch on a facilitator click
(rather than only at page load) adds no new network surface and no
listening behavior; it is a manual pull, same as pressing F5, just from an
in-app button instead of the browser chrome.

**Recommendation for E2/E3 scope: a "Reload group" button in the
facilitator wizard dashboard** that re-invokes the existing group-load path
after a `cn intake apply` run is reported complete by the facilitator (the
facilitator still runs `cn intake apply` from a terminal - this phase does
not attempt to invoke a native CLI command from the browser, which would
be a much larger and riskier change to the app/native boundary). This
closes the "then reload" step's friction without adding a listener,
polling loop, or file-system watch of any kind. A polling or
file-system-watch variant is explicitly OUT of scope: it would make the
browser tab something that "listens" for change in spirit even if not by
socket, and ADR-005's commitment is treated as binding by this plan, not
reinterpreted.

### (d) What data would drive `t` (Tribes) and `s` (State) presenter beats?

**Confirmed: `tribe` is a free-text person ATTRIBUTE
(`fixtures/templates/atni-convention.template.json:18`,
`"id": "tribe", "type": "text"`); there is no `state` attribute or kind
anywhere in the template or the fixture generator
(`app/scripts/generate-atni-ops.mjs`).** This is a genuine gap between the
human's request and what the data model supports today. Options, laid out
as a candidate decision (not chosen here):

- **Option 1 - `t` clusters by the existing free-text `tribe` value,
  exact-string grouping.** No schema change. Risk: free text means
  near-duplicate spellings ("Cedar Hollow Nation" vs "Cedar Hollow") split
  what should be one cluster; acceptable on the synthetic fixture (values
  are drawn from a fixed 12-item word bank, `generate-atni-ops.mjs:76-80`,
  so they exact-match by construction) but would need the alias-table
  matching design (D-093c, session E-series does not build this) before
  real attendee data could rely on it.
- **Option 2 - convert `tribe` to an `enum` or fixed `tags` vocabulary.**
  Removes the near-duplicate risk but is a schema change (additive,
  version bump) and constrains what a real attendee could type; ATNI
  Climate would need to supply the fixed tribe list, which is exactly the
  kind of "vocabulary authored by ATNI after the system is stable" the
  standing ruling D-051 defers.
- **`s` (State) has NO underlying data at all.** ATNI committees are a
  tribal/national structure, not a US-state structure, so it is not
  obvious "state" maps onto this domain the way "committee" or "tribe"
  does. Options: (i) add a new optional `state` attribute to the person
  kind (schema bump, needs the human to say what "state" should mean here
  - state of residence? state of the org's HQ?); (ii) drop `s` from the
  reveal's key bindings and free it for something the data already
  supports well, e.g. `s` for "single-tie" (the edge-of-network measure
  that already exists per `docs/research/discovery-2026-09-12.md` Track B
  and `CONTROLS.md`'s `single_tie` beat measure); (iii) leave `s` bound but
  inert with an on-screen "coming soon" affordance until the human decides.

**This plan does not choose an option.** Session E3 implements `c`
(committees) and `o` (organizations) as straightforward kind-filter beats -
the data for both exists today with zero schema change, identical to the
`members`/`committees`/`organizations` beats already in
`app/public/beats.atni.json`. `t` ships as Option 1 (exact-match tribe
clustering) since it requires no schema change and the synthetic fixture's
tribe values already exact-match by construction. `s` ships only as
Option (ii) above - repurposed to `single_tie` - unless the human picks
Option (i) or (iii) before E3 starts, in which case E3's blueprint is
amended before coding begins. Candidate decision to record: **D-099c -
presenter key `s` scope**, options as above, decision needed before E3.

**Resolved 2026-09-12 by the human: `s` = state of residence (D-100).** Option (i). S-E3
adds an optional `state_of_residence` enum to the person kind (US states plus DC,
"Outside the United States", "Prefer not to say"; group visibility; schema PATCH
bump), synthetic values in the fixture generator, and an aggregate-only `s` beat (no
names; states under 3 members shown as "fewer than 3"). The fallback paragraph above is
superseded.

## 3. Phase sessions

Execution order: E1 -> E2 -> E3 -> E4, with E5 running whenever the human
clears (or moves toward clearing) the deploy bar - it does not block E1-E4
and E1-E4 do not block it. All sessions build and test on synthetic data
only; none crosses D-059.8.

### S-E1 - ATNI-template form build + dark-mode theme-token slot

Make `form/` build against `atni-convention.template.json` by an explicit,
documented switch (not only an undocumented env var), and give the app and
form a theme-token slot for an ATNI palette and fonts without inventing
brand assets. Reads: `form/vite.config.ts` (whole file, the
`CN_FORM_TEMPLATE_PATH`/`CN_FORM_*` mechanism), `scripts/build-form.ps1`
(whether it exposes a template switch), `app/src/theme/` (the existing
OKLCH/CVD token pipeline this session plugs an ATNI palette INTO, never
bypasses), `fixtures/templates/atni-convention.template.json` (`theme`
block, already `default-dark` with 3 role colors). Deliverable: a
`-Template atni-convention` (or equivalent) build-script flag; a documented
token slot (e.g. `theme/palettes/atni.ts` or a template `theme.roles`
extension) that the human's real ATNI colors/fonts drop into later with NO
further code change; placeholder colors/fonts used until the human
supplies real ones (never invented or scraped ATNI assets - CLAUDE.md
prime directive and this plan's own instruction). Gate:
`pwsh scripts/check-all.ps1` plus `npm run build` in `form/` against the
ATNI template plus a manual visual check that dark mode renders. Does not
touch `approval.rs` or any durable-owner code; not permission-adjacent.

### S-E2 - Intake-to-edges: committee/org membership becomes graph edges

Builds research question (a)'s fix: a `tags`-typed `committee_memberships`
(and optionally `organization_affiliations`) attribute on the ATNI
template's person kind (additive, schema PATCH bump per the D-090
precedent), rendered by the existing form/wizard field widgets with no new
widget type (`tags` already exists); `plan_approval` extended to resolve
named committees/orgs against the group's FIXED committee/org instances
(the 15 committees and N organizations are template-fixed, so resolution
is a lookup by display name, not fuzzy matching - D-093c's alias/fuzzy
work is explicitly NOT pulled into this session) and emit
`member_of`/`affiliated_with` `EdgeCreate` ops alongside the existing
`EntityCreate`, inside the SAME approval batch (so the durable seam's
atomicity guarantee - all ops in a plan succeed or fail together - covers
the new edges too, no separate write). **THIS SESSION TOUCHES THE DURABLE
OWNER (`approval.rs`) AND IS PERMISSION-ADJACENT per D-056.1/D-056.2 - it
gets the mandatory adversarial round, same as S-R3, before its commits are
accepted.** Reads: `core/crates/cn-ingest/src/approval.rs` (whole file,
already read this planning session - the `plan_approval` function and its
batch-digest/pre-link discipline this session must preserve exactly),
`fixtures/templates/atni-convention.template.json` (whole file, the
committee/org fixed-instance ids the resolution logic looks up against),
`docs/blueprints/intake-pipeline.md` section 2 (`plan_approval` bullet,
the "EdgeCreates deferred to merge-by-hand" line this session closes),
`schemas/group-template.schema.json` (additive-field policy),
`app/src/ui/intake/panel.ts` (the review view this session's new field
must render read-only alongside existing fields, no redesign needed).
Subagents: three narrow reviewers mirroring S-R3's pattern (schema/field
reviewer, ingest/edge-resolution reviewer, durable-owner/review-view
reviewer), each with a refute mandate, findings verified on disk before
disposition. Gate: `pwsh scripts/check-all.ps1` 12/12 AND the adversarial
round's verdict recorded in DECISIONS.md BY THE HUMAN (not by this
session). Depends on: none (can run before or after E1; does not touch
`form/` build config).

### S-E3 - Presenter controls: c/o/t/s keys + on-screen control rail

Adds keyboard shortcuts and a small always-visible button rail to presenter
mode, on top of the existing beat-advance machinery
(`app/src/viz/index.ts` `handlePresenterKeydown`, already
Space/ArrowRight/ArrowLeft/Home/F/Escape) - this session ADDS cases to that
same function and a matching rail of buttons, it does not replace the
beat-index model. `c` jumps to (or toggles) the `committees` kind-filter
beat, `o` to `organizations`, `t` to a new tribe-clustering beat (Option 1
from research question (d): exact-match on the `tribe` attribute, computed
either as a new `cn-graph`/`cn-api` grouping call following the
`graph_measures` pattern R4a already established, or as a client-side
group-by over the projection's already-fetched entities if a core call is
not warranted for a same-fixture exact-match - implementer's call, recorded
in the session's own outcome). `s` ships per whatever D-099c resolves to
before this session starts (default: repurposed to the existing
`single_tie` measure beat if the human has not decided by then). The
on-screen rail: a small fixed strip of labeled buttons (Committees / Orgs /
Tribes / [S-key label] / Members / Fit) mirroring the keys, hidden along
with the rest of the chrome outside presenter mode, shown only in
`present` mode - kept "not too fancy" per the human's own words: no new
visual language beyond the existing toolbar button style. Every button and
key: keyboard-reachable, ARIA-labeled (I9), holds at 375px, respects
`prefers-reduced-motion` (reuses the existing camera-token RM handling,
adds no new animation). Reads: `app/src/viz/index.ts` (the
`handlePresenterKeydown` function, lines ~431-462, and its surrounding
event wiring), `CONTROLS.md` (the current key table this session extends
and must keep in sync), `app/public/beats.atni.json` (the beat shape this
session's new beats follow), `docs/research/discovery-2026-09-12.md` Track
C (the `single_tie`/`betweenness_top_n` measures already wired, if `s`
resolves to `single_tie`), the D-099c decision record once it exists.
Depends on: E2 not required (kind-filter and tribe beats do not need
edges to exist to render kind-based views, though committee-membership
EDGES from E2 make the committee beat visually richer - connectors, not
just colocated spheres). Gate: `npm run typecheck && npm run build && npm
run test` (app/) plus `pwsh scripts/check-all.ps1` 12/12 plus a director
screenshot review (HUMAN-ONLY per AGENTS.md, same as S-R4b's gate) plus a
keyboard-only walkthrough and a 375px layout check (both HUMAN-ONLY per
AGENTS.md's accessibility-testing line).

### S-E4 - End-to-end system test: scripted rehearsal on synthetic data

Extends `scripts/e2e-remote-intake.ps1` (today: synthetic QR-target URL,
Node-sealed submission, local `wrangler dev` relay, `cn intake pull`,
`emit-approve-decision` example, `cn intake apply`, `export` - the
Degraded D8 path) into the FULL rehearsal the human asked to "prepare for
tests of the system": synthetic QR target -> the ATNI-templated form
BUILT AND OPENED IN A REAL BROWSER via Playwright (`app/` already carries
`@playwright/test` 1.61.1 as a devDependency - this closes the D-089-owed
"real-browser smoke of the built form" item as a side effect, on the ATNI
template specifically, which is stronger evidence than the
research-network template the original D-089 finding was scoped against)
-> fill fields including E2's new committee-membership field -> seal and
POST to local `wrangler dev` -> `cn intake pull` -> facilitator approves
in the wizard (Playwright drives the actual `app/src/ui/intake/panel.ts`
review UI, not the `emit-approve-decision` shortcut, so this exercises the
production interactive path the D-080 debt note flagged as owed) -> `cn
intake apply` into a gitignored scratch queue (never the real queue path,
never inside the repo) -> app reload on `?group=atni-convention` -> assert
via the app's own projection (not just the CLI export) that the new person
entity AND its committee/org edges (from E2) are present and that at least
one presenter beat (from E3) highlights or includes it. Reads:
`scripts/e2e-remote-intake.ps1` (whole file, the existing automated
rehearsal this session extends rather than replaces),
`docs/runbooks/e2e-remote-intake.md` (the manual counterpart's procedure,
step-for-step, since this session automates roughly that same arc with
Playwright standing in for the human's browser actions),
`scripts/e2e/` (existing helper scripts this session's new Playwright
steps slot alongside), `app/src/ui/intake/panel.ts` (the review view
Playwright must drive - selectors, button labels), E2's and E3's own
blueprints/outcomes once they exist. Depends on: E2 (committee/org edges
to assert on) and E3 (a beat to assert against) should land first, though
the browser-form-smoke half is independently valuable even if E2/E3 slip -
implementer may split this into "smoke only" then "full assertions" if
E2/E3 are not yet landed when this session starts. Gate: the script runs
green end to end at least twice in a row (flake check) plus
`pwsh scripts/check-all.ps1` 12/12; this session's own completion claim
must paste the full script output, per the roster's `completion_rule`.

### S-E5 - Deploy-bar clearance for the ATNI form (extends S-R1, non-executing until gates clear)

`SESSION_ROSTER.yaml`'s existing `S-R1` already scopes the generic Pages
workflow + D8 hash step + keygen rehearsal + Playwright smoke + Workers
tier decision surfacing. This session is a SMALL addendum, not a
duplicate: retarget S-R1's Playwright smoke and build steps at the
ATNI-templated form (once E1 lands the build switch) instead of only the
research-network default, and add the ATNI-specific manifest/D8 check
(the deployed bundle's manifest must pin the ATNI-built `form/dist`, not
whichever template happened to build last). This session does NOT
duplicate S-R1's relay/Pages/keygen work - see S-R1 in `SESSION_ROSTER.yaml`
for that. Depends on: E1 (the build switch to retarget), and remains
gated exactly as S-R1 is: it produces no live deploy, and every
human_steps item S-R1 already lists (D-023 sign-off, real keygen ceremony,
Pages Settings click, Workers tier decision) stays open and human-owned.
Reads: `SESSION_ROSTER.yaml` (S-R1's own entry, to avoid re-reading files
S-R1 already scoped), `form/vite.config.ts` (E1's build-switch addition,
once it exists). Gate: identical to S-R1's gate, run again against the
ATNI-built artifact specifically.

## 4. Human input needed

- ATNI brand palette (hex or OKLCH values) for the dark-mode theme - E1 ships
  placeholder colors until then; nothing invented or scraped. FONTS RESOLVED
  (D-101): League Spartan SemiBold titles and Bold for bold body text, Lexend Medium
  subtitles, League Spartan presentation body, Arial traditional body (D-102; falls back
  to self-hosted Arimo). Self-hosted open-licensed files only; no runtime Google Fonts requests
  (`form/index.html:23` CSP, R8 offline, participant IP privacy).
- RESOLVED (D-100: state of residence). D-099c: what `s` should mean on the presenter keyboard (see research
  question (d)) - state of residence, something else, or repurposed to
  `single_tie` - before S-E3 starts, or S-E3 ships the `single_tie`
  fallback by default.
- Whether `tribe` should become a fixed enum/tags vocabulary (removing
  near-duplicate risk for real data) or stay free text - can wait until
  real ATNI data is being considered, since the synthetic fixture's exact-
  match values make this a non-issue for the rehearsal.
- D-023 sign-off on the intake consent text, the real keygen ceremony, the
  GitHub Pages Settings source switch, and the Workers Paid tier decision
  (D-092c) - all already-known human gates, restated here because S-E5 and
  the go-live checklist both depend on them and none of E1-E4 can close
  them.
- The recorded ATNI Climate committee approval of the network activity
  (CLAUDE.md real-data gate) - required before any real ingestion, wholly
  separate from and prior to any of the deploy-bar items above.

## Wow-factor candidates (from the creative pass)

From a separate creative pass (Fable 5.1, read-only, 2026-09-12), spot-checked by the
director: `client.queryPaths` / `queryNeighborhood` exist in `app/src/wasm/` with no
consumer in `app/src/viz/`; the fixture has 180 `member_of`, 66 `affiliated_with`, 45
`connected_to` ops (grep counts). Every idea below uses existing measures and the
existing highlight path; none adds a dependency or changes the ADR-004 pipeline. Ranked
by audience impact times comprehension over effort.

1. **Kind hotkeys plus a button rail** (this is S-E3). Captions carry projection counts
   ("15 standing committees", "60 members, 291 connections"). Consider `m` for members.
2. **Committee constellation.** One ring lights, its members brighten and are labeled,
   the rest dims in place; caption gives member count and how many other committees
   share members. `[` / `]` cycles the 15 rings. Reuses `focusEntityId` beats and
   `computeFocusSet`. Effort S.
3. **Arrival beat.** After a facilitator batch-applies and reloads, the new nodes glow and
   the camera fits them: "3 new members, connected to 5 committees and 2 organizations".
   A seen-ids set (UUIDs only, never names) diffed against the projection feeds
   `highlightedIds`. The reload is the trigger, so the graph still never listens
   (ADR-005). Needs S-E2's edges to say anything about connections. Effort M.
4. **Bridge builders and edge-of-network, with a name gate.** The two existing measure
   beats (`betweenness_top_n`, `single_tie`) plus a beat flag `nameOnStage` (default
   off): when off, the caption reads "N highlighted" and no person labels show.
   Turning it on for real participants needs an on-stage consent line, which is
   community-facing text (D-023 human gate). Effort XS-S. **Privacy-critical: the
   projector is a public surface; apply the name gate to ideas 2, 5, 6 as well.**
5. **Ripple from a committee.** `queryNeighborhood` layers light hop 1 then hop 2 within
   500 ms; caption "Within two steps of Energy: 31 members, 9 committees, 6
   organizations". Reduced motion shows all layers at once. Effort M.
6. **Connect two committees.** `queryPaths` restricted to `member_of`: a lit chain of
   people and rings; caption "via 2 members". Needs `computeFocusSet` to accept an edge
   set. Effort M.
7. **Tribes represented, aggregate only.** "Members from N Tribal Nations" as a count;
   never a per-committee tribe breakdown (small-N re-identification). Free-text spelling
   drift inflates the count until term normalization (D-093c) lands. Effort S for the
   count; a tribe relayout is L.
8. **Selection pulse.** The selection ring pulses once when a beat lands; steady glow
   under reduced motion. Shader edit inside the instanced halo layer. Effort S.

Rejected: edge particles along `member_of` edges - excluded by the presenter blueprint,
a second animated layer needing ADR-004 review, and on a projector the dim-versus-lit
hierarchy already says it more clearly.

Suggested roadmap slice (acceptance criteria verifiable by test or screenshot):

- **W1 presenter rail and kind keys** (fold into S-E3). In present mode `c`, `o`, `m`
  dispatch `presentBeatAdvanced` to the matching beat id (unit test); a 1080p screenshot
  shows the rail and the caption "15 standing committees".
- **W2 committee constellation beats.** Screenshot with one ring, its members and their
  labels lit, all else dimmed; the caption's member count equals the ring's `member_of`
  in-degree in the projection (test).
- **W3 measure beats with the name gate.** `presenter.test.ts` asserts a
  `betweenness_top_n` beat with `nameOnStage: false` produces a caption with no
  `display_name` that ends in "N highlighted".
- **W4 arrival beat** (after S-E2). Load the fixture, then a copy with three appended
  synthetic entity and edge ops; after reload exactly those three are lit (screenshot)
  and the diff function returns their ids (test).
- **W5 connect-two-committees.** The highlight set equals the `cn-graph` path's nodes and
  edges for a fixture pair (test), and a screenshot shows the lit chain.
