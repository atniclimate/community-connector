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

**Precondition, checked before starting:** `core/crates/cn-api` must expose
a measures JSON call (git log on `core/crates/cn-api` and
`core/crates/cn-graph` for R4a's commit). At blueprint-writing time
(2026-09-12) neither has landed since the original scaffold
(`3dfe8a8`/`2af163c`) - R4a is still `planned` in `SESSION_ROSTER.yaml`.
**Do not start UNIT 2 until that changes.** If R4a has not landed by Sunday
2026-09-14 16:00, skip UNIT 2 and record it in `HANDOFF.md`'s next actions
instead of blocking on it.

- Extend `PresentBeat` with the already-reserved `measure`/`topN` fields
  (UNIT 1 added them to the type; UNIT 2 is the first thing that reads
  them).
- On entering a beat that names a `measure`, call R4a's new `cn-api` JSON
  endpoint (same client pattern as the existing `query_paths`/
  `query_neighborhood`/`search` calls in `app/src/wasm/client.ts`) to
  resolve the target entity ids (betweenness top-N, `single_tie`, or a
  committee's `member_of` set).
- Feed the resolved ids into `viz/focus.ts`'s existing `analysis` priority
  tier (already implemented per the design brief's ladder - transition >
  analysis > story path > selection > user filter > view-mode ghost >
  base) as one `uFocusBlend` target-state update, same mechanism the
  existing selection/story states already use. Do not add a second blend
  path.
- The detail panel renders each measure's plain-language explanation
  string (R4a's `cn-api` response carries it per the discovery memo's
  Track B table, e.g. "sits on 40% of shortest paths between others") next
  to the beat label. No new panel; extend whatever component already
  renders `EntityDetailDto`/beat label.
- Reduced motion: the highlight blend reuses `motion.focus`'s existing RM
  variant (150ms opacity-only, no stagger) - no new RM branch.

## Verification (both units)

`npm run typecheck && npm run build && npm run test` from `app/`, then
`pwsh scripts/check-all.ps1`. Grep the diff for any state mutation outside
`app/src/state` (I4) and any new dependency in `app/package.json` - both
must be clean before commit. Manual check in the dev app: enter present
mode on the `atni-convention` fixture, step through three beats with
Space/Left/Right, confirm Escape exits, confirm chrome hidden and labels
enlarged. Screenshots to
`docs/design/screenshots/present-2026-09-13-*.png`. Visual acceptance is
the human's per `SESSION_ROSTER.yaml`'s S-R4b gate - this session does not
claim it.
