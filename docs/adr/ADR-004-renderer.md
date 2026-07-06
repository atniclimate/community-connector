# ADR-004: Renderer Choice - Custom Instanced Three Layer

- Status: accepted (evidence-based; spike data below)
- Date: 2026-07-06
- Phase: 3
- Drivers: 30+ FPS at 5,000 nodes / 10,000 edges on Intel Iris Xe (design
  brief performance budget; R6 scale), one-edge-system rule (brief revision
  ruling 1), replaceable-renderer stance (core untouched by this choice)

## Evidence

Deterministic spike (docs/blueprints/rendering-spike.md; app/spike/): same
seeded 5,000-node / 10,000-edge dataset, same scripted 20s camera path
(orbit, dolly-in, close orbit), DPR 1.5, no labels, measured on the
reference machine (i5-1340P, Iris Xe, Chrome 149, headed, 2026-07-06):

| Candidate | avg FPS | p95 frame ms | worst ms | draw calls |
|---|---|---|---|---|
| custom instanced layer | **33.5** | 70.3 | 90.1 | **5** |
| stock 3d-force-graph | 26.1 | 54.0 | 54.3 | 9,864 |
| three-forcegraph | 22.0 | 54.1 | 70.6 | 10,125 |

A headless-Chrome control run showed the same ordering at larger margins
(44.1 vs 19.1 vs 20.7 avg FPS). Only the instanced layer meets the 30 FPS
acceptance bar, and its 5 draw calls versus ~10k means it alone has frame
budget headroom left for the layers this spike excluded: labels, halos,
picking, UI compositing.

## Decision

1. **The custom instanced Three layer owns all graph rendering** in Phase 3:
   one InstancedMesh per template shape kind, one merged LineSegments
   geometry for all edges (per-vertex endpoint-blended colors), halo shell
   instancing per the design brief, FogExp2, DPR capped at 1.5.
2. **One edge system**: the merged-geometry path. No 3d-force-graph link
   stack anywhere in product code.
3. **3d-force-graph and three-forcegraph are not runtime dependencies of the
   product app.** They remain devDependencies of the spike only (evidence
   artifact); if a future layout need wants d3-force-3d, that is a separate
   small dependency decision, not this ADR reopened - product layouts are
   precomputed (design brief, predecessor lesson).
4. Picking is raycast-on-InstancedMesh (instanceId); selection/hover state
   are instance-attribute writes per the design brief's animation rulings.

## Consequences

- We own scene management, picking, camera choreography, and label
  integration ourselves - accepted; the brief already specifies each, and
  the library paths fail the budget outright.
- p95 frame times (70ms) cluster in the dolly-in fill-rate section: halo
  overdraw is the first optimization target (half-res or distance-culled
  halos) if interactive close-ups jank once labels land.
- The spike harness stays in app/spike as the regression benchmark; rerun
  it when the render layer changes materially and update this table.

## Rejected

- Stock 3d-force-graph: 26.1 avg FPS, ~10k draw calls - under the bar with
  no headroom; per-node Object3D architecture is the structural cause.
- three-forcegraph: 22.0 avg FPS, same structural ceiling.
- "Hybrid" (library for management, instanced for rendering): rejected -
  two systems fighting over one scene graph was the exact desync risk the
  brief's critique flagged (two-edge-systems).
