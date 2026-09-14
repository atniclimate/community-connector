# ATNI Design System Digest

Read-only research digest for the design system the human just updated. Written for
S-E1/S-E3 planning. No assets, fonts, or HTML from the source were copied into this
repository; only color values, type names, and usage rules are recorded here (I1, D-059).

## 1. Source receipt

- Folder: `I:\ATNI_design-system\ATNI design system\` (127 files).
- Zip: `I:\ATNI_design-system\ATNI design system.zip` (33,174,396 bytes after the
  2026-09-13 asset fix; was 33,181,599).
- Asset fix 2026-09-13: the human replaced a bad image. The only file changed was
  `assets/logos/atni-climate-lockup.png` (1,086,970 bytes, modified 2026-09-12 23:59
  local). The corrected file has a transparent background: a white seal with black and
  red linework, "ATNI" in white with a black outline, "CLIMATE" in red. The
  design-system HTML did not change, so the usage rules in section 5 stand. Folder and
  zip re-verified identical (127 files each, same sizes).
- Zip-versus-folder: **match.** The zip's single top-level entry (`ATNI design system/`)
  contains the same 127 files as the folder, and every shared path has an identical byte
  size. No file exists in only one of the two, and no size differs.
- The design system is not itself version-numbered, but two internal timestamps establish
  its history:
  - `ATNI Design System.dc.html` (top level) footer: **"Revised 09/12/2026"** - this is
    today's file and the current, authoritative one.
  - `design_handoff_atni_climate_design_system/CLAUDE.md`: **"as of 07/16/2026"** - an
    older internal handoff bundle nested inside the current folder.
- **Important finding: the folder holds two different design systems, not one plus a
  copy.** The top-level `ATNI Design System.dc.html` (610 lines, dated 09/12/2026, uses
  Google-Fonts-hosted League Spartan + Lexend Deca + Cascadia Code, self-hostable via
  `assets/fonts/{League_Spartan,Lexend,Lexend_Deca,Cascadia_Code}`) is the current
  system. The nested `design_handoff_atni_climate_design_system/` folder is an older
  07/16/2026 bundle with its own `README.md`, its own `.dc.html` (613 lines), and its
  own font set (`Spartan MB` - a renamed OFL rework of League Spartan - plus four TeX
  Gyre families). Color palette is identical between the two; typography is not (section
  3). This digest treats the top-level, dated file as authoritative and flags the handoff
  bundle's typography as superseded-but-unresolved, since neither file says it was
  withdrawn.
- Non-public exclusions: none found. All hex values, font names, spacing numbers, and
  logo filenames read as intended for implementation. No contact info, unreleased
  material, or real-person data was present in any file read.
- Unread: `uploads/` PDFs and slide PNGs (filenames only, per scope) and the two
  seal-artwork scratch PNGs - not needed for token/type extraction.

## 2. Color tokens

All hex values below appear identically in both the top-level and handoff `.dc.html`
files and in the handoff `README.md`; citations point to the top-level file (current).

### Core palette
Source: `ATNI Design System.dc.html:69-79` (swatch grid, section 1.1).

| Token | Hex | Role |
|---|---|---|
| ATNI Red | `#E13D33` | Accent, emphasis, links/marks at large or bold sizes, primary buttons. Never a page background. |
| Red Alt | `#B02821` | Links/small red text on light grounds. Hover: `#8C1F19` (README only, not swatched in HTML). |
| Black BG | `#010B13` | Default web/product page ground; also heading/ink color on light. |
| Grey | `#636363` | Body text on light. |
| Grey Alt | `#3B3B3B` | Secondary text, Heading 3 color, minor labels. |
| Grey Element | `#141414` | Heavy rules/dividers; dark-surface "Raised" step. **Name collision**, see below. |
| Grey Overlay | `#1E242C` | Dark-surface "Overlay" step (menus, dialogs, hover). |
| Surface | `#FFFFFF` | Default light surface/card. |
| Surface Tint | `#F4F7FB` | Page tint / recessed light surface. |
| Text on Dark | `#E8ECF0` | Body text on dark (not pure white). |
| Red on Dark | `#F26B5E` | Links/small red text on dark grounds. |

### Program palette - Climate
Source: `ATNI Design System.dc.html:88-90`.

| Token | Hex | Role |
|---|---|---|
| Red Element | `#C30101` | Climate program tooling accent. |
| Grey Element (program) | `#98A1B4` | Program UI grey. **Name collision** with core `#141414` - flagged as an open issue by the design system's own README, unresolved. |
| Tribal Magenta | `#9732B4` | Reserved: land, Treaty, sovereignty content only, never a chart/button color. |

Drought/fire/heat/alert-severity colors are explicitly out of scope of this palette (they
carry regulatory/scientific meaning and live with the Dynamic Drought Module).

### Semantic / component colors
Source: handoff `README.md` (component color list; not separately re-swatched in the
top-level HTML but used inline throughout section 5 "Boxes and Containers", e.g.
`ATNI Design System.dc.html:373-414`).

| Purpose | Values |
|---|---|
| Muted / tertiary text | `#8A94A6` |
| De-emphasized text | `#C6CBD4` |
| Hairline borders | `#E7ECF3`, `#EEF1F6` |
| Neutral tag / secondary button bg | `#EEF1F6` on `#3B3B3B` |
| Status Active/"Prefer" | text `#1C8A5E`, bg `#E6F9F1` (tag) / `#EAF7F0` (callout) |
| Status Under Review | text `#B4740E`, bg `#FDF3E2` |
| Callout note/"Avoid" | bg `#FBEEED` |
| Callout sovereignty/reserved | bg `#F9EDFB`, tag bg `#F6E6F8` |

No data-viz/categorical palette is defined anywhere in the design system; the three
kind-role colors currently in the template (`kind-1/2/3`) have no counterpart here (see
section 6).

### Dark-ground doctrine
Source: `ATNI Design System.dc.html` section 1.5 "The Dark Ground"; `CLAUDE.md` (top
level, "RULE: Websites default to dark").

- Web/products default to Black BG `#010B13` (never pure `#000000`); documents/print stay
  light.
- Body text on dark is `#E8ECF0`, not pure white; pure white (`#FFFFFF`) reserved for
  headings and bold lead-ins.
- Bump body weight one step on dark (400 to 500); no light-weight faces on dark.
- Surface ladder (lightens as it rises): `surface-base #010B13` -> `surface-raised
  #141414` -> `surface-overlay #1E242C`.
- Radius is 0 everywhere (every edge square). Elevation on light grounds uses a subtle
  shadow (`0 1px 3px rgba(11,18,32,.08)`) for document cards only; dark grounds use no
  shadows, only the luminance ladder.

## 3. Typography

### Families and roles

**Top-level system (current, dated 09/12/2026)** - `ATNI Design System.dc.html:14`
(`<link>` to Google Fonts) and section 1.2 (`:94-96`):

| Family | Weights used | Role |
|---|---|---|
| League Spartan | 500 (Medium, large lede/body), 600 (Semibold, Title/H1) | Title, Heading 1, large lede/body text, presentation "Key Message" text, program wordmark |
| Lexend Deca | 200 (ExtraLight italic), 300 (Light), 400 (Regular), 600 (Semibold) | Subtitle, Subheading (eyebrow), Heading 2/3, Body, Caption, Quote, buttons, small labels |
| Cascadia Code | 300 (Light) | Code samples, inline technical text |

Self-hostable via `assets/fonts/League_Spartan/`, `assets/fonts/Lexend_Deca/`,
`assets/fonts/Cascadia_Code/` (variable-font `.ttf` files, OFL). `assets/fonts/Lexend/`
(the non-Deca family) is also present in the folder but is not referenced anywhere in
either `.dc.html` file - it appears to be leftover or reserved, not an active role.

**Handoff bundle (superseded-but-present, dated 07/16/2026)** -
`design_handoff_atni_climate_design_system/README.md` typography table: a completely
different family set - Spartan MB (OFL rework of League Spartan, weights 400-800), TeX
Gyre Heros (body), TeX Gyre Heros Condensed (subtitles/labels), TeX Gyre Adventor (quote,
italic), TeX Gyre Termes (serif, formal print). This bundle also defines the full
type-scale ratio table (Title 2.55x Body, H1 2.25x, H2 1.75x, H3 1.17x, Quote 1.3125x,
Caption 0.8125x Body, anchored Web 16px / Document 12pt / Deck 28px) which the top-level
file's own type-scale demo (`:552-561`) uses the **same ratios**, just remapped onto
League Spartan + Lexend Deca instead of Spartan MB + TeX Gyre. So the ratio system
carried forward; the font-family choice did not.

### Type scale (current, ratios from `ATNI Design System.dc.html:552-561`)

| Style | Family | Weight | Ratio (x Body) | Web 16px anchor | Tracking | Color |
|---|---|---|---|---|---|---|
| Title | League Spartan | 500 | 2.55 | 40.8px | -0.025em | `#010B13` |
| Subtitle | Lexend Deca | 300 | 1.25 | 20px | +0.01em | `#3B3B3B` |
| Subheading (eyebrow) | Lexend Deca | 400 | 0.8125 | 13px | +0.08em, uppercase | `#E13D33` |
| Heading 1 | League Spartan | 600 | 2.25 | 36px | -0.025em | `#010B13` |
| Heading 2 | Lexend Deca | 400 | 1.75 | 28px | -0.025em | `#010B13` |
| Heading 3 | Lexend Deca | 400 | 1.17 | ~18.7px | -0.01em | `#3B3B3B` |
| Small Label | Lexend Deca | 600 | 0.75 | 12px | +0.06em, uppercase | `#3B3B3B` |
| Quote | Lexend Deca | 200 italic | 1.3125 | 21px | normal | `#010B13` |
| Body | Lexend Deca | 300 | 1.00 | 16px | normal | `#636363` |
| Caption | Lexend Deca | 300 | 0.8125 | 13px | normal | `#3B3B3B` |

Line-heights (from the same block): Title 1.14, Subtitle 1.3, Subheading 1.4, H1 1.1, H2
1.15, H3 1.2, Small Label 1.4, Quote 1.4, Body 1.6, Caption 1.4.

Density/measure rules (`ATNI Design System.dc.html` section 1.4, `:149-178`): reading
measure targets 50-75 characters, ~66 ideal, capped at a 640px column at 16px; ragged text
uses `text-wrap: pretty`, headings use `text-wrap: balance`; body text is never tracked;
justification only with hyphenation in print columns >=50 characters wide.

### Reconciliation against D-101/D-102

| Role | Human's spoken rule (D-101/D-102) | Design system's current rule | Match/Conflict |
|---|---|---|---|
| Titles | League Spartan **SemiBold** | League Spartan, weight **500** (Medium) for the Title style itself; weight 600 is used for **Heading 1**, not Title (`ATNI Design System.dc.html:552,555`) | **Conflict** - the weight the human named as "SemiBold titles" is the system's Heading-1 weight, not its Title weight. Which literal weight number counts as ATNI's "SemiBold" is unstated by either party. |
| Bold body text | League Spartan **Bold** (700) | League Spartan appears only at 500 and 600 in the current system; no 700 weight is used anywhere in either `.dc.html` file for body-scale text | **Conflict** - the system has no defined "League Spartan Bold" body role at all. |
| Subtitles | **Lexend** Medium | **Lexend Deca**, weight 300 (Light), for Subtitle | **Conflict**, two ways: (1) Lexend and Lexend Deca are different type families (related SIL-published designs, not interchangeable weights of one family); the system's `assets/fonts/Lexend/` folder exists but is unused by either `.dc.html`. (2) weight: system uses Light (300), human said Medium. |
| Body text, presentation style (large text) | League Spartan | League Spartan, weight 500, is used for the Hall-mode "Key Message" slide text (`ATNI Design System.dc.html:273-283`) and for large lede paragraphs (`:31,62,192,209,249,308,422`) | **Match** on family; weight (500) is not specified by the human's ruling, so not a conflict, just unconfirmed. |
| Traditional body text (form fields, panels, reading text) | **Arial** (fallback Arimo) | **Lexend Deca**, weight 300, is the system's only defined Body style; Arial/Arimo do not appear anywhere in the design system | **Conflict** - the design system has no Arial/system-sans role at all; its entire reading-body role is Lexend Deca, a self-hosted OFL face, which is the opposite design choice from D-102's deliberate "leave it to the OS-installed proprietary font" reasoning. |
| Presenter-mode / projector captions and rail | (not explicitly ruled beyond "presentation style = League Spartan") | Hall mode uses League Spartan 500 for on-screen key messages and caption text (`:273-283`), sized in `cqw` container units, floor of 24px at deck anchor per the ratio table | No conflict recorded; this is the system's own presenter typography and is a plausible source for S-E3's caption styling. |
| 3D graph node labels | Atkinson Hyperlegible (current, human decides on any change) | Design system does not address 3D graph/node-label typography at all | No conflict - out of scope for the design system; D-101's own note that S-E1/E3 may *propose* a change stands unaffected. |

None of these conflicts are resolved here; they are listed in section 7 for the human.

### Lexend Deca and Cascadia Code
- **Lexend Deca**: carries every role except Title, Heading 1, and large lede/presentation
  text (i.e., everything League Spartan does not carry) - Subtitle, Subheading, Heading 2,
  Heading 3, Small Label, Quote, Body, Caption, and all button/UI label text.
- **Cascadia Code**: code samples and inline technical text only, weight 300, sized at
  `.88em` relative to surrounding body text (`ATNI Design System.dc.html:480`).

## 4. Spacing, radius, elevation, borders, components

Source: `ATNI Design System.dc.html` sections 1.4 and 5; handoff `README.md` spacing
table (identical numbers).

- **Radius: 0 everywhere.** No rounded corners anywhere in the system, on any surface.
- **Elevation:** light grounds use `0 1px 3px rgba(11,18,32,.08)` for document cards only;
  dark grounds use no shadows at all - hierarchy is luminance-only (the 3-step surface
  ladder in section 2 above).
- **Borders:** hairline `1px solid #E7ECF3` (light dividers) or `#141414` (dark section
  rules, e.g. `ATNI Design System.dc.html:304,367,420`); cards themselves carry no border,
  only a background shade.
- **Section rhythm:** section top margin 96px, section-rule padding-top 64px, subhead top
  margin ~52px, content column max 1080px, reading measure 640px.
- **Buttons:** no border, square corners; padding `11px 20px` in-content or `13px 22px`
  hero; Lexend Deca 400, 14-14.5px. Primary = ATNI Red bg, white text
  (`ATNI Design System.dc.html:413`). Secondary = `#EEF1F6` bg, `#3B3B3B` text (`:414`).
  Text link = weight 600 with a trailing arrow ("Text Link ->").
- **Cards:** ~`22px 24px` padding; light default is white on page tint (`#F4F7FB`); dark
  cards use the surface-raised/overlay steps; on slides a card is a transparent grey plate,
  never outlined.
- **Status tags:** small square pills, weight 600, ~12.5px, `4px 12px` padding; mark status
  only, never a button or link. Active `#E6F9F1`/`#1C8A5E`; Draft/neutral
  `#EEF1F6`/`#3B3B3B`; Under Review `#FDF3E2`/`#B4740E`.
- **Square bullets:** small filled `#010B13` squares (~7px in the web sample), never disc
  bullets.
- **Presentation layouts** (`ATNI Design System.dc.html` section 3):
  - *Room mode:* bright, structured, text allowed; red open-bracket anchors the top-left
    title block; League Spartan 500 title with a bold subtitle at half size beneath;
    seal top-right; off-topic items drop to grey rather than being removed.
  - *Hall mode:* full-bleed darkened background, text confined to the lower third (<=60%
    of frame height), <=40 words/slide, League Spartan 500 in `cqw` units (`2.3cqw` in
    the sample), one red-bold phrase per line, confidence qualifiers in parentheses.
  - *Emphasis grammar (both):* bold lead-in + colon + text; the "highlight pass" is a
    duplicate slide dropping all text to grey except red phrases, cross-fading ~0.6s;
    sections change by fade, never cut.

## 5. Logo usage

Source: `ATNI Design System.dc.html` section 4 ("4. Marks", `:305-364`); asset filenames
only, no copies made.

| Asset (filename only) | Use |
|---|---|
| `atni-climate-lockup.png` | Full lockup, "page badge" - report footers, brief mastheads, slide corners **on light ground**. Shown on white background. |
| `atni-climate-lockup-on-dark.png` | Same lockup, one-grey rework - Black BG pages, dark slides, night-mode surfaces. |
| `atni-seal.png` | **Standard mark.** Light backgrounds; default for print and general web. Explicitly shown as an "Avoid" case placed directly on dark (ring text disappears). |
| `atni-seal-on-dark.png` | **On-dark mark.** Dark surfaces, night mode, photography, colored backgrounds; a single-grey drawing that keeps the full ring legible without needing a plate. Marked "Prefer" over the standard mark on dark. |
| `atni-seal-subtle.png` | Low-emphasis: dark-mode chrome, watermarks, footers. |
| `atni-seal-subtle-original.png` | Pre-thickening original, kept for a future retrace session - not for use. |
| Program wordmark (no separate file; drawn inline as "ATNI" over "CLIMATE" in League Spartan 600) | Used without the seal, only in dense black product chrome. |

Rules: minimum mark size 32px tall (below that the outer-ring text stops reading); clear
space around the mark at least equal to the width of one red corner dot; never recolor the
red rings, rotate the mark, or stretch it off its circle; choose by background, not habit
(standard on light, on-dark on dark/photographic, subtle for watermarks, wordmark only on
black chrome).

Note: the design system's own text flags the seal artwork (`atni-seal.png`,
`atni-seal-on-dark.png`) as carrying "thin state-outline linework pending a full-resolution
retrace" - treat as not-yet-final for any high-resolution use.

## 6. Mapping proposal for S-E1 and S-E3

`app/src/theme/defaults.ts` defines these token keys (`DEFAULT_THEME_TOKENS`):
`bg.center`, `bg.edge`, `surface.panel`, `surface.scrim`, `text.primary`, `text.secondary`,
`text.onAccent`, `accent.primary`, `accent.focusRing`, `label.text`, `label.outline`, and
`kind.default-N.base` (currently 8 generic slots). The ATNI template
(`fixtures/templates/atni-convention.template.json:53-60`) currently sets
`theme.mode: "default-dark"` with three placeholder `roles` (`kind-1/2/3` = person,
committee, organization respectively, per `:11-46`).

Proposed mapping (design-system token -> theme-pipeline token):

| Theme token | Design-system source | Hex | Note |
|---|---|---|---|
| `bg.center` / `bg.edge` | Black BG | `#010B13` | The system's mandated page ground; `bg.edge` could darken slightly toward true black at the vignette edge, but the system says never pure `#000000` - keep `bg.edge` a shifted-lightness step of `#010B13`, not `#000000`. |
| `surface.panel` | Grey Element (surface-raised) | `#141414` | Matches the system's "cards, grouped content" step exactly. |
| `surface.scrim` | Grey Overlay (surface-overlay) | `#1E242C` | Matches "menus, dialogs, hover" step. |
| `text.primary` | Text on Dark | `#E8ECF0` | System's explicit "not pure white" body-on-dark rule. |
| `text.secondary` | Muted / tertiary text | `#8A94A6` | System's own dark-ground secondary/caption color. |
| `text.onAccent` | Black BG | `#010B13` | For text set on ATNI Red buttons/badges, check against the accent, not the page (component-level pair, see contrast note below). |
| `accent.primary` | ATNI Red | `#E13D33` | The system's primary-button and emphasis color; **on-dark contrast is borderline, see below.** |
| `accent.focusRing` | Red on Dark | `#F26B5E` | Purpose-built by the system for links/marks on dark grounds; brighter and safer than ATNI Red itself for a small-stroke focus ring. |
| `label.text` | Text on Dark | `#E8ECF0` | Node-label text in the 3D graph, consistent with body-on-dark. |
| `label.outline` | Black BG | `#010B13` | SDF outline color, matches page ground so labels read as "lifted" text. |
| `kind.default-1.base` (person) | *(no design-system counterpart)* | - | The system defines no categorical/data-viz palette (section 2). Options for the human: derive 3 hues from ATNI Red via the existing OKLCH `rotateHue`/`scaleChroma` helpers (keeping one true ATNI Red for person, since person is the template's primary entity), or ask the human directly - this is flagged as an open question in section 7. |
| `kind.default-2.base` (committee) | *(none)* | - | See above. |
| `kind.default-3.base` (organization) | *(none)* | - | See above. Tribal Magenta `#9732B4` is explicitly reserved for "Land, Treaty, and sovereignty content" and must **not** be repurposed as a generic kind color even though it is visually distinct. |

### Contrast/CVD checks the pipeline will run

`app/src/theme/contrast.ts` (`solveContrast`) walks a token's OKLCH lightness toward a
target `floor` (a WCAG contrast ratio) against a given surface, in `LIGHTNESS_STEP`
increments, and records an adjustment if it had to move. `app/src/theme/cvd.ts` (not fully
read this session, present in the directory) separately checks hue confusion between kind
colors. Both checks would run automatically once ATNI hexes are substituted into
`defaults.ts` or the template's `theme.roles`.

Computed contrast ratios (WCAG relative-luminance formula, same math as `color.ts`) for
the pairs the mapping above would actually render:

| Pair | Ratio | 4.5:1 (text) | 3:1 (UI/large text) |
|---|---|---|---|
| Text on Dark `#E8ECF0` on Black BG `#010B13` | 16.71:1 | pass | pass |
| White `#FFFFFF` on Black BG `#010B13` (headings) | 19.84:1 | pass | pass |
| Red on Dark `#F26B5E` on Black BG `#010B13` (focus ring) | 6.65:1 | pass | pass |
| **ATNI Red `#E13D33` on Black BG `#010B13`** | **4.64:1** | pass (barely) | pass |
| **ATNI Red `#E13D33` on current app `bg.center` `#0d1017`** | **4.46:1** | **fail** | pass |
| Muted text `#8A94A6` on Black BG `#010B13` (text.secondary) | 6.49:1 | pass | pass |
| **Tribal Magenta `#9732B4` on Black BG `#010B13`** | **3.24:1** | **fail** | pass |
| Current fixture kind colors (`#56b4e9`,`#009e73`,`#e69f00`) on `#0d1017` | 5.56-8.45:1 | pass | pass |
| Grey `#636363` / Grey Alt `#3B3B3B` / Red Alt `#B02821` on white | 6.01-11.20:1 | pass | pass |

**Flag:** if `accent.primary` is set to ATNI Red `#E13D33` and used as body-scale text
directly against the app's slightly-off-black `bg.center` (`#0d1017`, not the design
system's exact `#010B13`), the ratio drops to **4.46:1 and fails the 4.5:1 text
threshold** (it still clears 3:1, so it is safe for large text, icons, and UI strokes such
as the focus ring). Two fixes, either reversible: (a) set `bg.center` to the design
system's exact `#010B13` (which alone brings ATNI Red to 4.64:1, a bare pass), or (b) never
set small body text directly in ATNI Red - reserve it for large/bold text and the focus
ring, per the design system's own rule that ATNI Red is for "large or bold display sizes,"
which independently agrees with this contrast finding. **Tribal Magenta must never be used
as `accent.primary` or any kind color** - it fails 4.5:1 outright (3.24:1) and is reserved
for sovereignty content by the design system's own rule.

### Presenter-mode captions and rail

Per section 4, presenter captions/rail should draw from the Hall-mode typography: League
Spartan 500, sized relative to viewport (the system's own sample uses `cqw` units, floored
at the deck-anchor Caption size of 24px per the ratio table in the handoff bundle). The
rail's small always-visible controls (button rail, hotkey labels) map naturally to the
system's "Small Label" style (Lexend Deca 600, uppercase, +0.06em tracking) for
consistency with how the design system already labels UI groups (e.g. "CORE PALETTE").
Caption text under the graph (measure counts, e.g. "60 members, 291 connections") maps to
the system's Caption style (Lexend Deca 300, `#3B3B3B` on light / `#8A94A6`-equivalent
muted tone on dark).

## 7. Open questions and conflicts for the human

1. **Which design system is current?** Two dated, differently-typed systems coexist
   (top-level, 09/12/2026, League Spartan + Lexend Deca; nested handoff bundle,
   07/16/2026, Spartan MB + TeX Gyre), and nothing marks the handoff bundle withdrawn.
   Confirm the top-level file is sole authority, or say which handoff pieces still apply.
2. **Lexend vs. Lexend Deca vs. D-101's "Lexend Medium."** D-101 named plain Lexend at
   Medium; the system uses Lexend Deca at Light (300) for subtitles, and ships an unused
   `Lexend` font folder alongside the used `Lexend Deca`. Which family/weight is correct?
3. **League Spartan weight for "Titles" and "Bold body."** D-101 asked for SemiBold
   titles and Bold (700) body; the system's Title style is weight 500, its Heading 1 is
   600, and no 700 weight appears anywhere. Update D-101's weights, or add a Bold role?
4. **Traditional body text: Arial (D-102) vs. Lexend Deca (design system).** The system
   defines no Arial/system-sans role - its whole body-text role is self-hosted Lexend
   Deca. D-102's Arial reasoning (proprietary, OS-installed, broad device coverage) may
   still hold for the privacy-driven intake form, but it now diverges from the brand
   system's own body face. Does "traditional body" follow the brand system or keep Arial?
5. **No categorical/data-viz palette exists.** The template's three kind-role colors and
   the app's 8 generic `kind.default-N` slots have no design-system source - ATNI Red is
   a single accent, not a palette. Needs either human-supplied kind colors or an approved
   OKLCH-derived set seeded from ATNI Red (section 6).
6. **Grey Element name collision**, unresolved by the design system's own author: core
   `#141414` and program `#98A1B4` share one name. Do not reuse "Grey Element" as a single
   token key without picking one meaning.
7. **ATNI Red at body-text scale fails 4.5:1 against the app's near-black `#0d1017`**
   (4.46:1, vs. 4.64:1 against the design system's own `#010B13`). Confirm whether
   `bg.center` should move to the exact hex, and that ATNI Red stays reserved for
   large/bold text and UI strokes in the app, per the system's own usage rule.
8. **Seal artwork is explicitly marked not-final** (thin linework pending a retrace) -
   fine as a placeholder now, but any go-live asset should wait for that retrace.

## 8. Applied 2026-09-14 (CS-05)

A bounded slice of this digest is now live in `app/`, scoped to exactly what the
convention sprint plan (CS-05) ruled - nothing else was restyled, and open
questions 1-6 and 8 above are still unresolved and still the human's call:

- Presenter caption (`.cn-present-beat`, `app/src/ui/ui.css`): League Spartan
  SemiBold 600, self-hosted via the pinned `@fontsource/league-spartan` 5.2.8
  package (`600.css`, latin + latin-ext + vietnamese subsets), imported once in
  `app/src/main.ts`. Zero runtime font requests; verified against the built
  `dist/` output. Computed size clears 28px at 1920x1080 (the existing
  `clamp(1.125rem, 2.5vw, 2rem)` already hit its 32px ceiling at that width).
- Stage ground and caption text color, through the theme-token pipeline
  (`app/src/theme/defaults.ts`), since `bg.center`/`bg.edge` feed the canvas
  background as tokens (`app/src/viz/scene.ts`, not touched here) rather than
  CSS: `bg.center` is now Black BG `#010b13` exactly (closes open question 7 by
  option (a)); `bg.edge` is a shifted-lightness step off the same hue
  (`#00040a`), never pure black. `text.primary` is now Text on Dark `#e8ecf0`,
  and `.cn-present-beat` reads it via `var(--cn-text-primary, #e8ecf0)` instead
  of its old hardcoded literal.
- ATNI Red was not applied anywhere in this slice - the selection ring and
  kind colors are unchanged, per the plan's "reserved for the selection ring
  only if contrast passes" caveat and open question 5 (no categorical
  palette exists yet).
- Verification: `tsc --noEmit`, `vite build`, `vitest run` (170/170) and
  `scripts/pii-scan.ps1` all green; a new `theme.test.ts` suite pins the two
  new hex values, the WCAG 4.5:1 caption-on-ground ratio, the 3:1 floor for
  every default kind color against the new ground, and no adjustment/warning
  regression on the two fixture templates.
