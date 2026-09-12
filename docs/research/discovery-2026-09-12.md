# Discovery memo - 2026-09-12: deploy bar, reveal analytics, presentation mode

Status: DISCOVERY ONLY. Nothing in this memo was built. It scopes work for the roster
(`SESSION_ROSTER.yaml`) and drafts candidate decisions (suffix `c`) for the human to
record in DECISIONS.md. Companion files: `docs/planning/RECONCILIATION-2026-09-12.md`
(verified state), `TRACE.yaml`, `SESSION_ROSTER.yaml`.

Ground truth: HEAD `375e8b2` (2026-08-11), tree clean, 31 commits ahead of origin,
check-all 11 of 12 green (rust-clippy fails on one new 1.98 lint at
`core/cli/src/intake/keymat.rs:258`; toolchain drift, not a code change). The human
confirmed the ATNI convention date as 2026-09-14 and added that the mainstage reveal's
visuals outrank everything else for the sprint. Standing rulings are honored, not
revisited: ADR-004, D-032, D-037, D-051, D-056.4. Every pilot entry is TSDF T1 by ruling
(D-034); no tier is invented here.

## Track A - Secure intake to live: the gap list against D-059.8

D-059.8 (as amended by D-089) requires all of the following before the Pages form and
Workers relay go live. State verified 2026-09-12 (receipts in the reconciliation):

| Bar item | State | Owner |
|---|---|---|
| ADR-005 accepted after its round | MET (D-068) | - |
| P3.5/P3.6 intake pipeline working | MET (D-089; e2e passes on synthetic data) | - |
| Keygen ceremony EXECUTED | NOT DONE. Tooling exists (`cn intake keygen / fingerprint / selftest / backup verify`, `core/cli/src/intake/mod.rs:44-80`); the ceremony is off-repo by design, so no in-repo evidence can ever exist. `cn intake selftest --dry` is the rehearsal (ceremony doc section 5) | human, with an agent-run rehearsal on synthetic keys first |
| D-023 sign-off on the form text | NOT DONE. DRAFT banner at `docs/design/intake-consent-text-draft-2026-07-24.md:1-17` | human |
| Real-browser smoke of the built form (D-089) | NOT DONE. No Playwright or browser test exists under `form/` or `scripts/`; `app/` already carries `@playwright/test` 1.61.1 as a devDependency (registry latest 1.63.0) | engineering |
| GitHub Pages deploy workflow (D-089) | NOT DONE. `.github/` is absent | engineering |

What the deploy path needs today (facts, then the gap each implies):

- GitHub Pages project-subpath deploy. The current custom-workflow shape is
  `actions/configure-pages@v5`, `actions/upload-pages-artifact@v4`,
  `actions/deploy-pages@v4`, permissions `pages: write`, `id-token: write`,
  `contents: read`, environment `github-pages`, and the repository's Pages source set
  to "GitHub Actions" in Settings (a human click). The project URL matches the form's
  `base` (`form/vite.config.ts:73`). Gaps: (1) ADR-005 D8 pins the LOCAL build's
  manifest, so a CI build must be byte-identical to it or the puller halts on its first
  pull; `scripts/build-form.ps1 -CheckReproducible` exists (D-083) but no CI-vs-local
  comparison does. (2) The workflow must deploy exactly the manifest's file set and
  never the manifest (D8). (3) The build needs three non-secret inputs: the production
  relay origin (`CN_FORM_RELAY_ORIGIN`, `form/vite.config.ts:58`), the facilitator
  PUBLIC key, which does not exist until the ceremony runs, and the group template,
  which defaults to the research-network fixture today (`form/vite.config.ts:47-79`).
  (4) The guard against shipping the `localhost:8787` default origin (D-089 R4-5) is
  still absent.
- Cloudflare Workers + KV. Workers Free: 100,000 requests/day, 10 ms CPU per
  invocation. KV Free: 100,000 reads, 1,000 writes, 1,000 deletes, 1,000 lists per day,
  1 GB storage; writes take up to about 60 seconds to be visible everywhere. Workers
  Paid is USD 5/month minimum. Gap: each accepted POST performs about three KV writes
  (blob, ledger, and the KV-backed per-IP rate-limit window, D-084), so 300
  convention-day submissions plus retries and abuse sit within a factor of two of the
  free tier's writes/day, and the runbook's "billing ceiling" line has no tier decision
  behind it. The relay's `APPROXIMATE_BLOB_CAP` 500 and the 300 s consistency margin
  (D-085) are consistent with the 60 s propagation figure.
- wrangler drift. Relay pins `^4.120.1` (4.120.1 installed, absent from PATH); registry
  latest is 4.131.1. The changelog between them has no KV or `wrangler dev` breaking
  change; 4.122.0 auto-enables Node.js compatibility for `compatibility_date` >=
  2026-08-04. The relay's date is 2026-08-11 (`relay/wrangler.toml:14`) and its comment
  forbids `nodejs_compat` (`:16-17`), so an unpinned upgrade would silently widen the
  runtime surface. Gap: keep the pin through the pilot window; a bump needs an explicit
  flag decision and a relay test run (47 tests, not in check-all).
- libsodium-wrappers and CSP. The form pins 0.7.15; the registry line is 0.8.4.
  WebAssembly under an enforced CSP needs `'wasm-unsafe-eval'` in `script-src`; the
  form carries it (`form/index.html:22-23`, D-089 R4-1). Gap: none for the event; an
  upgrade regenerates the sealed-box vectors, so it is post-event work.

Ranked shortlist (Track A):

1. Simplest thing that works for the event: none of the remote path. The bar cannot
   clear by 2026-09-14 because two items are human ceremonies. Convention intake runs
   on the in-app facilitator entry path, which D-053 already names as primary and
   ADR-005 D2 names as the fallback. Engineering still ships the pre-step (clippy fix,
   toolchain pin) so check-all is green for every reveal commit.
2. Post-event, one Sonnet session (roster R1): the Pages workflow with a post-deploy D8
   fetch-and-hash step, the Playwright smoke in `form/` (seal, POST to `wrangler dev`,
   assert a receipt, under the real CSP), the R4-5 localhost build guard, a keygen
   rehearsal on synthetic keys, and the Workers Paid tier decision. Then the two human
   items, then the runbook.
3. Later: multi-key puller for rotation (F3), libsodium 0.8.x with vector regeneration,
   wrangler bump with an explicit compatibility decision, CI reproducibility
   attestation.

## Track B - Graph analysis for the reveal, with no LLM and no opaque score

What `cn-graph` exposes today: `build`, `shortest_path` (weighted and unweighted),
`neighborhood` (k-hop layers), `degrees`, `search` (`core/crates/cn-graph/src/query.rs`);
`cn-api` wraps paths, neighborhood, and search as JSON. There is NO external graph crate
(`core/crates/cn-graph/Cargo.toml` lists cn-model, cn-perm, serde, thiserror); the index is
a BTreeMap adjacency with a BinaryHeap Dijkstra. Betweenness, closeness, eccentricity,
components, community detection, and Jaccard are absent. The prompt's assumption that
petgraph is the base is false; petgraph 0.8.3 is available but adding it is a crate
decision, not a given.

Measures that answer the human's questions, each with a plain-language explanation the
detail panel can show:

| Question | Measure | Cost at 300 nodes | Explanation string |
|---|---|---|---|
| Major connectors | degree (exists) + betweenness centrality (Brandes, exact) | trivial (O(VE)) | "sits on N% of shortest paths between others" |
| Who is on the edge | degree 1 (single tie), eccentricity (BFS from each node), isolated components | trivial | "one connection; farthest from the center by K steps" |
| Affinities | Jaccard over shared `tags` (interests, committees); committee co-membership count | trivial | "shares 3 of 5 interests" |
| Relationship strength | edge weight = declared connection (intake "connected to") + count of shared committees + shared tags, each term listed | trivial | "connected (declared) + 2 committees" |
| Communities | deterministic Louvain (sorted node order, fixed tie-break) labeled by the group's dominant committee tag | small (a few hundred edges) | "mostly Climate Resilience and Energy members" |

Architecture rules: every measure is a `cn-graph` query over the permission-filtered
projection `cn-perm` hands it (I2); BTreeMap iteration keeps output deterministic; results
reach the app through a new `cn-api` JSON call in the style of `query_paths`.

Community detection options, ranked: (1) deterministic Louvain written into `cn-graph`,
no new dependency; (2) Leiden via `leiden-rs` 0.8.1 (created 2026-04, about 19k
downloads, gitcode-hosted) after a dependency review; (3) petgraph 0.8.3 plus a Louvain
layer, mature but it re-implements the index `cn-graph` already owns.

Term normalization design (new; for R3, not for this sprint):

- Alias table lives in the group template as an additive, optional top-level block
  (`schemas/group-template.schema.json` sets `additionalProperties: false`, so the schema
  gains one property and a version bump per the project's additive-field policy):
  `aliases: [{ canonical: "climate", attribute_ids: ["areas_of_interest"], terms:
  ["climate change", "global warming", "greenhouse gas reduction"] }]`.
- Matching runs in `cn-ingest` at plan-approval time only (never in the app, never in
  `cn-perm`): NFC and case-fold, trim, whitespace collapse (reuse `one_line` and
  `normalize_name` in `near_dup.rs`), exact alias lookup, a tiny suffix-stem list
  (plural s, -ing), then optional `strsim` 0.11.1 Jaro-Winkler at a fixed threshold
  (0.92) that can only SUGGEST. Every suggestion carries its rule name as the reason.
- Facilitator confirmation happens in the existing review view
  (`app/src/ui/intake/panel.ts`): each suggestion is an accept or keep-as-written
  choice; the decision file carries the per-tag choices; the durable owner
  (`approval.rs`) writes the canonical tag and preserves the as-written term in the
  intake provenance block (I6). Nothing is ever applied silently (I12: unconfirmed
  suggestions appear as warnings in the run report). D-056.4 is unchanged.
- Size: roughly 400-600 lines plus tests across cn-schema, cn-ingest, the decision
  format, and the panel; one build session (roster R3, Opus 5), then the mandatory
  adversarial round because it touches the approval plan.

Ranked shortlist (Track B): simplest for the event is degree, single-tie, and shared-
committee Jaccard on the synthetic ATNI fixture, plus betweenness if the R4 session has
budget; post-event is eccentricity, deterministic Louvain, and term normalization (R3).

## Track C - Presentation mode on the existing instanced Three layer

Specified by `docs/design/DESIGN_BRIEF.md` and implemented: halos (2.4; `viz/halos.ts`),
`uFocusBlend` and the priority ladder (2.7, 3.3; `viz/edges.ts:46`, `viz/focus.ts`),
troika labels with tier caps (2.6; `viz/labels.ts`), reduced motion (3.1; `main.ts:53`),
idle drift (3.1; `viz/camera.ts:44`), quality tiers and DPR caps (6; `viz/config.ts:2-6`),
instance picking (2.3), ARIA parallel DOM (7), story playback (`state/actions.ts:72-82`).
Specified but ABSENT: edge particles (2.5 layer 3; no `particle` in `app/src`), story-beat
presets as filtered `zoomToFit` (3.2 rule 8; no `zoomToFit` or `preset` in `app/src`),
bloom (2.4; Tier A only, and Iris Xe never runs a composer, so out of scope by the brief
itself). Not specified anywhere: a presenter mode (`state/state.ts:155,206`).

What a projected mainstage reveal still needs, each checked against I8 (5 MB) and the
Iris Xe budget (brief section 6):

| Need | Proposal | Brief basis | Fits? |
|---|---|---|---|
| Presenter mode | new view mode `present` through the state machine (I4): chrome hidden, label scale about 1.8x with the visible cap lowered to 30, stronger resting halos, keyboard beat advance, Escape exits | 2.6, 4.2, 4.4 | yes: code only |
| Auto-orbit | reuse drift (`camera.ts:44`) at speed 0.2 under the 3.2 rules: fixed world-up, no roll, angular-velocity cap, RM variant off | 3.1, 3.2 | yes |
| Story-beat camera presets | implement `zoomToFit(duration, padding, predicate)`; beats = view mode + filter predicate in story steps (the story-path schema carries no camera fields, so a minor bump) | 3.2 rule 8 | yes |
| Connectors and edge-of-network highlight | Track B measures feed the ladder's `analysis` slot as `aHighlight` targets, one blend per beat | 2.7, 3.3 | yes |
| Edge pulse travel | particle layer keyed by merged edge index, Points with per-particle progress, top-weight edges only, 300 x 2 = 600 cap, RM variant = static brightness | 2.5 layer 3, 6 | yes on Tier B; first thing the tier manager drops |
| Committee-affinity color | at most 8 hues per view (5.3 rule 5) on HALO color only, kind color and shape untouched (7), CVD gate via `theme/cvd.ts`; 15 committees means top-N plus "other" per beat | 5.2, 5.3, 7 | yes |
| Projector legibility | presenter toggle raises text L toward 0.93, halves fog density (`config.ts:91`), keeps Tier B DPR 1.5; verify with the CVD simulator and a 1080p projector rehearsal (P5.9) | 4.2, 4.4, 6 | yes |

three.js: the app pins 0.185.1; the registry latest is 0.186.0, one release behind. The
r185 to r186 migration adds `Object3D.dispose()` (subclasses must call `super.dispose()`),
renames `Source` to `TextureSource`, and makes `toTrianglesDrawMode()` in-place; none touch
InstancedMesh, LineSegments, ShaderMaterial, Raycaster, or OrbitControls as used here.
Freeze at 0.185.1 for the event. troika-three-text 0.52.4 (latest 0.52.5) is fine but sits
in devDependencies while `viz/labels.ts:2` imports it (hygiene, R4 pre-step).
`postprocessing` is not installed and stays out.

Budget: `dist/index.html` is 753.69 kB of the 5 MB budget, against a 2.0 MB allocation for
code, libraries, and fonts; everything above is code only. The external 2.1 MB worker is a
snapshot self-containment gap (R5), not a reveal blocker, because the reveal runs the
normal app build on the reference Iris Xe laptop. Dropped for the reveal: bloom, and
community detection beyond the synthetic fixture.

Ranked shortlist (Track C): simplest for the event is presenter mode, auto-orbit, zoomToFit
beats, and the connector and edge-of-network highlight from degree plus single-tie (roster
R4, one Sonnet session on synthetic data); stretch inside R4 if the gate is green early is
the edge pulse layer behind the quality tier; post-event is committee-affinity halos with
the CVD gate, Louvain-driven community beats, and the three 0.186 bump.

## Candidate decisions for the human

- **D-090c - Sprint priority and roster order.** For the 2026-09-14 reveal, the visual
  chain (R2 convention template on synthetic data, then R4 presentation mode) runs before
  deploy-bar clearance (R1). The deploy bar D-059.8 stays UNMET; nothing goes live for the
  convention. Rationale: the human's 2026-09-12 rider; two bar items are human ceremonies
  that cannot complete in two days.
- **D-091c - Convention intake path.** Convention-day intake, if any, uses in-app
  facilitator entry into the gitignored queue (D-053 primary path). This requires the
  D-023 sign-off on the consent text before 2026-09-14; without it the reveal is
  demo-only on synthetic data (D-072.1 permits DRAFT text for synthetic use only).
- **D-092c - Relay hosting tier.** Run the relay on Workers Paid (USD 5/month minimum) for
  any live window, because KV Free allows 1,000 writes/day and each submission costs
  about three writes. Spend for the relay is already opened by D-053; this fixes the tier.
- **D-093c - Term normalization.** Adopt the Track B design: alias table in the group
  template, matching in `cn-ingest` at plan approval, facilitator confirmation in the
  review view, as-written preserved (I6), Jaro-Winkler suggest-only, never silent.
  Scheduled as R3 with a mandatory adversarial round.
- **D-094c - ATNI convention template shape.** A `committee` kind with 15 fixed instances
  (Energy; Taxation; Education (K-12); ICWA; Law & Justice; Philanthropy; Telecomms &
  Tech; Food Sovereignty; Economic Development; Native Vote; TERO; Gaming; Drug Abuse &
  Prevention; Housing; Climate Resilience) and a `member_of` edge kind; person attributes
  `display_name` (text, required), `tribe` (text, optional), `role`, `affiliations`,
  `areas_of_interest`, `specialties`, `events_of_interest` (tags), `contact_email` (link,
  default visibility trusted), `contact_preference` (enum); a `connected_to` edge kind for
  declared connections; consent as the structural checkbox gate (D-030). All entries T1
  per D-034. Synthetic fixture lands in R2; nothing is written this session.
- **D-095c - Reveal build.** The mainstage reveal runs the normal app build on the
  reference laptop; snapshot self-containment (R5) moves after the convention.
- **D-096c - Toolchain pin.** Fix the one clippy lint and add a `rust-toolchain.toml` at
  the version that check-all is green on, so calendar drift cannot turn a clean tree red.

## Sources

- npm registry: three, troika-three-text, postprocessing, wrangler, libsodium-wrappers,
  @playwright/test (https://registry.npmjs.org/<package>/latest)
- three.js migration guide: https://github.com/mrdoob/three.js/wiki/Migration-Guide
- wrangler changelog: https://github.com/cloudflare/workers-sdk/blob/main/packages/wrangler/CHANGELOG.md
- Cloudflare docs: https://developers.cloudflare.com/workers/platform/pricing/,
  https://developers.cloudflare.com/workers/platform/limits/,
  https://developers.cloudflare.com/kv/platform/limits/,
  https://developers.cloudflare.com/kv/concepts/how-kv-works/
- GitHub Pages custom workflows: https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages
- GitHub Pages actions: https://github.com/actions/deploy-pages, https://github.com/actions/upload-pages-artifact
- MDN CSP script-src (wasm-unsafe-eval): https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/script-src
- libsodium.js CSP issue: https://github.com/jedisct1/libsodium.js/issues/196
- libsodium.js releases: https://github.com/jedisct1/libsodium.js/releases
- crates.io: strsim (https://crates.io/crates/strsim), petgraph (https://crates.io/crates/petgraph), leiden-rs (https://crates.io/crates/leiden-rs, https://docs.rs/leiden-rs)
