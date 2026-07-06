# WASM Smoke

Build the web package first from `core/`:

```sh
wasm-pack build crates/cn-wasm --target web
```

Then build the page from `app/`:

```sh
npm run build:smoke
```

Open `app/dist-smoke/smoke/index.html` in a browser served by a static file
server. The page constructs `CnApi`, calls `core_info`, loads the synthetic
research template, commits the load, and renders a projection result.

The headless check is:

```sh
npm run smoke:node
```

The node check builds `core/crates/cn-wasm/pkg-node/` with
`wasm-pack --target nodejs` when that package is missing. Both `pkg/` and
`pkg-node/` are ignored build outputs.
