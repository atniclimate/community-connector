# Rendering Spike

This throwaway harness compares three renderer candidates at 5,000 nodes and
10,000 edges with deterministic data, fixed camera choreography, no labels, no
user input during measurement, and DPR capped at 1.5.

## Run

From `app/`:

```sh
npm run spike
```

Vite opens `/spike/index.html`. For the director measurement run, use:

```text
http://localhost:5173/spike/index.html?auto=1
```

To run only the custom renderer:

```text
http://localhost:5173/spike/index.html?candidate=instanced
```

## Output

Each candidate warms up for 3 seconds, then measures 20 seconds of `requestAnimationFrame`
times over the same camera path: 8 seconds orbiting at radius 1200, 6 seconds
dolly-in to radius 500, and 6 seconds orbiting at radius 500.

The console logs one line per candidate:

```text
SPIKE_RESULT {json}
```

An auto run reloads the page between candidates to clear GPU state, accumulates
results in `localStorage`, and logs:

```text
SPIKE_ALL {json}
```

The table reports measured frames, average FPS, p95 frame time, worst frame
time, and `renderer.info.render.calls` when available.

## Acceptance Rule

The director records evidence for ADR-004 from the headed browser run on the
reference machine. A renderer meets the spike bar when it sustains 30 or more
average FPS for 5,000 nodes and 10,000 edges at DPR 1.5.

The spike is excluded from the production Vite build. It is included in
`npm run typecheck` so strict TypeScript still covers the harness.
