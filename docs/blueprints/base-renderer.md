# Blueprint: base renderer (Phase 3, director; ADR-004 path only)

Sources: ADR-004 (decided: custom instanced layer, one merged edge system),
design brief sections 2/3/6 (tokens, motion, budgets), AGENTS.md I4/I5/I9.
TypeScript strict. Runtime deps: `three` only (already present).

## Part 0 - demo data (prerequisite: the app needs something to render)

`app/scripts/generate-demo-ops.mjs` (node, deterministic seeded LCG, no
system entropy): for EACH fixture template, generate a synthetic community
as ops JSONL to `fixtures/groups/research-network.ops.jsonl` and
`fixtures/groups/fisheries-committee.ops.jsonl` (committed; SYNTHETIC ONLY -
names like "Alder Riverbend", emails only @example.test; ~120 entities
across the template's kinds, ~300 edges of its edge kinds, 2-3 members with
Governance role, one story). Ops must satisfy cn-store validation (group
create with template_json, memberships, entity/edge creates with correct
attribute types, sort keys strictly increasing). Verify by running the
existing wasm smoke path: extend `app/scripts/smoke-node.mjs` (or add
smoke assertions) to load the research-network ops file end-to-end and
assert projected entity count > 100 for a member viewer. npm script:
`"generate:demo": "node scripts/generate-demo-ops.mjs"` (regeneration must
be byte-identical - seeded).

## Part 1 - state additions (minimal, stay pure)

- `data.kindMeta: Readonly<Record<string, { shape: ShapeName; label: string;
  colorRole: string }>>` populated by the load effect from the parsed
  template (the theming effect already parses it - share that parse).
- `view.hoveredEntityId: string | null` with a `entity hovered` action.
- Dev-mode group auto-load: main.ts effect loads the research-network
  template + ops.jsonl via the worker in dev builds (fetch from /fixtures -
  add a vite server alias/static serve for ../fixtures in dev only).

## Part 2 - app/src/viz/ modules (each under ~300 lines, I5)

- `layout.ts`: DECISION (record in code comment): v0 layout is computed
  client-side, deterministically, once per projection revision - seeded
  per-kind spherical clusters (spike technique, seed = hash of entity id so
  positions are stable across sessions and independent of array order).
  Proper pipeline-precomputed layout is a Phase 4+ concern; this module is
  the only thing that changes then.
- `nodes.ts`: one InstancedMesh per shape kind present (low-poly geometry
  table per template shape names); instance color from theme kind tokens;
  instance scale from degree (renderer receives degrees via cn-graph? NO -
  compute degree from projection edges locally, it is display math);
  hover/selected/dimmed are instance-attribute writes only. Rebuild on
  (projection revision | theme) change; reuse buffers when counts match.
- `edges.ts`: ONE merged LineSegments BufferGeometry; per-vertex colors =
  endpoint kind base colors; alpha from weight via vertex alpha + custom
  ShaderMaterial (brief ruling: mass dim/focus blends need both states in
  buffers - allocate a second color attribute now, blend factor uniform,
  even though focus mode lands later).
- `halos.ts`: per-kind BackSide shell InstancedMesh, fresnel-ish
  ShaderMaterial; DISTANCE-CULLED (only instances within camera distance D
  or nearest N=300) and tier-gated by quality manager (halos are the FIRST
  thing degraded - ADR-004 fill-rate finding).
- `picking.ts`: raycaster over node InstancedMeshes; instanceId -> entity
  id map maintained by nodes.ts; hover (throttled to one raycast per frame
  max) dispatches `entity hovered`; click dispatches focus/detail actions.
  Dispatch-only - no state held (I4).
- `quality.ts`: frame-time EMA (rAF deltas); tiers: A (halos on, DPR 1.5),
  B (halos culled harder, DPR 1.5), C (halos off, DPR 1.25), D (DPR 1.0).
  Degrade when EMA > 33ms for 60 frames; upgrade with hysteresis (EMA <
  22ms for 300 frames). Expose current tier for the status line/dev HUD.
- `camera.ts`: OrbitControls (three/examples, damped) + `flyTo(entity)`
  eased dolly (600-900ms per brief motion tokens); REDUCED MOTION (from
  ui.reducedMotion): flyTo is instant, no damping drift, no idle motion.
  All animation respects a global `motionScale` derived from the flag.
- `scene.ts`: background (radial gradient via large shader quad or scene
  bg + vignette per brief 2.2 with dithering), FogExp2 from theme bg tokens,
  lights per brief 2.3 (key/fill/ambient values as tokens from defaults).
- `index.ts`: `mountViz(container, store): () => void` (returns unmount).
  Subscribes to store; renders on-demand (dirty flags from state changes +
  continuous only while camera animates or damping active); pauses entirely
  when document.hidden. NO direct DOM outside its container; ARIA: the
  canvas gets role="img" and a live-region-updated aria-label summarizing
  selection (full a11y interface is its own later work - Session A - but
  the canvas must not be a black hole now, I9).

main.ts: mount viz under the status line; status line adds current quality
tier and entity count (subscribed, not polled).

## Test obligations (vitest; rendering itself is verified visually next cycle)

1. layout determinism: same projection -> identical positions; order
   permutation of entities -> identical positions per id.
2. demo generator determinism: two runs byte-identical (hash compare).
3. nodes: instanceId<->entityId mapping round-trips; theme change recolors
   without instance count change; degree->scale mapping bounds.
4. edges: vertex buffer sizes = 2 per edge; endpoint colors match theme;
   weight->alpha mapping clamped.
5. quality: EMA transitions with hysteresis (simulated frame times);
   degrade order halos-first.
6. picking dispatch: mocked raycast hit dispatches the right actions and
   holds no local state.
7. reduced motion: flyTo duration is 0 and drift disabled when flag set.

## Definition of done

From app/: npm run typecheck; npm run build; npm run build:smoke; npm test;
npm run generate:demo twice is byte-stable; extended smoke-node assertion
(Part 0) passes. Root: pwsh scripts/pii-scan.ps1. No changes under core/.
Dev-run visual check is the DIRECTOR's job next cycle (playwright headed) -
your job is that `npm run dev` + auto-load renders without console errors
(verify with a quick headed playwright/chrome check if available to you,
else state untested).
Final message (to .codex/renderer-result.md - never write that path
yourself): files, check results, test count, ambiguities resolved.
