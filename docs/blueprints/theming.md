# Blueprint: template-driven theming pipeline (Phase 3, director)

Sources: design brief sections 1, 5 (the data-driven theming contract:
community intent leads, legibility WINS), section 9 item 1; the two fixture
templates; AGENTS.md I9, I12. TypeScript strict, app-side (theming is
presentation - the core stays ignorant of colors per ADR-001 "presentation
metadata" ruling). No new runtime dependencies: implement the small color
math (sRGB <-> OKLCH, WCAG relative luminance/contrast) in-repo with unit
tests - no color libraries.

## app/src/theme/

- `tokens.ts`: the token model. A resolved `Theme` is a flat readonly map of
  named tokens, grouped by role:
  - surface: `bg.center`, `bg.edge`, `surface.panel`, `surface.scrim`
  - text: `text.primary`, `text.secondary`, `text.onAccent`
  - accent: `accent.primary`, `accent.focusRing`
  - per-kind (from the group template, up to 8 simultaneous): `kind.<id>.base`,
    `kind.<id>.hover`, `kind.<id>.dim`, `kind.<id>.halo`
  - motion + misc knobs the brief names as tokens ride separately later; this
    task is COLOR only.
  Each token value: `{ hex: string; source: "template" | "default" | "adjusted" }`
  - `adjusted` is the legend-indicator hook.
- `defaults.ts`: the default Hearthlight dark theme (brief section 2 values:
  bg #0d1017 -> #06080d etc.) as the base every template merges over.
- `derive.ts`: `deriveTheme(template: GroupTemplateDto): Theme` - the
  pipeline, EXACTLY this order (brief section 5, deterministic):
  1. Merge: template theme.roles map onto defaults by color_role -> kind.
  2. Derive per-kind variants: hover = lightness +0.08 in OKLCH (clamped),
     dim = chroma * 0.35 and alpha handled at the renderer, halo = base at
     fixed lightness 0.75 (halo shells read additive against dark bg).
  3. CONTRAST FLOOR: every text token vs its surface must reach WCAG 4.5:1;
     kind base colors vs bg.center must reach 3:1 (non-text graphic,
     WCAG 1.4.11). Failing colors move lightness in OKLCH (preserve hue,
     then chroma) by minimal steps until passing; mark source: "adjusted".
  4. CVD DISTANCE: pairwise deltaE-style distance between kind colors under
     deuteranopia/protanopia simulation matrices; pairs under threshold get
     the later kind nudged (hue rotate +-12deg first, then lightness) until
     the set passes or 8 iterations - then mark it "adjusted" and record an
     I12 WARNING in the theme report. Shape redundancy remains mandatory
     regardless (renderer concern, not this module).
  5. Emit `ThemeReport { adjustments: [{token, from, to, reason}],
     warnings: [...] }` alongside the Theme - NEVER silently (I12). The
     legend UI (later checklist item) shows "adjusted for readability" when
     any adjustment exists.
- `css.ts`: `applyTheme(theme)` writes tokens as CSS custom properties
  (`--cn-kind-person-base` naming) on :root for UI chrome; the renderer
  reads Theme directly (numbers, not CSS).
- Store integration: a `theme derived` action carries {theme, report} into
  AppState (extend state/actions/reducer minimally: `readonly theme` slice);
  derivation runs in an effect when a group template arrives. Reducer stays
  pure - derivation happens in the effect, result is dispatched.

## Schema

- `schemas/theme-tokens.schema.json`: JSON Schema (2020-12, versioned per
  I7) for a SERIALIZED resolved theme (token map + report) so snapshots can
  embed themes verifiably; keep in sync with tokens.ts (a vitest test
  validates a derived theme against the schema using ajv - already a
  devDependency).

## Test obligations (vitest)

1. Color math: sRGB<->OKLCH round-trips within tolerance on a grid of
   sample colors; WCAG contrast of known pairs matches published values
   (e.g. white on black = 21:1, #777 on white ~4.48:1).
2. Determinism: same template -> byte-identical Theme + report, twice.
3. Both fixture templates derive with zero errors; assert which (if any)
   tokens get "adjusted" and that the report matches.
4. Sabotage template (test fixture, inline in the test): 6 kinds all within
   a narrow hue band on near-black - pipeline yields a passing set with
   adjustments recorded and a CVD warning if the 8-iteration cap hits;
   NOTHING fails silently.
5. Contrast floor: a deliberately low-contrast text token gets adjusted to
   >= 4.5:1 with hue preserved (assert hue delta < 2deg).
6. Schema: derived themes validate against theme-tokens.schema.json;
   an unknown-major schema_version is rejected loudly.
7. Reducer: `theme derived` action stores theme + report; stale rules do
   not apply to theme (it is revision-independent) - document why in a
   comment (template changes arrive via new group loads in v0).

## Definition of done

From app/: npm run typecheck; npm run build; npm run build:smoke; npm test.
Root: pwsh scripts/pii-scan.ps1. No changes under core/. NO new runtime
dependencies (verify package.json dependencies section unchanged; ajv and
vitest are dev-only).
Final message (to .codex/theming-result.md - never write that path
yourself): files, check results, test count, the fixture-template
adjustment/warning outcomes from test 3, ambiguities resolved.
