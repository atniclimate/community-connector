/**
 * Vite config for the remote intake form (relay blueprint step 6; ADR-005
 * D2/D3/D8). This is a SEPARATE Vite project from app/ - it deploys to
 * Cloudflare Pages, not the app build. It shares no runtime code with app/.
 *
 * Build-time configuration is injected as compile-time constants via `define`
 * (the same mechanism app/vite.config.ts uses for __CN_SNAPSHOT_MODE__). This
 * repo has no .env* files anywhere and D8 requires a dependency-closed build
 * with no runtime fetches, so a committed default + `define` (overridable by
 * the build script through CN_FORM_* env vars), not runtime environment, is
 * the right mechanism. See src/config.ts for the consuming side.
 *
 * Determinism (I8, D8 reproducible build): sourcemaps are disabled (they would
 * leak build-machine absolute paths and add an unpinned file); no build
 * timestamp is injected into the bundled bytes (a timestamp belongs only in
 * the manifest's own provenance, which is legitimately allowed to vary and is
 * NOT part of the deployed file set). Vite's content-hashed asset filenames
 * are deterministic for a given source commit + lockfile.
 *
 * Output shape (deliberate, see DECISIONS.md): multi-file
 * (index.html + assets/*.js + assets/*.css), NOT vite-plugin-singlefile. The
 * blueprint CSP `script-src 'self'; style-src 'self'` cleanly admits external
 * same-origin assets; a fully-inlined single file would instead need inline
 * hashes or 'unsafe-inline'. Multi-file keeps the exact blueprint CSP and
 * matches D8's multi-file manifest model.
 */
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

const here = path.dirname(fileURLToPath(import.meta.url));

// TEST-ONLY committed defaults: the synthetic keypair from
// fixtures/crypto/sealed-box-vectors.json (its `note` says it is NEVER used
// operationally). ONLY the public key + fingerprint are copied here; the
// paired secret key is never embedded anywhere in form/. The real facilitator
// key is produced by the (out-of-scope) keygen ceremony and passed in via
// CN_FORM_* env at build time.
const DEFAULT_PUBLIC_KEY_HEX =
  "205fd0dce7b409a3231b86dad10f6e3a276bb2e0838bc605501131f96b117a71";
const DEFAULT_KEY_FINGERPRINT = "efdf-7ce7-69fa-feeb-7512-a100-6450-45c6";
// Standard local `wrangler dev` port. Never assume step 5's deployed URL.
const DEFAULT_RELAY_ORIGIN = "http://localhost:8787";
// Distinct from the in-app path's "in-app-draft-2026-07-24" (provenance).
const DEFAULT_FORM_VERSION = "remote-draft-2026-08-11";
// Default template baked in for local dev + tests only.
const DEFAULT_TEMPLATE_PATH = path.resolve(
  here,
  "..",
  "fixtures",
  "templates",
  "research-network.template.json",
);

const publicKeyHex = process.env["CN_FORM_PUBLIC_KEY_HEX"] ?? DEFAULT_PUBLIC_KEY_HEX;
const keyFingerprint = process.env["CN_FORM_KEY_FINGERPRINT"] ?? DEFAULT_KEY_FINGERPRINT;
const relayOrigin = process.env["CN_FORM_RELAY_ORIGIN"] ?? DEFAULT_RELAY_ORIGIN;
const formVersion = process.env["CN_FORM_VERSION"] ?? DEFAULT_FORM_VERSION;
const templatePath = process.env["CN_FORM_TEMPLATE_PATH"] ?? DEFAULT_TEMPLATE_PATH;

// Raw file bytes: deterministic given the same source file. config.ts parses
// this at load time (the template drives the R2 field widgets, blueprint 4.1).
const templateJson = readFileSync(templatePath, "utf8");

export default defineConfig({
  define: {
    __CN_FORM_PUBLIC_KEY_HEX__: JSON.stringify(publicKeyHex),
    __CN_FORM_KEY_FINGERPRINT__: JSON.stringify(keyFingerprint),
    __CN_FORM_RELAY_ORIGIN__: JSON.stringify(relayOrigin),
    __CN_FORM_VERSION__: JSON.stringify(formVersion),
    __CN_FORM_TEMPLATE_JSON__: JSON.stringify(templateJson),
  },
  plugins: [
    {
      // Substitute the CSP connect-src origin (GitHub Pages cannot set HTTP
      // response headers, so the CSP ships as a <meta> tag; the relay origin
      // is the one allowed connect destination).
      name: "cn-form-csp-relay-origin",
      transformIndexHtml(html: string): string {
        return html.replaceAll("%CN_RELAY_ORIGIN%", relayOrigin);
      },
    },
  ],
  build: {
    target: "es2022",
    outDir: "dist",
    emptyOutDir: true,
    sourcemap: false,
    // No inline modulepreload polyfill script - keeps script-src 'self' clean
    // (no inline <script> in the built index.html).
    modulePreload: false,
    assetsInlineLimit: 0,
    cssCodeSplit: false,
  },
  test: {
    environment: "node",
  },
});
