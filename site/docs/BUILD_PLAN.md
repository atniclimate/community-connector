# ATNI Climate - Site Build Plan

Status: **awaiting approval**. No implementation code written yet.

Scope decision (approved this session): the site is built under `site/` in the
`community-connector` repo on branch `claude/skills-spd6be`. The root `CLAUDE.md`
(the Community Navigator contract) is not touched. A second `CLAUDE.md` scoped to
this subtree is created at `site/CLAUDE.md`.

Design decision (approved this session): `design/reference/variant-b-clean.jsx` and
`variant-c-wild.jsx` do not exist on disk and were never read. The visual system
below is rebuilt from the brief's section 2 text. Every layout rule, the grid
overlay, and the tilt camera are original constructions, not ports.

---

## 1 · What was verified, and how

Everything in this section was read this session. DDM clone pinned at commit
`300703e`, `https://github.com/atniclimate/dynamic-drought-module`, verified via
`git -C /workspace/atniclimate/dynamic-drought-module rev-parse --short HEAD`.

### 1.1 · Region keys - `REGIONS`

`src/config/regions.ts:10-20` defines `RegionKey` as ten keys:

| Key | Label | Group |
|---|---|---|
| `washington_state` | Washington State | pnw |
| `columbia_snake_basin` | Columbia & Snake River Basin | pnw |
| `cascades` | Western & Northern Cascades | pnw |
| `central_oregon` | Central Oregon | pnw |
| `southwest_washington` | Southwest Washington | pnw |
| `south_puget_sound` | South Puget Sound | pnw |
| `national` | United States | explore |
| `alaska` | Alaska | explore |
| `hawaii` | Hawaii | explore |
| `british_columbia` | British Columbia | canada |

Table body at `regions.ts:54-194`. Default is `washington_state`
(`regions.ts:199`). Each region carries a `group` field (`pnw` / `explore` /
`canada`) - the region rail groups by it rather than inventing an order.

### 1.2 · View presets - `VIEW_PRESETS`

`src/config/presets.ts:75-109`. Five, as the brief said:

| Key | Label | Layers |
|---|---|---|
| `right-now` | Right now | `nadm-drought, aiannh, telemetry` |
| `this-week` | This week | `heatrisk, nws-alerts, aiannh` |
| `season-ahead` | Season ahead | `drought, aiannh` |
| `fire-risk` | Fire risk | `spc-fire-weather, nifc-fires, hms-smoke, aiannh` |
| `whose-land` | Whose land | `aiannh, bia-reservations, states` |

There is also a separate `MOBILE_HAZARD_PRESETS` set of three
(`presets.ts:50-73`: `hazard-drought`, `hazard-heat`, `hazard-fire`). It is the
module's own mobile rail. The site's chips generate from `VIEW_PRESETS` only;
duplicating the hazard rail outside the module would be a parallel list of exactly
the kind item 3 of the definition of done forbids.

Note `presets.ts:9-11`: **there is no `preset` URL parameter.** A preset is
expressed only as the `layers` list it produces. The site's chips therefore emit
`layers=`, not `preset=`.

### 1.3 · Layer keys and the default-on set - `LAYER_DEFS`

`src/config/layers.ts:176-221`. The default-on set is computed at
`layers.ts:266-268` by filtering `defaultOn`. Actual default-on keys:

| Key | Name | Line |
|---|---|---|
| `hillshade` | Terrain Shading | `layers.ts:182` |
| `nadm-drought` | North American Drought Monitor | `layers.ts:192` |
| `aiannh` | Tribal Lands | `layers.ts:199` |
| `bia-reservations` | Reservation Boundaries | `layers.ts:206` |
| `states` | State Boundaries | `layers.ts:207` |

**Drift found - do not trust the README here.** `README.md:126` documents the
`layers` default as `usdm,aiannh,bia-reservations,states,hillshade`. The config
disagrees: `usdm` is `defaultOn: false` (`layers.ts:171`, defined as
`DROUGHT_CONDITIONS_DEF` at `layers.ts:162-174` and spread into the array at
`layers.ts:190`), and the default surface is `nadm-drought`
(`layers.ts:192`). This is precisely why the chips generate from config. It is
DDM's bug, not ours; worth an upstream issue but out of scope here.

Deployer slots `tribal` and `treaty` are `defaultOn: false, uiHidden: true`
(`layers.ts:200-201`). The site never names them in a generated chip.

### 1.4 · The URL-as-state contract

`README.md:117-152`. Confirmed exactly as the brief describes:

| Param | Values | Default |
|---|---|---|
| `region` | the ten keys above | `washington_state` |
| `layers` | comma-separated layer keys | see 1.3 |
| `select` | `state:<postal>` e.g. `state:WA`; applied once then dropped | none |
| `embed` | `true` or `1`, hides the sidebar | `false` |

Additional round-tripping params exist (`view`, `week`, `dmode`, `sst`,
`outlook`, `basemap`) with the authoritative grammar in `src/state/url.ts`
(`README.md:130-133`). The site drives only the four documented above.

`README.md:138-140`: a `layers` list naming several surfaces resolves to the
first surface named. The site's chip generator must therefore emit at most one
surface-role layer per chip - which `VIEW_PRESETS` already guarantees by
construction (`presets.ts:13-16`).

### 1.5 · Stewardship note - reproduced, not paraphrased

From `public/data/README.md:30-35`, verbatim:

> **Stewardship note.** Anyone populating these files is responsible for the
> underlying authorizations. Tribal Lands, Treaty Areas, and any
> sovereign-jurisdiction polygons must be redistributed only with the relevant
> Tribal Nation's consent. The empty-placeholder pattern is a deliberate part of
> this module's design; please preserve it on contributions back upstream.

Supporting context for the About page, `public/data/README.md:10-28`: the map's
default Tribal geography does **not** come from bundled files. `aiannh` (US Census
AIANNH) and `bia-reservations` (BIA AIAN-LAR) fetch the publishing federal services
live at view time and redistribute nothing. The `tribal` / `treaty` slots ship as
empty placeholders, off by default.

### 1.6 · Brand text and stack

- `index.html:8` - title pattern `Dynamic Drought Module · ATNI Climate`
- `index.html:235` - "A project of ATNI Climate (Affiliated Tribes of Northwest Indians)"
- `index.html:370` - "ATNI Climate, the climate resilience program of the Affiliated Tribes of Northwest Indians (ATNI)"
- `package.json` - Vite `^8.1.3`, TypeScript `^7.0.0`, Playwright `1.61.1`, Node `>=20`; build is `tsc --noEmit && vite build`
- `.github/workflows/deploy.yml` - Node 22, `npm ci`, gate, `actions/configure-pages@v5`, `upload-pages-artifact@v3`, `deploy-pages@v4` with a 30s single retry for the known transient Pages failure

### 1.7 · Telemetry stations

`src/config/telemetry.ts:30` onward - `TELEMETRY_STATIONS`, nine stations
including `ihr` (Ice Harbor), `bono3` (Bonneville), `snotel_791`, `snotel_711`,
`mrso`, `pobo`, `wickiup`, `modo3` (Deschutes at Moody, USGS 14103000), `conw1`.
The site does **not** restate station IDs or readings; it links to the module.

---

## 2 · Two conflicts in the brief that need your ruling

Both are load-bearing and I am not resolving them unilaterally.

### 2.1 · The condition ramp does not match DDM (blocking for Conditions)

Section 2.1 of the brief gives the ramp as "from the DDM reader, non-negotiable":

```
NONE #A8D88F · D0 #C4D98A · D1 #F0D07A · D2 #FFB067 · D3 #FF9A6E · D4 #E42F27
```

**None of those seven hexes appear anywhere in DDM's `src/` or `index.html`.**
Verified by grep across the clone; zero matches. DDM's actual ramp is the official
NDMC USDM palette at `src/config/palette.ts:188-224`:

| Code | Label | DDM color | Brief color |
|---|---|---|---|
| D0 | Abnormally dry | `#FFFF00` | `#C4D98A` |
| D1 | Moderate drought | `#FCD37F` | `#F0D07A` |
| D2 | Severe drought | `#FFAA00` | `#FFB067` |
| D3 | Extreme drought | `#E60000` | `#FF9A6E` |
| D4 | Exceptional drought | `#730000` | `#E42F27` |

Two consequences:

1. **Same category, two colors, one screen.** The Conditions page embeds the live
   module. D4 renders `#730000` inside the iframe and `#E42F27` in the surrounding
   chrome. "One ramp everywhere a category appears" cannot hold across an embed we
   do not control.
2. **`NONE` asserts something DDM deliberately refuses to assert.** `palette.ts:226-235`
   is explicit: USDM publishes D0-D4 polygons but no analyzed-area mask, so bare
   ground "cannot be promoted to a confident 'no drought' reading." DDM's swatch is
   named `No D0-D4 category drawn` (`palette.ts:236-240`). A green `NONE #A8D88F`
   reads as "conditions are fine" - a claim the underlying data does not support.

**My recommendation:** keep your ramp - it is the better ramp on `--ink`, and the
official NDMC yellow is close to unreadable on a black ground. Two amendments:
rename `NONE` to `No category drawn` and treat it as a null state rather than a
"good" green, and put a one-line note beside the embed noting the module renders
official NDMC colors. Alternative, if fidelity to the module matters more than the
dark-ground reading: adopt `USDM_CATEGORIES` colors wholesale and use your palette
only for non-category UI.

### 2.2 · One typeface, or DDM's two

Brief section 2.1 and section 8: League Spartan only, "do not add a second family."
DDM ships **two** - `src/styles/app.css:4` and `:75-77`: League Spartan for display,
**Lexend** for body and data. Section 0's "matching whatever DDM already uses" and
section 8's "do not add a second typeface" therefore point opposite ways.

**My recommendation:** follow section 8. League Spartan only, weight 300 vs
700-uppercase as the whole hierarchy, exactly as specified. The embedded module
will render Lexend inside its own frame; that is the module's identity and the
hairline frame edge makes the boundary legible. No action needed unless you want
brand unification, which would be an upstream DDM change.

---

## 3 · Open questions - CAST and the PNW Weather Tracker

Section 1 asked me to search the org rather than guess. Result via
`list_repos` (19 repos visible):

**CAST exists and is public**: `https://github.com/atniclimate/CAST`, pushed
2026-07-18. But `https://atniclimate.github.io/CAST/` returns **404** - the repo is
public, the Pages site is not deployed.

> **Q1.** Link CAST to its repo URL and mark it "not deployed", or hold it as a
> reserved slot with no link until Pages is live? I will not invent a live URL.

**No repo named anything like "PNW Weather Tracker" exists in the org.** The
closest candidates, neither of which I will assume is it:

| Repo | Visibility | Pages | Note |
|---|---|---|---|
| `pnw-tribal-dashboard` | public | **200 - live** | name is closest to "PNW ... tracker" |
| `sovereign-skies` / `sovereign-skies-v2` | public | 404 | weather-adjacent by name |
| `maps` | public | 200 - live | unknown relation |

> **Q2.** Is the PNW Weather Tracker one of these under a different name, something
> not yet created, or private under another org? Until you say, it ships as a
> reserved slot that states plainly it is not deployed.

Also missing from DDM, contrary to section 1 of the brief - these files **do not
exist** in the repo at `300703e` and I could not read them: `CLAUDE.md`,
`ROADMAP.md`, `CHANGES.md`, `TODO.md`, `docs/PHASE_PLAN_090.md`. DDM's only root
markdown is `README.md`; `docs/` holds `COVERAGE_MATRIX.md`, `RELEASE_NOTES.md`,
`design/`. Nothing in the plan depends on the missing files, but the layer table,
embedding contract and architecture invariants came from `README.md` alone.

---

## 4 · Tokens

```css
:root {
  /* ground */
  --ink:        #07090B;
  --white:      #FFFFFF;

  /* accents */
  --red:        #FF8A80;            /* eyebrows, section markers */
  --red-solid:  #E42F27;            /* D4, current-value marks */

  /* structure and prose */
  --line:       rgba(255,255,255,.16);
  --prose:      rgba(255,255,255,.78);
  --quiet:      rgba(255,255,255,.55);

  /* motion */
  --ease:       cubic-bezier(.32,.72,0,1);

  /* condition ramp - pending the section 2.1 ruling */
  --c-none:     #A8D88F;
  --c-d0:       #C4D98A;
  --c-d1:       #F0D07A;
  --c-d2:       #FFB067;
  --c-d3:       #FF9A6E;
  --c-d4:       #E42F27;

  /* the only cool color in the system */
  --water:      #8FD8D0;

  /* geometry */
  --margin:     14px;               /* 24px at >=720px */
  --gutter:     10px;               /* 16px at >=720px */
  --baseline:   8px;
  --cols:       12;
}

/* Alerts page only */
[data-ground="day"] {
  --paper:      #DEDCD5;
  --day-ink:    #16191C;
  --banner:     #F2A63A;
  --respond-1:  #B4442A;
  --respond-2:  #7A3E1E;
  --respond-3:  #3C7E92;
  --respond-4:  #14181C;
}
```

Type: League Spartan only, self-hosted via `@fontsource/league-spartan` (woff2
copied into `dist/`, no runtime Google Fonts). Weights 300 / 400 / 500 / 600 / 700.
`font-display: swap` plus a `size-adjust` fallback metric so there is no layout
shift on font load (definition of done, quality floor).

Hierarchy is exactly two moves: **300** for the thing the reader came for, **700
uppercase, tracking `.14em`** for every label around it.

---

## 5 · Layout law

- 12 columns, every element spans whole columns. No arbitrary widths.
- 8px baseline. Every vertical measure is a multiple of 8.
- Margins 14px, 24px at >=720px. Gutters 10px / 16px.
- Sections butt against each other, separated by full-bleed 1px `--line` hairlines.
  No gaps between sections, no shadows, nothing floats.
- `border-radius: 0` globally, asserted once in the reset.
- Grid debug overlay ships in production: URL key `?grid=1` and the `g` shortcut,
  drawing the 12 columns and the 8px baseline in red at ~10% opacity.

Implementation is one CSS grid on a `.sheet` wrapper; every page section is a grid
child with a `--span` custom property. The overlay is a single `::after` with two
`repeating-linear-gradient`s, so it proves the real grid rather than a redrawn copy.

---

## 6 · Page-by-page wireframes

Target viewport 390x844. Home / Conditions / Alerts each fit `100dvh`, no scroll.

### 6.1 · Home - dark, no scroll

```
+------------------------------------------------+ <- 100dvh
| ATNI CLIMATE                    [ D2 ]         |  module stamp + grade
+================================================+  hairline
|                                                |
| ECOREGION                                      |  eyebrow, 700 up, --red
| Columbia                                       |  w300, 44px, cols 1-10
| Plateau                                        |
|                                                |
+------------------------------------------------+
| Ceded lands of the Yakama Nation, Umatilla,    |  --quiet, 13px
| and Warm Springs. Representation of cession    |  cols 1-11
| areas, not a depiction of jurisdiction.        |  <- required caveat
+================================================+
|                                                |
|   D2                          SEVERE DROUGHT   |  grade block
|   ^^ w300 88px                700 up 11px      |  cols 1-4 / 8-12
|                                                |
+------------------------------------------------+
| [][][][][][]                                   |  ramp, 6 cells, cols 1-12
| NONE D0 D1 D2 D3 D4                            |  labels beneath, 9px
+================================================+
| 34%      |  128%       |  1,410              |  three stats
| AREA D2+ |  SNOWPACK   |  CFS                |  cols 1-4 / 5-8 / 9-12
+------------------------------------------------+  hairline dividers
|      /\                                        |
|  /\ /  \    /\                                 |  streamflow trace
| /  v    \__/  \___                             |  --water, cols 1-12, 96px
+================================================+
| SOURCE  NADM · USGS · NRCS      sample data    |  source line, --quiet
+------------------------------------------------+
                                            [ | ] <- right-edge rail
```

The right-edge rail is a fixed 12px strip of hairline ticks, one per tool,
jumping straight to Conditions / Alerts / CAST / Tracker.

### 6.2 · Conditions - dark, no scroll

```
+------------------------------------------------+
| CONDITIONS                      [ D2 ]         |
+================================================+
| [Right now][This week][Season ahead][Fire...] |  chips from VIEW_PRESETS
+------------------------------------------------+  horizontal scroll, no bar
| [WA State][Columbia/Snake][Cascades][...]      |  chips from REGIONS, by group
+================================================+
| ?region=washington_state&layers=nadm-drought,  |  live query string, 10px
| aiannh,telemetry&select=state:WA&embed=true    |  mono-ish tracking, --quiet
+------------------------------------------------+
|                                                |
|          [ live DDM iframe ]                   |  fills remaining dvh
|                                                |  1px --line border
|                                                |
+------------------------------------------------+
| (o) live          NDMC colors inside frame     |  status pill + ramp note
+------------------------------------------------+
| [ tilt terrain ]                          off  |  variant-C mode toggle
+------------------------------------------------+
```

Status pill: `loading` -> `live` on iframe load, or `unavailable` after 14s with a
fallback link that opens the module in its own tab. Never an empty box. Each pill
carries its word beside its dot - no status depends on hue alone.

### 6.3 · Alerts - light, no scroll, prints on one page

```
+------------------------------------------------+  --paper #DEDCD5
| ALERTS                              ATNI       |  --day-ink
+================================================+
|                                                |
|  SEVERE DROUGHT                                |  banner #F2A63A
|  D2 · Yakima basin · through Sept              |  full-bleed, w300 32px
|                                                |
+================================================+
| SOURCE  NADM 2026-08-12 · NWS · sample data    |  black strip #14181C
+================================================+  white text
|                     |                          |
|   CONSERVE          |   CHECK ON               |  2x2 Respond grid
|   #B4442A           |   #7A3E1E                |  cols 1-6 / 7-12
|                     |                          |  each tile 8px-multiple
+---------------------+--------------------------+
|                     |                          |
|   WATER             |   REPORT                 |
|   #3C7E92           |   #14181C                |
|                     |                          |
+---------------------+--------------------------+
```

Print stylesheet: one page, same ramp and tile colors forced with
`print-color-adjust: exact`, nav and toggles `display: none`.

### 6.4 · About - dark, scrolls

`.doc` at 600px measure, 40px section rhythm, prose 15.5px / 1.62.
Sections: Who this is for · What this will not do · Classification (TSDF: public
agency surfaces are T0; anything unclassified defaults to T3 and does not ship) ·
Stewardship (the verbatim note from 1.5) · Contact `climate@atniTribes.org` ·
License CC BY-NC-SA 4.0.

### 6.5 · Learn - dark, scrolls

Same `.doc` measure. Instructional and dialectical - real questions answered
plainly. Planned set: What is a drought category, and who decides it? · Why does
"no category" not mean "no drought"? (draws directly on `palette.ts:226-235`) ·
Whose land am I looking at? · Why are Tribal Lands and Treaty Areas empty here? ·
What is a ceded area, and why is the caveat on every map? · What does this tool
refuse to do?

### 6.6 · Navigation - never persistent

| Gesture | Action |
|---|---|
| pull down from top edge | page switcher, tracks the drag, snaps past 60px |
| swipe left / right | tool deck |
| right-edge rail (Home) | jump to a tool |
| `arrow down` / `arrow up` | open / close switcher |
| `arrow left` / `arrow right` | deck |
| `Esc` | close |
| `g` | grid overlay |

No tab bar, no hamburger, no sticky header beyond the two-word module stamp and
the current grade. Routing is a hash (`#/home`, `#/conditions`, ...). The switcher
is a real focus trap with visible focus rings, and every page is reachable by
keyboard alone.

`prefers-reduced-motion`: terrain canvas off, drag-tracking transitions become
instant state changes, no flicker; static gradients replace animation.

---

## 7 · The variant-C surface (the one allowed place)

Conditions only, opt-in via the `tilt terrain` toggle, default off. Taken from C:
the tilt-and-compass camera (`deviceorientation`, `webkitCompassHeading`, iOS
`requestPermission()` behind an explicit tap, drag-to-look fallback if no event
fires within 1.6s) and the wireframe terrain read of the drought surface.

Rendered in the B palette on `--ink` with the condition ramp - no neon, no chrome,
no glitch, no CRT. Canvas 2D wireframe, no WebGL dependency. Off entirely under
`prefers-reduced-motion`. Budget: this is the only surface with any C influence,
well under the ten percent ceiling.

---

## 8 · Stack and structure

Vite + TypeScript, vanilla DOM. No framework, no state library, no CSS framework,
no component library, no router beyond the hash. Mirrors DDM's toolchain (Vite 8,
TS 7, Playwright, Node 22 on CI) without adopting Preact - five static pages do not
need it.

```
site/
  index.html
  public/fonts/            league-spartan woff2, self-hosted
  src/
    main.ts                hash router, page mount
    nav.ts                 pull-down switcher, swipe deck, keyboard
    grid-overlay.ts        ?grid=1 and the g shortcut
    ddm.ts                 URL builder + iframe status machine
    ddm-config.ts          GENERATED - do not hand-edit
    terrain.ts             the one variant-C surface
    pages/                 home.ts conditions.ts alerts.ts about.ts learn.ts
    styles/                tokens.css grid.css type.css pages/*.css print.css
  scripts/
    sync-ddm-config.mjs    regenerates ddm-config.ts from the DDM clone
  tests/                   playwright: dvh fits, keyboard path, print, contrast
  docs/BUILD_PLAN.md
  CLAUDE.md
```

**On item 3 of the definition of done.** Chips must come from DDM's config, not a
parallel list. DDM is a separate repo, so a build-time import is not available.
`scripts/sync-ddm-config.mjs` fetches `regions.ts`, `layers.ts` and `presets.ts`
from a pinned DDM tag, extracts keys, and emits `src/config/ddm-config.ts` with the
source commit stamped in a header. CI re-runs it and fails the build if the output
differs from what is committed - so drift surfaces as a red build instead of a
silently stale chip row. Values in section 1 are the first generation, taken from
`300703e`.

Deploy mirrors DDM's `deploy.yml`: Node 22, `npm ci`, typecheck, build, verify
`dist/index.html`, `configure-pages@v5` / `upload-pages-artifact@v3` /
`deploy-pages@v4`, including the 30s single retry for the transient Pages failure
DDM documents. Publishing to GitHub Pages is an outward-facing action - I will not
enable it without your explicit go-ahead, and the workflow lands disabled
(`workflow_dispatch` only) until you say otherwise.

---

## 9 · Sovereignty and honesty rules, as implemented

- TSDF: only T0 public agency surfaces are touched. Nothing unclassified ships.
- Tribal Lands and Treaty Areas stay empty placeholders. The site redistributes no
  sovereign-jurisdiction data and never caches or proxies a Tribal layer. The
  stewardship note is reproduced verbatim (section 1.5), not paraphrased.
- Every surface showing treaty or cession geometry carries the caveat that treaty
  polygons represent cession areas, not a definitive depiction of jurisdiction.
- No analytics, no tracking, no accounts, no cookies, no runtime third-party fonts
  or scripts. Selections never leave the device - the hash and the iframe query
  string are the only state, both client-side.
- License CC BY-NC-SA 4.0. Contact `climate@atniTribes.org`.
- "Casino" is legitimate vocabulary here (the Annual Convention is hosted at tribal
  casino resorts); nothing filters it.
- No fabricated repo names, URLs, station IDs, or readings anywhere.

**Numbers on Home and Alerts.** There is no backend, and the live data lives inside
the module's iframe where the page cannot read it. So the grade, the three stats and
the streamflow trace have exactly two honest options: fetch the same public agency
endpoints client-side, or show sample data labeled as sample data.

Plan: **phase 1 ships visibly labeled sample data** - the wireframes above carry
`sample data` in the source line and the Alerts source strip. **Phase 2** wires live
fetches to the endpoints in `src/config/urls.ts`, but only after verifying each one
sends CORS headers that permit a browser fetch from a Pages origin, with honest
`loading` / `live` / `unavailable` states matching the module's convention. If an
endpoint fails that check it stays sample-labeled rather than becoming a guess.

> **Q3.** Confirm phase 1 ships sample-labeled numbers. The alternative - Home
> waits for live data before shipping at all - is also reasonable, and your call.

---

## 10 · Verification before done

| Check | How |
|---|---|
| `npm run build` emits static `dist/` | CI, no backend, serves from any host |
| Home / Conditions / Alerts fit `100dvh` at 390x844 | Playwright, `scrollHeight <= clientHeight` |
| 768x1024 both orientations, down to 320px | Playwright viewport matrix |
| every element on a column and the 8px baseline | grid overlay screenshot diff |
| full keyboard path, visible focus everywhere | Playwright keyboard walk |
| WCAG AA on both grounds | contrast assertion over the token pairs |
| red-light readable | assert every status pill renders its word, not only a dot |
| `prefers-reduced-motion` | emulate media, assert canvas absent |
| Alerts prints to one page | print emulation snapshot |
| Lighthouse mobile >= 90, no CLS | CI run against the built `dist/` |
| no invented URLs | grep the tree for non-verified hosts |

---

## 11 · Build order

1. Scaffold, tokens, grid, type scale, grid overlay, `site/CLAUDE.md`
2. `sync-ddm-config.mjs` + generated config + the CI drift check
3. Router, pull-down switcher, swipe deck, keyboard, focus management
4. Home
5. Conditions - chips, live query string, iframe status machine
6. Alerts - daylight palette, print stylesheet
7. About, Learn
8. Tilt terrain mode
9. Playwright matrix, Lighthouse, contrast, print
10. Deploy workflow, landing disabled pending your go-ahead

Small commits, real messages, one task in progress at a time.

---

## 12 · Answer these three, and I start

1. **The ramp** (section 2.1) - keep your ramp with `NONE` renamed to a null state,
   or adopt DDM's official NDMC colors?
2. **CAST and the Tracker** (section 3, Q1 and Q2) - link CAST to its repo and mark
   it undeployed? And is the PNW Weather Tracker any of the three candidates?
3. **Sample data** (section 9, Q3) - phase 1 ships labeled sample numbers?

Nothing is implemented until you approve.
