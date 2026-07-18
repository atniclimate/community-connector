# app/src/ui - UI components

Self-contained components with `mount(container, deps)` exports; layout and
wiring happen in `main.ts`. All data mutations flow through `app/src/state`
(I4); components render cn-api projections as handed to them - no permission
logic in the app layer (I2). Accessibility baseline per I9: keyboard-reachable,
ARIA-labeled, `prefers-reduced-motion` respected, layouts hold at 375px.

- `search.ts` - P1.4 debounced search box over the existing cn-api `search`
  operation (attribute hits only); combobox/listbox keyboard pattern.
- `detail.ts` - P1.5 entity detail panel over cn-api `entity_detail`; TSDF
  tier codes primary with generic plain-language secondaries (D-032);
  provenance rendered exactly as the payload carries it (D-033).
- `flat.ts` - P1.7 flat list/table reading view over the same projection the
  3D scene renders (D-035 accessibility down-payment).
- `format.ts` - pure formatting helpers for the detail panel.
- `dom.ts` - element construction helpers.
- `ui.css` - component styles on the derived theme's `--cn-*` custom
  properties.
