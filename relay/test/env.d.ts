// Type the test-runtime `env` (from "cloudflare:test", which types it as
// `Cloudflare.Env`) with the Worker's own Env shape, so `env.INTAKE_BLOBS`, the
// vars, and the secrets are all typed in tests. Vars/KV come from wrangler.toml;
// secrets come from the miniflare bindings in vitest.config.ts. The
// `cloudflare:test` module declarations themselves are pulled in via the
// tsconfig `types` entry "@cloudflare/vitest-pool-workers/types".
import type { Env as RelayEnv } from "../src/env.js";

declare global {
  // Declaration-merge the (empty) Cloudflare.Env from @cloudflare/workers-types
  // so it carries every relay binding.
  namespace Cloudflare {
    interface Env extends RelayEnv {}
  }
}
