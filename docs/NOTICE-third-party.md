# Third-Party Asset Notices

This file records licenses of third-party assets shipped with or embedded in
Community Navigator build artifacts. It does NOT choose or imply a license for
this project itself - the project license is a human gate (see CLAUDE.md,
"Human gates") and remains undecided.

| Asset | Version | License | Purpose |
|---|---|---|---|
| Atkinson Hyperlegible (font) | via `@fontsource/atkinson-hyperlegible` 5.2.8 | SIL Open Font License 1.1 | Legibility-first typeface embedded in the 3D graph labels and UI; shipped as woff assets from the npm package. |
| troika-three-text | 0.52.4 | MIT | SDF text rendering for Three.js; draws the node labels in the 3D constellation view. |

Notes:

- The SIL OFL 1.1 permits bundling and redistribution of the font, including
  embedding in the single-file snapshot, provided the font itself is not sold
  standalone and retains its license notice. License text ships inside the npm
  package (`node_modules/@fontsource/atkinson-hyperlegible/LICENSE`).
- Dev-only tooling (compilers, test runners, bundlers) is not listed here;
  this file covers assets that ship to users.
