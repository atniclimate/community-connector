Visualization layer for Community Navigator. Renders the permission-filtered
projection the store holds; no permission logic lives here (I2).

Modules:
- scene.ts / camera.ts - scene setup and camera rig (fly-to, damping, idle
  drift with reduced-motion suppression).
- layout.ts - deterministic seeded layout (v0, client-side).
- nodes.ts / edges.ts / halos.ts / labels.ts - instanced render layers.
- focus.ts - focus-mode dim/highlight (P1.2): focus set from the projection,
  node recolor/rescale, edge dual-color-buffer targets, blend animator.
- legend.ts - DOM overlay legend (P1.6) rendered from the live theme,
  with the "adjusted for readability" indicator.
- picking.ts - pointer picking dispatching store actions only (I4).
- quality.ts / config.ts - adaptive quality tiers and render tokens.
- index.ts - mountViz(container, store): wiring and the frame loop.
- presenter.ts - presenter beats (public/beats.*.json): highlight sets,
  camera decision, and the stage caption.

Presenter beats (D-103):
- Highlight and camera are decoupled. Every beat's highlight (kind filter,
  edge-kind filter, measure result, focusEntityId neighborhood) flows through
  the one focus pipeline (computeFocusSet -> node roles, edge target colors,
  label emphasis); no second render pass. The camera move is decided
  separately by `beatCameraMove`.
- `camera` (optional): undefined keeps the old rule (fly when the focused
  entity changes, else fit the beat's positions); `hold` never moves the
  camera, even with a focusEntityId, so a spotlight is opacity-only; `fit`
  always frames the beat's `filter.kinds` positions (all when none); `fly`
  always flies to the focused entity and fits when there is none. Reduced
  motion snaps either move (I9).
- `filter.edgeKinds` (optional): entities incident to an edge of those kinds
  are highlighted and those edges stay bright while the rest dim. Combined
  with `filter.kinds` the entity set is the intersection and only edges with
  both endpoints lit stay bright. Measure beats ignore both filters for the
  highlight set (the measure owns it); `filter.kinds` still shapes the fit.
- Captions show counts, never names: a measure beat reads
  "label - N highlighted" (including "- 0 highlighted" when the measure
  returns nothing; the bare label shows only while the call is pending);
  explanations are never concatenated on stage.
- Stage name gate (D-099, no person name ever on stage): `labelKinds`
  (optional) restricts a beat's labels to those entity kinds. The focus
  pipeline's label emphasis set (focused entity, neighbors, highlights - or
  every entity on an all-lit beat) is filtered through `stageLabelIds` before
  `setEmphasis`, so the focused person and lit people are never labeled on
  such a beat, only e.g. their committees; `[]` labels nothing; undefined
  keeps today's behavior. Same emphasis/visibility path, no new pass. In
  present mode the hover tooltip is suppressed outright
  (`hover.ts` `stageTooltipSuppressed`); node hover emphasis still shows.
  `beats-fixture.test.ts` requires every ATNI beat to carry `labelKinds`
  without `person`, and asserts every number in a caption against the
  fixture (people with a `connected_to` edge; entities per kind).
- `beats-fixture.test.ts` validates public/beats.atni.json against the
  atni-convention fixture (ids, kinds, edge kinds, measures, camera values,
  the finale shape) so a mistyped beat fails in `npm run test`.

Presenter hotkeys and rail (CS-06): in present mode, `c`/`o`/`m`/`p`/`1`/`End`
jump straight to the `committees` / `organizations` / `members` /
`shared-priorities` / `one-node` / `constellation` beat, resolved by beat id
at keypress time via `presenter.ts`'s `beatIndexForKey` - never by position,
so a key is simply inert if that beat id is missing from `beats.atni.json`.
A small ARIA-labeled button rail (`.cn-present-rail` in `ui.css`, built in
`index.ts`) mirrors the same keys plus Fit; it dispatches the identical
`presentBeatAdvanced` action so a button and its hotkey can never resolve to
different beats. The rail is operator chrome: hidden outside present mode,
hidden by default inside it, toggled with `r` (`presenter.ts`
`nextRailShown` / `railHidden`, pure); the `hidden` attribute removes its
buttons from the tab order, and leaving present mode resets the toggle. No
new animation; see `CONTROLS.md` for the full key list and the A9 cue sheet.
