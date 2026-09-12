# Blueprint: Presenter Mode (S-R4b)

Director: Claude. Implementer: Codex `grind`. Scope fence: `app/src/**`,
`app/public/beats.atni.json`, this file, `SESSION_ROSTER.yaml`. Never `core/`.
No new runtime dependency. No bloom, no particles. ADR-004 renderer untouched.

## UNIT 1 - presenter basics (one commit)

### State (I4: mode change only through the state machine)
- `app/src/state/state.ts`: extend `ViewMode` to
  `"overview" | "focus" | "story" | "present"`. Add a `PresentBeat` type
  (`id`, `label`, optional `focusEntityId`, optional `filter: { kinds?:
  readonly string[] }`, optional `measure?: string`, optional `topN?:
  number`) and a `PresentationState` slice: `{ beats: readonly
  PresentBeat[]; beatIndex: number; loadState: LoadState }`
  (`initialPresentationState()` mirrors `initialSearchState()`). Add
  `readonly presentation: PresentationState` to `AppState`.
- `app/src/state/actions.ts`: add `presentBeatsLoaded { beats }`,
  `presentEntered { beatIndex: number }`, `presentBeatAdvanced { beatIndex:
  number }`, `presentExited`.
- `app/src/state/reducer.ts`: add a `presentView(beat)` pure function
  alongside `resetView`/`focusView`/`storyView` (`{ mode: "present",
  focusedEntityId: beat?.focusEntityId ?? null, hoveredEntityId: null,
  storyId: null, storyStep: 0 }`). `presentEntered`/`presentBeatAdvanced`
  set `view` via `presentView` and set `presentation.beatIndex`.
  `presentExited` calls `resetView()` (same as `storyExited`).
  `presentBeatsLoaded` sets `presentation.beats`/`loadState`. Escape's
  existing exit path must cover `mode === "present"`.
- Load `app/public/beats.atni.json` once at boot (fetch, like other static
  config) and dispatch `presentBeatsLoaded`; entering present mode dispatches
  `presentEntered { beatIndex: 0 }`.

### Beats file
`app/public/beats.atni.json`: a plain array of `PresentBeat` objects, no
schema validation, no core involvement - this is presenter config, not a
persisted `Story` (`schemas/story-path.schema.json` is untouched). UNIT 1
beats use only `id`/`label`/`focusEntityId`/`filter.kinds`; leave `measure`/
`topN` for UNIT 2 to populate once meaningful.

### Camera (`app/src/viz/camera.ts`)
- Factor `beginFlight`'s tail (position/target lerp + flight-object
  construction) into a shared `beginFlightTo(camera, controls, toPosition,
  toTarget, durationMs, reducedMotion)` used by both the existing `flyTo`
  (unchanged behavior) and the new `zoomToFit`.
- Add `zoomToFit(rig, positions: readonly Vector3[], opts: { paddingWorldUnits: number; reducedMotion: boolean })`
  exported alongside `createCameraRig`: centroid of `positions` is
  `toTarget`; bounding radius is max distance from centroid + padding;
  direction is centroid-from-origin (brief rule 1, same origin fallback as
  `beginFlight`); distance = `boundingRadius / Math.sin(fov/2 in radians)`;
  duration is a new `RENDER_TOKENS.camera.beatDurationMs` (1200, matching
  brief token `motion.camera.beat`), clamped like `flightDurationMs`. Empty
  `positions` is a no-op.
- Add `RENDER_TOKENS.drift.presentAutoRotateSpeed = 0.2` (config.ts); while
  `view.mode === "present"` the rig's drift consumer sets
  `controls.autoRotateSpeed` to this value instead of the default, else
  unchanged. `OrbitControls.autoRotate` already satisfies fixed-world-up/
  no-roll (brief rule 3); `driftActive` already disables under reduced
  motion, so no separate RM branch is needed here.

### Chrome and legibility
- A `data-view-mode` attribute (or equivalent single toggle) on the app
  root drives chrome visibility via CSS: toolbar, left panel, right detail
  panel, legend hidden in `present`; restored on `presentExited`. Instant
  toggle, not an animation - no RM variant needed for this one.
- `app/src/viz/config.ts`: add `RENDER_TOKENS.label.capPresent = 30` and
  `RENDER_TOKENS.label.presentScaleMultiplier = 1.8`; `viz/labels.ts`'s cap
  and font-size selection branch on `view.mode === "present"` instead of the
  tier cap, still respecting whichever quality tier is active for
  everything else (present does not change DPR).
- Add `RENDER_TOKENS.halo.restingAlphaPresent = 0.35` (config.ts);
  `viz/halos.ts` uses it in place of `halo.restingAlpha` when
  `view.mode === "present"`. Selected-halo alpha is unchanged.
- Text lightness toward L 0.93: add a present-mode label text color token
  (e.g. `RENDER_COLORS.labelTextPresent`) rather than touching the OKLCH
  derivation pipeline (`theme/`) or its CVD/contrast solver - this is a
  fixed, pre-checked override for the fixed dark presenter background, not
  a new template capability.
- Fog: where the scene reads `RENDER_TOKENS.scene.fogDensity`, multiply by
  0.5 when `view.mode === "present"` (no new token needed). DPR cap is
  untouched, per the brief.

### Keyboard (present-mode only; wherever Escape/keyboard traversal already
lives, e.g. `viz/interaction.ts` or an app-level listener)
- `Space` / `ArrowRight`: dispatch `presentBeatAdvanced` with
  `beatIndex + 1` clamped to `beats.length - 1`.
- `ArrowLeft`: same, `beatIndex - 1` clamped to `0`.
- `Home`: dispatch `presentBeatAdvanced { beatIndex: 0 }`.
- `F`: camera-only, no action dispatch - call `zoomToFit` over the full
  projection's node positions (ignore the beat filter); padding from a new
  `RENDER_TOKENS.camera.fitAllPaddingWorldUnits`; reducedMotion from
  `state.ui.reducedMotion`.
- `Escape`: dispatch `presentExited` (reuses the existing exit-to-overview
  path already handling Escape for `focus`/`story`).
- Every key above is inert unless `view.mode === "present"`.

### Effects wiring
Whatever effect currently reacts to `view.mode`/`focusedEntityId` changes
to drive `rig.flyTo` (story/focus already do this - read `effects.ts` for
the exact pattern before adding a branch) gets a `present` branch: on
`presentEntered`/`presentBeatAdvanced`, resolve the active beat's
`filter.kinds` against the current projection to a position list and call
`zoomToFit`; if the beat has no filter, fit the full projection. Do not
duplicate the story machinery - extend the same effect, one more `case`.

## UNIT 2 - highlight beats (second commit, gated)

**Precondition met 2026-09-12 14:32:** R4a landed (`18d4a09` and four
commits before it) - `cn-api::graph_measures(group_id, viewer_ctx_json,
request_json)` returns `{ degree, betweenness, eccentricity,
shared_committee_jaccard }`, keyed by entity id, each carrying its own
`explanation` string (`core/crates/cn-api/src/dto.rs`). `cn-wasm` exports
it, mirroring `query_neighborhood`. Correction to the original assumption
below: `viz/focus.ts` has no separate "analysis" tier yet -
`computeFocusSet`/`focusRole` only know `focused | neighbor | unrelated |
base` for ONE `focusedId`. Extend additively, no new tier:

- `app/src/wasm/client.ts`: add `graphMeasures(groupId, viewer, request):
  Promise<JsonObject>` (mirrors `queryNeighborhood`). `app/src/wasm/
  protocol.ts`: add the `graphMeasures` request/response variant.
  `app/src/wasm/worker.ts`: add the dispatch case calling
  `wasmCore.graph_measures(group_id, viewer_ctx_json, request_json)`.
- `PresentBeat.measure` accepts `"betweenness_top_n" | "single_tie"` (the
  two Track-C-shortlisted measures; committee-member highlighting is
  already covered by UNIT 1's `filter.kinds`, it is not a measure call).
  `topN` applies only to `betweenness_top_n`.
- `viz/focus.ts`: widen `FocusSet.focusedId` to `string | null`; add an
  optional `highlightedIds: ReadonlySet<string>` (default empty) param to
  `computeFocusSet`, merged into the returned `neighborIds` so
  `focusRole` renders highlighted entities as today's "neighbor" role -
  no new role, no new shader path. Non-null when EITHER `focusedId` is
  set OR `highlightedIds` is non-empty. Existing 2-arg calls keep
  compiling.
- `viz/index.ts` beat-entry branch: for a beat with `measure`, call
  `client.graphMeasures`, pick the id set (top `topN` by
  `betweenness.value` desc, or every `degree.single_tie === true`), pass
  it as the new argument alongside `focusedEntityId` (null for these
  beats). Beats without `measure` are unaffected.
- Beat label surfaces the matched entities' `explanation` strings from
  the same response; extend UNIT 1's beat label, no new panel.
- Reduced motion: unchanged - `FocusBlend` already snaps under RM.

## Verification (both units)

`npm run typecheck && npm run build && npm run test` from `app/`, then
`pwsh scripts/check-all.ps1`. Grep the diff for any state mutation outside
`app/src/state` (I4). Manual dev-app check: enter present mode on `atni-convention`,
step through beats, confirm Escape exits and chrome hides. Screenshots to
`docs/design/screenshots/`. Visual acceptance is the human's per
`SESSION_ROSTER.yaml`'s S-R4b gate - this session does not claim it.
