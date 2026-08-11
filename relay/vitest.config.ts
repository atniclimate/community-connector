import { cloudflareTest } from "@cloudflare/vitest-pool-workers";
import { defineConfig } from "vitest/config";

// Workers-native Vitest integration (@cloudflare/vitest-pool-workers v0.21, the
// vitest-v4 line): the pool ships as a Vite plugin (`cloudflareTest(...)`), the
// successor to the older `defineWorkersConfig`. Tests run inside a
// Miniflare-simulated Workers runtime with a local KV namespace - no real
// Cloudflare account (blueprint 5.4). This version dropped the `isolatedStorage`
// option, so test/setup.ts wipes the KV namespace before each test to keep blob
// counts and rate-limit windows from leaking across cases.
//
// Binding names and [vars] come from wrangler.toml. Secrets are not in
// wrangler.toml, so they are supplied here as miniflare bindings with
// LOCAL-ONLY test values:
//   CREDENTIAL_HASH     = SHA-256 hex of the test bearer token "cn-relay-test-bearer"
//   ADMISSION_ALLOWLIST = test fingerprints (the leading space on the second
//                         entry deliberately exercises per-entry trimming)
export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: { configPath: "./wrangler.toml" },
      miniflare: {
        bindings: {
          CREDENTIAL_HASH:
            "235c2b6e68ec3d987a3c61b2c8e9966ebdd0c7c7350df9462753f2fe7ea07192",
          ADMISSION_ALLOWLIST: "3f9a-1c02-7b41-e6d5, 0000-1111-2222-3333",
        },
      },
    }),
  ],
  test: {
    setupFiles: ["./test/setup.ts"],
  },
});
