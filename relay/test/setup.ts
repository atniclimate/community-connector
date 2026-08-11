import { beforeEach } from "vitest";
import { env } from "cloudflare:test";

// Deterministic clean slate. @cloudflare/vitest-pool-workers v0.21 removed the
// isolatedStorage option, so rather than relying on the pool to isolate storage
// we wipe the single test KV namespace (all prefixes: blob:, ledger:,
// ratelimit:) before every test.
beforeEach(async () => {
  let cursor: string | undefined;
  do {
    const listing = await env.INTAKE_BLOBS.list(cursor === undefined ? {} : { cursor });
    await Promise.all(listing.keys.map((key) => env.INTAKE_BLOBS.delete(key.name)));
    cursor = listing.list_complete ? undefined : listing.cursor;
  } while (cursor !== undefined);
});
