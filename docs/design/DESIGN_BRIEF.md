# Community Navigator Design Brief

Status: Phase 3 input. Synthesized from five research tracks (visual language, motion design, UI chrome, predecessor audit, performance + accessibility). Every numeric value in this document is a starting default for a named config token, never a hardcoded constant. The default theme ships as a group template JSON like any community's.

---

## 1. Aesthetic direction

**Name: Hearthlight Constellation.** The community is a night sky you are standing inside of, lit from within - not a dashboard observing you from above. Every person, place, and gathering is a softly self-lit body in a warm, deep, tinted dark (never pure black), connected by woven threads of light that brighten where relationships are strong. The graph is quietly alive: a barely perceptible orbital drift when idle, shimmer on only the strongest ties, and light that ripples outward from whatever you touch. Nothing moves that the user did not cause or cannot stop.

This direction serves community users in three specific ways. First, warmth is engineered, not asserted: warm-biased fill lighting, moderate chroma (OKLCH C around 0.10-0.13 at rest, saturation spent only on selection), rounded 10-16px geometry, a humanist accessibility-first typeface, and people-first vocabulary ("people, places, gatherings, stories" - never "nodes, edges, entities"). Surveillance aesthetics are the inverse of each of these: neon on flat black, KPI grids, cold uniform light, database nouns. Second, trust comes from constancy: the layout is precomputed and identical every time the file opens, selection dims rather than hides (you always see where your neighborhood sits in the whole), and data-governance context stays visible on every person's detail panel. Third, the look is a parameterized system, not a palette: the predecessor's proven dark-space language is the default template, but every hue, duration, radius, and label arrives from community JSON and survives to the screen unmangled (hence Neutral tone mapping, Section 2).

The emotional target for the first 10 seconds: splash fades, the whole community resolves into view with a single gentle zoomToFit, names of the most connected people are already readable, and the sky drifts almost imperceptibly. It should feel like arriving at a gathering, not logging into a console.

## 2. Visual system

All values below are tokens in the theme/config layer. Token names are indicative; the schema is defined in Section 5.

### 2.1 Renderer foundation
| Concern | Starting value | Token |
|---|---|---|
| Tone mapping | `THREE.NeutralToneMapping` (Khronos PBR Neutral, r162+). Never ACES - it desaturates and hue-shifts community palette colors | `render.toneMapping` |
| Pixel ratio | `min(devicePixelRatio, 1.5)`; drops to 1.25/1.0 under quality tiers | `render.dprCap` |
| Antialias | Renderer MSAA on when no composer; FXAA/SMAA final pass only if bloom tier active | `render.aa` |
| Draw call budget | < 100 per frame, monitored via `renderer.info.render.calls` | `perf.drawCallBudget` |

### 2.2 Background and depth
- Single-pass procedural background: radial gradient `bg.center` #0d1017 to `bg.edge` #06080d (tokens - communities may tint the sky), vignette folded into the same shader, dithered with interleaved gradient noise (`(1.0/255.0) * ign(gl_FragCoord.xy) - 0.5/255.0`) to kill banding. Cost < 0.2ms.
- Static starfield: 300-500 `THREE.Points`, per-star brightness variation, baked once, never animated (`bg.starCount`, `bg.starBrightnessRange`).
- Depth cue: `THREE.FogExp2`, color identical to `bg.edge`, density tuned so the far half of the node cloud shifts perceptibly toward the background (`scene.fogDensity`, start 0.0009 at layout scale 500 and tune on real data). No depth of field, ever.

### 2.3 Nodes
- One `THREE.InstancedMesh` per entity-kind shape (6 shapes max: sphere, cube, octahedron, tetrahedron, torus, cone - CVD redundancy, Section 7), low-poly (icosahedron detail 1 or 6-segment sphere, `node.geometryDetail`). 3d-force-graph's default per-node Mesh path is banned at this scale; hook instancing via `nodeThreeObject` with a shared instancing manager, or drop to three-forcegraph/custom renderer if the library fights it. Picking via raycast `instanceId`.
- Material: MeshLambert with the kind color split between diffuse and emissive at partial intensity (`node.emissiveShare` 0.45) so nodes read self-lit. No MeshStandardMaterial/PBR.
- Lighting (all tokens): cool key from camera-up (`light.key.color` #cfd8e6, intensity 0.9), warm fill low-opposite (`light.fill.color` #ffd9a8, intensity 0.5), ambient 0.25. The warm fill is a primary warmth lever.
- Per-instance color and scale live in instance buffers; all state changes (hover, selected, dimmed) are buffer writes, never material swaps.
- Sizes: resting radius from degree, mapped to `node.sizeRange` [3, 10]; primary tier 10, secondary 7, ghost 3 (predecessor's proven three-tier emphasis).

### 2.4 Glow
- Base glow: back-side fresnel halo shells (GitHub globe technique) - a second InstancedMesh sharing node positions at `halo.scale` 1.15x, `THREE.BackSide`, fresnel alpha falloff (c 0.6, p 6, `halo.falloff`). Faint on resting nodes (`halo.restingAlpha` 0.15), strong on selected (`halo.selectedAlpha` 0.8). Zero postprocessing, works in the offline build, one draw call per kind.
- Bloom: quality Tier A only (discrete GPUs). pmndrs postprocessing SelectiveBloom at half resolution, `bloom.threshold` 0.85, `bloom.strength` 0.65, `bloom.radius` 0.5, luminanceSmoothing 0.3, feeding only emissive-boosted selected/story nodes (emissive intensity vocabulary from the predecessor: selected 2.5, neighbors 0.8, story path 2.0, convergence hubs clamped 3.5). **Conflict resolved:** the predecessor audit says keep UnrealBloomPass; visual-language and perf-a11y say halo-first. We keep the bloom *look* but invert the default: halo shaders are the base glow on all tiers, bloom is an additive luxury on Tier A. Iris Xe never runs a composer. Tradeoff: Tier B loses light-leak across neighbors; the halo shell preserves 90% of the read at ~5% of the cost.

### 2.5 Edges
Three-layer ladder (3,000-10,000 edges):
1. Base: all edges in ONE merged `LineSegments` BufferGeometry, per-vertex colors blended 50/50 from endpoint node colors, alpha from weight (`edge.alphaRange` [0.08, 0.6], predecessor formula `0.35 + (width-2)/8 * 0.5` as the starting map), `transparent: true`, `depthWrite: false`. One draw call. Precomputed at load, recomputed only on theme/scheme switch.
2. Curvature: bezier curves only for parallel multi-edges between the same pair (`edge.multiEdgeCurvature` 0.3); everything else straight (or `edge.baseCurvature` 0.05 to separate near-coincident lines).
3. Emphasis: directional particles only on edges above a weight percentile computed from the data at export time (`edge.particleTopPercent`, null = derive), with an absolute cap of `edge.particleEdgeCap` 300 edges x `edge.particlesPerEdge` 2 = 600 particles max. Speed 0.006, width 2.5 (3.0 highlighted). Narrative "cascade" bursts reuse `emitParticle` (burst 5, interval 30ms, 300ms between hops) with AbortController cancellation, not activation-id flags.

### 2.6 Labels
- troika-three-text SDF labels (worker-generated atlas, crisp at any zoom). **Conflict resolved:** predecessor's SpriteText re-rasters canvas textures on every text change and blurs during camera flights; troika wins. We keep the predecessor's zoom-adaptive *policy* (distance thresholds gate initials -> last name -> full name; primary-tier people earn labels at greater distance) as the LOD brain on top of troika rendering.
- Sigma.js-style screen-grid density culling: one worthiest label per grid cell, ranked by selection state > degree > size. Cap `label.visibleCap` 60 (hard max 150). Always label: selected node, its highlighted neighbors, hovered node.
- Legibility: 1-2px dark halo/outline at 50-70% opacity behind light text (cartographic convention, not label plates). Labels fade before fog swallows their nodes.
- Update on camera-settle and state-change events, debounced ~100ms trailing - never per frame.
- Exactly one HTML tooltip element (name + kind + affiliation), solid Material A surface, projected to screen space, viewport-clamped, appears on keyboard focus as well as hover, dismissible with Escape.

### 2.7 Focus mode
Dim-and-desaturate, never hide. Priority ladder resolved in ONE pure function per state change (the predecessor re-implemented it in five accessors and shipped desync bugs): transition > analysis > story path > selection > user filter > view-mode ghost > base. Selected at full brightness + strong halo; 1-hop neighbors elevated; everything else to the dimmed token (L 0.35, C 0.03, toward fog color). Applied as one instanceColor buffer write + one edge vertex-color write, animated by a single uniform (Section 3).

## 3. Motion system

### 3.1 Motion tokens
Durations and easings are theme tokens emitted as CSS custom properties so DOM and WebGL read one source. Exits are 25-40% shorter than entrances and use accelerate curves. Every row lists its reduced-motion variant; "RM" mode is one `motionEnabled` state fed by `prefers-reduced-motion` (with change listener) AND a visible in-app toggle (WCAG 2.2.2 requires the on-screen control).

| Token | Value | Easing | Used for | Reduced-motion variant |
|---|---|---|---|---|
| `motion.hover.in` | ~90ms (damp lambda 12) | exponential damp | node hover scale 1.2x + emissive up | instant brightness change, no scale |
| `motion.hover.out` | ~150ms (damp lambda 6) | exponential damp | hover release (asymmetric = no strobing) | instant |
| `motion.micro` | 90ms | entrance cubic-bezier(0,0,0.38,0.9) | buttons, toggles, chips | keep (<=100ms is RM-safe) |
| `motion.tooltip` | 100ms delay + 110ms fade | linear opacity | tooltip show | keep, opacity only |
| `motion.panel.in` | 240ms | entrance cubic-bezier(0,0,0.38,0.9), translateX | sidebar/detail panel open | 150ms opacity fade, no slide |
| `motion.panel.out` | 180ms | exit cubic-bezier(0.2,0,1,0.9) | panel close | 120ms opacity fade |
| `motion.focus` | 300ms | expressive cubic-bezier(0.4,0.14,0.3,1) | focus dim/highlight blend (uFocusBlend) | 150ms opacity-only blend, no stagger |
| `motion.stagger.window` | <= 500ms total | per-item entrance curve, scale from 0.6 never 0 | neighborhood reveal ripple | 0ms - all states apply at once |
| `motion.pulse` | 800ms decaying (A*exp(-3t)*sin(4pi t)) | shader | selection confirmation pulse | instant steady glow + DOM outline ring |
| `motion.camera.near` | 600ms | tween.js Quadratic.Out | neighbor-to-neighbor refocus | crossfade cut (below) |
| `motion.camera.std` | 800ms | Quadratic.Out | click-to-focus fly-to | crossfade cut |
| `motion.camera.beat` | 1000-1200ms max | Quadratic/Cubic.Out | story-beat presets, zoomToFit | crossfade cut |
| `motion.camera.rmCut` | 120ms out / 200ms in | linear opacity on canvas | RM replacement for all flights | (is the RM variant) |
| `motion.drift.speed` | autoRotateSpeed 0.2 (~1 orbit / 5 min) | deltaTime-corrected | idle ambient drift | off entirely |
| `motion.drift.resume` | 10s idle, then 2s ease-in from 0 | damp | drift restart | off |
| `motion.particles.speed` | 0.006 | n/a | edge flow on top-percentile ties | off; replaced by static +brightness accent on the same edges |
| `motion.splash.exit` | 700ms fade + scale 1.02 -> 1.0 | entrance curve | splash dismissal | 300ms opacity crossfade |
| `motion.scrim` | 700ms | standard cubic-bezier(0.2,0,0.38,0.9) | modal background dim (wizard only) | 150ms opacity |
| `motion.layoutChange` | ~1000ms position tween | ease-in-out cubic | view-mode relayout | jump to final positions |

### 3.2 Camera choreography rules (vection safety)
1. Dolly along the origin-through-node line (distance ratio pattern from the 3d-force-graph example); never animate FOV.
2. Aim locks in the first third of the flight (keep the library's lookAt-over-duration/3 behavior); position keeps traveling.
3. Fixed world-up, zero roll, ever. Single arc or straight paths, no S-curves.
4. During flights: fade particles out and dim peripheral nodes ~30% (cuts optical flow AND GPU load mid-flight); restore on arrival.
5. Cap angular velocity: > 120 degrees of orbit means lengthen duration, never spin faster.
6. One user action = one flight; never chain automatic moves. New click mid-flight redirects smoothly (damped), no jump cut.
7. Fast departure, soft landing: Quad/Cubic/Quint-Out family only. No linear. No bounce - camera damping is critically damped always; bounce is allowed only on small DOM elements.
8. Story presets are view modes + filtered `zoomToFit(1200ms, 20px, visibilityPredicate)`, never saved camera coordinates - they survive any relayout or dataset.
9. Panel + camera coordination: both fire from one event; panel finishes (240ms) inside the flight's first third; camera lookAt offsets by half the panel width so the focused node centers in the *visible* region. Close reverses: panel out fast, camera relaxes after.

### 3.3 Animating thousands of nodes: the shader-uniform strategy
Never per-node JS tweens. State lives in per-instance attributes written once per user event; progress lives in single uniforms:
- `aHighlight` (0 = dim, 1 = neighbor, 2 = selected) - one buffer write on click (~20KB, microseconds).
- `uFocusBlend` 0 -> 1 over `motion.focus` by one tween; fragment shader mixes base color toward dim/highlight targets.
- `aDelay` (distance-from-selected stagger) with in-shader local progress `clamp((uFocusBlend*T - aDelay)/dur, 0, 1)` - a full outward ripple for the cost of the same one uniform. Total window clamped to 500ms.
- Selection pulse: `uPulseTime`/`uPulseOrigin` uniforms, decaying sine in-shader.
- Continuous motion (hover, camera follow, drift settle) uses `THREE.MathUtils.damp` (frame-rate-independent exponential decay; lambda 4-6 camera, 8-12 hover). Raw `lerp(a, b, 0.1)` per frame is banned - it changes feel between 30 and 60 FPS, exactly the range Iris Xe spans.

### 3.4 Library budget
Zero new animation dependencies: tween.js already ships inside three-render-objects (camera), `damp` is in three core, DOM uses CSS transitions driven by the tokens. `motion/mini` (2.3KB) only if a multi-step onboarding sequence demands orchestration. All async choreography uses promise-returning primitives + AbortController - no setTimeout timing guesses (a documented predecessor race-condition source).

## 4. UI chrome

### 4.1 Panel materials
- **Material A (default, everything):** solid surface at `panel.alpha` 0.94-0.97, 1px border rgba(255,255,255,0.08), soft shadow. No backdrop-filter. Deterministic text contrast.
- **Material B (accent, max 1-2 small FIXED elements - the top toolbar only):** `backdrop-filter: blur(14px) saturate(140%)`, gated behind `@supports`, never on anything that scrolls or resizes. backdrop-filter over an animating canvas re-filters every frame and is the single biggest chrome perf risk on Iris Xe.
- Fallback ladder: one custom property `--panel-alpha`; `prefers-reduced-transparency` raises it to 0.99; missing backdrop-filter support downgrades B to A; the same knob appears in in-app accessibility settings (the media query is Chromium-only).

### 4.2 Typography (fits the 5MB offline bundle)
- One bundled face: **Atkinson Hyperlegible Next** (variable, wght axis only), subsetted with pyftsubset to Latin + Latin Extended (`U+0020-007F, U+00A0-00FF, U+0100-017F`) so Indigenous and community names with diacritics render correctly. Build-time audit flags template characters outside the subset.
- Two artifacts: woff2 for DOM `@font-face` (~40-90KB) and a woff/ttf subset for troika in-scene labels (troika does not read woff2; ~30-60KB). Both inlined as data URIs. Total budget <= 150KB. Fallback stack `system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif`.
- Body 15-16px, line-height 1.5+, `tabular-nums` for counts, body weight +50 on the wght axis over dark surfaces (halation compensation). Text color L 0.90-0.93, never pure white.

### 4.3 Layout
- **Desktop (Kumu/Felt grammar):** top toolbar ~48px (the one Material B element); left panel 280-320px (search, legend, people list - the parallel navigation surface); right detail panel 320-400px, resizable, collapsible to icon rail. All edge-docked (predictable landmarks for keyboard/SR region nav), all dismissible for full-bleed graph. No free-floating draggable cards.
- **Every panel = one skeleton:** Felt anatomy - header (title + close), scrollable body (the only scroll container), sticky footer with primary actions. CSS grid `auto 1fr auto`. One focus-trap implementation, one responsive collapse rule. The group-setup wizard is this same shell as the app's single modal over a dimmed (not blurred) canvas.
- **375px:** right panel becomes a non-modal bottom sheet with three snap points - peek ~96px (name, kind chip, avatar), half ~50vh, full (solid Material A). Visible drag handle that is a real button (Enter/Space cycles snap points), translateY transforms only, `touch-action: manipulation`. Graph keeps rendering behind peek/half; throttle the loop at full. Left panel becomes a hamburger list route; toolbar collapses to essentials + overflow. All targets >= 24x24 CSS px; raycast picking radius enlarged for touch.
- Canvas resize on panel drag is debounced to gesture end; canvas container gets `clip-path: inset(0)` (predecessor scar - prevents GPU-composited bleed over adjacent columns).

### 4.4 Legibility over the canvas
- Edge scrims: static linear-gradient bands behind docked chrome zones (e.g., top: `rgb(surface0 / 0.6)` to transparent over 120px). Free.
- In-scene labels: dark 1-2px halos (Section 2.6), never blur.
- DOM tooltips/toasts: always solid Material A + shadow.
- Compositing hygiene: all chrome in a fixed overlay root; panels animate transform/opacity ONLY; `will-change` applied at interaction start and removed at rest; no layout-property animation while the camera moves. Audit with DevTools Layers + paint flashing during flights.

### 4.5 Warmth mechanics
- Radius tokens: `radius.panel` 16px, `radius.control` 10px, pill chips for entity kinds.
- Avatars/initial chips (tinted from kind color, APCA-checked) lead every person row and detail header - names and faces are the currency, not metrics.
- Empty states: warm second-person microcopy ("No stories here yet - be the first to add one") + duotone inline SVG illustrations filled with `currentColor`/accent tokens so they restyle per community. < 20KB total SVG budget.
- Vocabulary is template data: section headings, kind labels, and narrative framing come from the group template ("Ecological Relatives", "Homelands and Waters"), never from code.
- Detail panels keep the predecessor's visible data-governance block (sovereignty/consent status) - a trust affordance, not metadata clutter.
- Home screen is the full-bleed living graph with a warm invitation - never a KPI grid.

## 5. Data-driven theming contract

Templates are W3C DTCG-format JSON (`$value`/`$type`), validated at load with a friendly in-app diagnostics surface (never console-only).

### 5.1 What the template SUPPLIES (intent only)
| Input | Form | Required |
|---|---|---|
| Entity kinds | list of `{ key, labelSingular, labelPlural, hue (OKLCH h) or hex, shapeKey (from the fixed 6-shape set), iconRef }` | yes |
| Accent color | one hex/OKLCH | yes |
| Surface base hue | one hue for the "sky" and panels | optional (default Hearthlight) |
| Contrast level | normal / high | optional |
| Background tint pair | `bg.center`/`bg.edge` overrides | optional |
| Radius scale, motion multiplier | scalar tweaks | optional |
| Vocabulary and narrative | UI nouns, view-mode names + descriptions, story paths (node-id arrays + text), anchor node id | yes for views |
| Font opt-out | use system stack instead of bundled face | optional |

Templates NEVER supply: text colors, hover/selected/dimmed variants, focus ring color, edge alphas, surface elevation steps. Template authors skip states; the system cannot let a skipped state be unreadable.

### 5.2 What the SYSTEM derives (once, at template load, in a small TS module - culori or ~2KB hand-rolled OKLCH math; portable to the WASM core later)
- Per-kind state ramp in OKLCH: resting (L 0.72, C 0.11), hover (+0.08 L), selected (+0.12 L, +0.04 C), dimmed (L 0.35, C 0.03 toward fog), label tint, halo color, edge endpoint color. Perceptually uniform offsets mean earth tones and jewel tones get identical hierarchy.
- Surface ladder (Linear model): base hue at C <= 0.03, L 0.14 / 0.18 / 0.22 / 0.27 for surface.0-3; hover +0.04 L; pressed -0.03 L; borders surface L + 0.10.
- Text colors SOLVED (not validated) via APCA search per surface: Lc >= 90 body-critical, >= 75 body, >= 60 headings/large, >= 45 non-text UI; auto black-or-white `onAccent`.
- Focus ring: accent-hued, solved to Lc >= 45 against both canvas background and panel surfaces.
- Legend, edge gradient blends, scrim colors, illustration duotone pair.

### 5.3 Enforcement (accessibility as invariant, not review item)
At template load, before first render:
1. Every derived resting node color checked >= 3:1 non-text contrast against the fixed background; L auto-nudged until it passes.
2. All text pairs pass the APCA solve AND the WCAG 2.x 4.5:1 compliance floor (APCA tunes, WCAG 2 certifies).
3. CVD gate: simulate deuteranopia, protanopia, tritanopia on the kind palette; compute pairwise CIEDE2000; any pair below threshold triggers a warning with suggested adjustments and an optional auto-nudge of lightness (lightness differences survive all CVD types). Color is never the sole kind encoding regardless (shapes, Section 7).
4. Story paths, view-mode kind references, and anchor node validated against the loaded graph; failures surface in the diagnostics panel with plain-language messages for template authors.

## 6. Performance budget

Target: 30 FPS floor on Intel Iris Xe at 1080p, DPR 1.5, 5,000 nodes / 10,000 edges. Frame envelope 33.3ms; budget 25ms of app work (browser compositing headroom).

| Line item | Budget (Tier B, Iris Xe) |
|---|---|
| JS: instance/uniform updates, picking, tooltip projection, WASM calls | <= 8ms |
| Node instances (<= 10 draw calls, one per kind shape) | ~3ms |
| Halo shells (instanced, bounded quad size 1.5-2x radius) | ~2ms |
| Merged edge layer (1 draw call; fill-rate is the real cost) | ~4ms |
| Particles (<= 600) | ~2ms |
| Labels (<= 60 troika draws) | ~1.5ms |
| Background + fog + starfield | ~0.5ms |
| Headroom | ~4ms |

**Ceilings (tokens):** 5,000 nodes; 10,000 edges in one geometry; 600 particles absolute; 60 visible labels (150 hard max); < 100 draw calls; DPR 1.5. Layout is always precomputed (cooldownTicks 0, no browser simulation; live relayout runs in the WASM core or a worker and hands back positions).

**Quality tiers (GitHub globe pattern):**
- Tier A (discrete GPU): DPR up to 2.0, selective half-res bloom on, full particle budget.
- Tier B (Iris Xe target, default): DPR 1.5, halo glow only, particles top 5% capped.
- Tier C: DPR 1.0, particles off, resting halos off, label cap 30, hover raycast throttled.
- Tier D (emergency floor): half-resolution render upscaled; far nodes as a Points impostor layer.

**Degradation order:** bloom -> DPR -> particles -> resting halos -> label cap -> far-node Points LOD -> half-res render. **Detection:** 2-second startup probe (median frame time on a representative scene, cached in localStorage) picks the initial tier; at runtime, a rolling 3s median frame time > 33ms steps down one tier; step up only after 30s under 20ms (hysteresis). The tier is a user-visible setting - the machine's guess never overrides the person.

**Idle economics:** render-on-demand. With frozen layout the scene is static unless the camera, a hover/selection, or particles animate. Dirty-flag the loop; pause after N seconds idle and on `visibilitychange` (`pauseAnimation()`/`resumeAnimation()`); resume instantly on pointerdown/wheel/keydown/focus. Reduced-motion mode disables the loop-keepers entirely and is therefore also the performance floor mode - one shared code path.

**Bundle budget (5MB single file):** graph JSON is the biggest line - exporter strips unused fields, shortens keys, quantizes positions to 1 decimal (2k-5k nodes can otherwise be 2-4MB alone). Fonts <= 150KB, SVG illustrations <= 20KB, splash as SVG/CSS composition (never a base64 full-bleed PNG). `vite-plugin-singlefile` + `resolve.dedupe: ['three']`, exact-pinned three/3d-force-graph versions, Vite 7.x pin encoded in package.json, and a bundle-size CI check against 5MB.

## 7. Accessibility commitments

Mapped to the R9 launch criteria (R9.1 ARIA, R9.2 keyboard, R9.3 reduced motion, R9.4 375px), plus CVD and WCAG 2.2 bindings. These are acceptance items for Phase 3, not a post-hoc audit.

**R9.1 - ARIA and screen readers: parallel semantic DOM (Data Navigator pattern).** The canvas is a black box to AT; slapping `aria-label` on it is a locked door with a nice sign. Generate a navigable HTML structure from the same permission-filtered projection the WASM core hands the renderer: a landmark region with (a) a text summary ("Community graph: 240 people, 18 organizations, 3,400 relationships"), (b) a grouped, virtualized node list (~200 live elements) whose focus/activation drives the same selection state as canvas clicks. It mutates only on discrete state changes, never per frame. Adopt cmudig/data-navigator or replicate its structure/input split. One `aria-live="polite"` region announces selection changes debounced 150ms ("Maria Torres, person, Elder Council. 14 connections. Neighbor 3 of 14 of River Restoration Project.") and mode changes ("View filtered to people and places. 180 of 260 visible."). Panels are labeled landmark regions; all data-sourced strings rendered via `textContent` (the predecessor's unescaped innerHTML is an XSS hole once communities supply data).

**R9.2 - Keyboard traversal.** The graph is ONE tab stop (never 5,000): roving tabindex composite widget per ARIA APG. Grammar: Tab enters at last-focused or highest-degree node; Up/Down cycles the focused node's neighbors sorted by edge weight; Enter/Right walks to the highlighted neighbor; Left/Backspace returns along the path; Home jumps to the template's anchor node; type-ahead jumps by name; Escape clears. Focus changes ease the camera to frame the node (cut under RM). Focus indicator is a DOM-projected ring (`Vector3.project`), 2px+ perimeter, 3:1 contrast against scene and states (WCAG 1.4.11 AA, 2.4.13 as design target), never obscured by panels (2.4.11), warm light ring (#FFD98F class) that works across all templates because the background is fixed. No postprocessing OutlinePass. Every capability has a visible labeled control; keyboard shortcuts are accelerators, never the only path (the predecessor's letter-key-only features are disqualifying for this audience).

**R9.3 - Reduced motion.** One `motionEnabled` state = `prefers-reduced-motion` media query (with change subscription) OR the visible in-app "calm mode" toggle. The toggle is mandatory independently: edge particles run > 5s, so WCAG 2.2.2 Pause/Stop/Hide requires an on-page mechanism - the OS setting alone does not satisfy it. Every animation's RM variant is specified in the Section 3 table; the principle is reduce-don't-remove: what particles say with motion ("this tie is strong"), static brightness says without it. Nothing is information-bearing through motion alone. User-driven direct manipulation (drag-orbit, scroll-dolly) stays enabled; no un-imparted inertia.

**R9.4 - 375px.** Bottom-sheet layout per Section 4.3; single column; >= 24x24px targets (WCAG 2.5.8); enlarged screen-space picking radius; tap-based flows replace all modifier-click interactions (pathfinding becomes a two-step "connect these two" action); tooltips are pointer-event based and keyboard-triggerable, hoverable, and Escape-dismissible (1.4.13).

**CVD and redundancy.** Entity kind is encoded by shape AND color: fixed 6-silhouette instanced set, billboard icon glyphs on the near/focused tier beyond 6 kinds, dash-pattern rings for secondary membership (never a second hue), legend keyed by shape + color, plus the load-time CVD validator (Section 5.3). Roughly 1 in 12 men has CVD; in any real community someone cannot rely on hue.

## 8. What we are NOT doing

Rendering and color:
1. **No 3d-force-graph default per-node meshes at scale** - documented collapse at 5k-7k elements; instancing is mandatory.
2. **No full-resolution UnrealBloomPass** - the documented Intel-GPU failure mode (20-40 FPS, GPU pegged); bloom is selective, half-res, Tier A only.
3. **No ACES Filmic tone mapping** - it desaturates and hue-shifts the community-supplied colors that ARE the data; Neutral only.
4. **No pure #000 background / pure #FFF text** - banding, halation for astigmatic users, defeats fog layering, and reads surveillance-hacker.
5. **No MeshStandardMaterial/PBR on nodes** - wastes fragment budget for a look Lambert + emissive does better here.
6. **No DoF, SSAO, film grain, or OutlinePass render passes** - each adds fullscreen passes the iGPU cannot afford; fog is the depth cue, DOM ring is the focus indicator, grain folds into the background shader if ever wanted.
7. **No per-edge transparent meshes with depthWrite** - the classic iGPU fill-rate killer; one merged vertex-colored geometry, depthWrite off.
8. **No labeling every node, per-node sprites, or per-node HTML overlays** - grid-culled SDF set + one DOM tooltip only.
9. **No uniform neon saturation** - max-chroma-everywhere is the surveillance signature and vibrates on dark; resting C 0.10-0.13, saturation spent on selection.
10. **No animated starfield or non-optional idle rotation** - large-field ambient motion is the prefers-reduced-motion trigger class; drift is slow, yielding, and off under RM.

Motion:
11. **No 3000ms camera flights** - demo pacing reads as lag by the third click; 600-1200ms band.
12. **No FOV-animated zoom, camera roll, or camera bounce** - top vection triggers; dolly only, critically damped.
13. **No uncorrected per-frame `lerp(a, b, 0.1)`** - frame-rate-dependent feel across the exact 30-60 FPS range Iris Xe spans; `damp()` everywhere.
14. **No per-node JS tweens for mass state changes** - thousands of tween objects = GC spikes; attributes + one uniform.
15. **No blanket `* { animation: none !important }` for RM** - kills useful orientation transitions and does nothing for WebGL; gate JS animation through `motionEnabled`.
16. **No GSAP/Framer Motion** - tween.js already ships in the dependency tree and CSS + damp cover the rest; bundle discipline.
17. **No setTimeout choreography or magic timing constants** - promise-returning primitives + AbortController; the predecessor's patched race conditions are the cautionary tale.

Chrome and architecture:
18. **No backdrop-filter blur on large, scrolling, or resizing panels** - re-filters every animated frame (FoundryVTT/shadcn documented jank); glass is one small fixed accent or nothing.
19. **No animating layout properties (width/height/top) or blur radius** - transform/opacity only, always.
20. **No CDN fonts, icon fonts, or any remote asset** - breaks the offline single-file constraint outright.
21. **No hardcoded hex, radii, shadows, durations, node IDs, or narrative content in code** - in a data-driven-theming app, every unhooked constant is a place a community theme silently fails; the predecessor's config drift (node_resolution 6 vs 12) already shipped.
22. **No template-supplied raw text colors or state variants** - authors skip states; the system derives and solves them.
23. **No free-floating draggable cards, no modal-heavy flows, no KPI-grid landing** - occlusion, landmark-hostile, context-destroying, surveillance-coded; edge-docked panels, one wizard modal, full-bleed graph home.
24. **No `Graph.refresh()` or scene rebuilds for appearance changes** - full object reconstruction is a guaranteed frame-drop at 5k nodes; mutate buffers and materials directly.
25. **No innerHTML string UI with unescaped data** - XSS-shaped and listener-leaking; typed DOM construction, textContent, event delegation.
26. **No hue-only kind encoding, hover-only tooltips, per-node tab stops, bespoke SR shortcut sets, or PRM-as-sole-pause-mechanism** - each violates a named WCAG 2.2 criterion (1.4.1-adjacent, 1.4.13, 2.1.1, APG conventions, 2.2.2).
27. **No always-running render loop in hidden or idle tabs** - render-on-demand; a community tool lives in a tab all day.
28. **No loose 0.x dependency ranges on the graphics stack** - three breaks APIs between minors; exact pins + CI bundle check.

## 9. Phase 3 integration checklist

Ordered; each item is one work unit with a testable output.

1. Define the DTCG token schema (color roles, motion, radius, vocabulary, kinds) and author the default "Hearthlight" group template JSON - the default theme is just another template.
2. Build the OKLCH derivation module: kind ramps, surface ladder, state offsets; emit CSS custom properties at load.
3. Add the APCA text-color solver (apca-w3) + WCAG 4.5:1 floor check + 3:1 non-text auto-nudge, wired into template load.
4. Add the CVD validator (3 simulation matrices + CIEDE2000) and the friendly diagnostics surface for template errors (also consumes story-path/view-mode validation).
5. Renderer bootstrap: NeutralToneMapping, DPR cap token, background gradient shader with IGN dither + vignette, static starfield, FogExp2 tied to bg tokens, 3-light rig from tokens.
6. Instanced node layer: per-kind InstancedMesh (6 shapes), instanceColor/scale buffers, instanceId picking, positions from the WASM projection (cooldownTicks 0).
7. Fresnel halo shell layer (instanced, BackSide, falloff tokens) with resting/selected alpha states.
8. Merged edge layer: single LineSegments geometry, endpoint-blended vertex colors, weight-mapped alpha, depthWrite off; multi-edge curvature pass.
9. Focus-mode pipeline: adjacency index at load, single pure state-resolver function, aHighlight/aDelay attributes, uFocusBlend uniform, 300ms blend + 500ms-capped stagger, RM branch.
10. Motion module: token table as CSS custom properties + TS constants, `damp()` helpers, `motionEnabled` state (media query + change listener + visible calm-mode toggle).
11. Camera choreography: fly-to with early aim-lock and panel-offset centering, view-mode zoomToFit presets, vection rules (particle fade + peripheral dim during flights), RM crossfade-cut path.
12. Idle drift: autoRotateSpeed 0.2 with deltaTime, pause-on-interaction, 10s resume with 2s ease-in, engagement vetoes, RM off.
13. Particle layer: export-time percentile flagging with absolute caps, emitParticle cascade with AbortController, visible pause control (WCAG 2.2.2).
14. Label system: troika-three-text with subsetted woff, screen-grid density culling, zoom-adaptive progressive disclosure, dark halos, fade-before-fog, single DOM tooltip (keyboard-triggerable, Escape-dismissible).
15. Quality tier manager: 2s startup probe cached in localStorage, rolling-median runtime stepping with hysteresis, user-visible override; tiers A-D wired to bloom/DPR/particles/halos/labels.
16. Render-on-demand loop: dirty-flag invalidation, visibilitychange + idle pause, instant resume.
17. Selective bloom Tier A path: pmndrs composer, half-res, threshold 0.85, FXAA final pass, fully absent on Tier B and below.
18. Panel shell component (header/body/sticky-footer grid), Material A/B tokens, transparency fallback ladder, one focus-trap implementation.
19. Desktop layout: toolbar + left list panel + right detail panel, edge-docked, collapsible, debounced canvas resize, clip-path guard.
20. 375px layout: non-modal bottom sheet with 3 snap points (button drag handle, translateY only), hamburger left panel, 24px targets, enlarged touch raycast radius.
21. Typography: subset Atkinson Hyperlegible Next (woff2 DOM + woff troika), data-URI embed, character-set audit build step, fallback stack.
22. Parallel DOM layer: Data Navigator-pattern structure from the WASM projection, virtualized, bidirectional selection sync, canvas summary region, debounced aria-live announcer.
23. Keyboard traversal: roving tabindex composite, neighbor arrow grammar, type-ahead, projected DOM focus ring with contrast guarantees.
24. Splash screen: SVG/CSS composition, min-display reconciliation with engine-ready gate, 700ms exit into gentle zoomToFit, RM crossfade.
25. Warmth pass: legend generated from theme, empty-state SVG system on currentColor, avatar chips, template vocabulary threading, governance block on detail panels.
26. Build hardening: singlefile Vite config with three dedupe, exact version pins, JSON exporter field-stripping/quantization, 5MB CI bundle check.
27. Acceptance run: WCAG 2.2 criteria map (2.1.1, 2.2.2, 1.4.11, 1.4.13, 2.4.7/11/13, 2.5.8, 4.1.3) + 30 FPS verification on Iris Xe reference hardware + 375px walkthrough + RM walkthrough, all against the default template and one deliberately hostile test template.
