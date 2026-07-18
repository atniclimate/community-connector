# Snapshot Byte Ledger (D-044.5)

Tracks the size of the single-file offline snapshot (`app/dist/index.html`,
built by `npm run build:snapshot`) across every size-relevant unit. The build
script enforces the hard budget mechanically (I8, `scripts/check-size.mjs`);
this ledger makes the trend visible so the budget is never approached by
surprise.

## Budgets

- **Hard budget: 5.00MB** - enforced by `scripts/check-size.mjs` on every
  snapshot build; exceeding it fails the build (I8, R8).
- **Soft headroom ceiling: 4.20MB** - D-044.5 working limit. Any unit that
  pushes the total past 4.20MB stops and gets a DECISIONS.md entry (diet plan
  or scope call) before more size-relevant work lands.
- Discipline: one fixture per snapshot artifact; append one row here per
  size-relevant unit, measured with `npm run build:snapshot` (run from
  `app/`), recording the printed size.

## Ledger

Append one row per size-relevant unit. "Total" is the printed size of
`dist/index.html` from the size check; "Delta" is against the previous row.

| Unit id | Delta | Total | Date |
|---|---|---|---|
| baseline (pre-Wave-1) | - | 0.55MB (572.78 kB raw, 146.52 kB gzip) | 2026-07-17 |

Notes:

- The baseline predates any troika/font/label work; the Wave 0 dependency
  pins (troika-three-text, @fontsource/atkinson-hyperlegible) add zero bytes
  until a unit actually imports them into the snapshot bundle. The first
  labels unit must append a row.
- `dist/worker-*.js` is emitted alongside the snapshot but is not part of the
  single-file artifact or its budget; only `dist/index.html` is measured.
