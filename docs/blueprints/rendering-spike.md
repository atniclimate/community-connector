# Blueprint: Phase 3 rendering spike (authored by director)

Design brief section 9 item 1. Question to answer with EVIDENCE: which
renderer meets 30+ FPS at 5,000 nodes / 10,000 edges, DPR 1.5, on the
reference machine (i5-1340P, Iris Xe) - stock 3d-force-graph, three-forcegraph,
or a custom instanced Three layer? Outcome feeds ADR-004 (renderer choice).
ONE edge system per candidate; no labels in this spike (labels are a
separately budgeted layer).

## Dependencies (app/, exact pins recorded by the implementer in
docs/ENVIRONMENT.md's table)

`npm install three 3d-force-graph three-forcegraph` plus
`npm install --save-dev @types/three`.

## Structure

- `app/spike/index.html` - minimal dark page: renderer select, "run all"
  button, results table, canvas container. No frameworks.
- `app/spike/data.ts` - deterministic synthetic dataset (seeded LCG, NO
  Math.random): 5,000 nodes over 6 kinds (sphere, cube, octahedron,
  tetrahedron, torus, cone; kind colors from the research-network fixture
  theme roles), 10,000 edges (mixed: a hub-and-spoke quarter, a ring
  quarter, random rest; weights 0-1), PRECOMPUTED positions (deterministic
  spherical cluster per kind, radius ~600) - we measure RENDERING, not
  force simulation. Export the same dataset shape to all three candidates.
- `app/spike/candidates/stock.ts` - 3d-force-graph: cooldownTicks(0) with
  precomputed fx/fy/fz (no simulation), nodeResolution 6, no labels,
  edge opacity by weight; nothing else fancy - this is the baseline.
- `app/spike/candidates/forcegraph.ts` - three-forcegraph object inside a
  hand-rolled Three scene/camera/renderer, same settings.
- `app/spike/candidates/instanced.ts` - custom Three layer per the design
  brief: one InstancedMesh per kind shape (low-poly: icosahedron detail 1
  or equivalent primitive), per-instance color from kind, one BackSide
  fresnel-ish halo InstancedMesh (shader material, alpha falloff), ALL
  edges in ONE merged LineSegments BufferGeometry with per-vertex colors
  (endpoint blend) and additive-ish transparency, FogExp2, capped DPR 1.5.
- `app/spike/harness.ts` - instrumentation:
  - camera choreography during measurement: 20s per candidate - 8s slow
    orbit at radius 1200, 6s dolly-in to 500, 6s orbit at 500 (fill-rate
    stress) - identical path for every candidate (deterministic, no user
    input).
  - rAF frame-time collection AFTER a 3s warmup; report frames, avg FPS,
    p95 frame ms, worst frame ms, and renderer.info draw calls (where
    accessible).
  - `?candidate=stock|forcegraph|instanced` runs one and logs
    `SPIKE_RESULT {json}` to console; `?auto=1` runs all three
    sequentially (full page reload between candidates via location
    change - clean GPU state) accumulating results in localStorage, then
    logs `SPIKE_ALL {json}` and renders the results table.
- `app/spike/README.md` - how to run (`npm run spike` -> vite dev server
  URL + `?auto=1`), what the numbers mean, and the 30 FPS acceptance rule.
- npm script: `"spike": "vite --open /spike/index.html"` (or equivalent
  documented invocation that serves app/spike within the existing vite
  root config; adjust vite config minimally if needed).

## Rules

- Deterministic everything (seeded LCG; fixed camera path; fixed durations).
- TypeScript strict; no `any` except at the 3d-force-graph boundary where
  its types force it (localize and comment).
- Spike code lives entirely under app/spike/ - it is throwaway evidence,
  not product code; the state-machine invariant I4 does NOT apply inside
  the spike, but NO changes to app/src.
- Keep `npm run typecheck` and `npm run build` green for the whole app
  (exclude spike from the production build if simpler - document choice).

## Definition of done

From app/: npm run typecheck pass; npm run build pass; `npm run build:smoke`
still pass; the spike page loads and `?candidate=instanced` completes a
measurement run in a headed browser (you cannot verify FPS headlessly - the
DIRECTOR runs the actual measurement; your job is a working, deterministic
harness). Root: pwsh scripts/pii-scan.ps1 pass.
Final message (to .codex/spike-result.md - never write that path yourself):
files created, check results, exact URL/steps for the director's measurement
run, and pinned dependency versions for ENVIRONMENT.md.
