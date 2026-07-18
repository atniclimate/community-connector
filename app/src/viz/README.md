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
