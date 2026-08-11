import type { Env } from "../env.js";
import { RATELIMIT_PREFIX } from "./kv.js";
import type { RelayConfig } from "./config.js";

const WINDOW_SECONDS = 60;
/** TTL slightly over one window so a bucket self-drains after it stops mattering. */
const WINDOW_TTL_SECONDS = WINDOW_SECONDS + 10;

/**
 * KV-backed fixed-window per-IP rate limiter (ADR-005 D6 "NAT-safe";
 * deliberately APPROXIMATE - D6: KV has no atomic counter, and "the real abuse
 * backstops are the size cap, the TTL, the billing ceiling, and the review
 * gate - not per-IP precision"). The read-modify-write can lose increments
 * under burst; that under-count is accepted, not a bug to engineer around.
 *
 * Key shape ratelimit:<ip>:<window-bucket>; the count is compared against the
 * generous per-IP limit and, when admitted, incremented. Over-limit requests
 * are refused BEFORE incrementing, so a blocked caller does not push the bucket
 * further.
 */
export async function checkRateLimit(
  env: Env,
  ip: string,
  config: RelayConfig,
): Promise<{ allowed: boolean }> {
  const bucket = Math.floor(Date.now() / 1000 / WINDOW_SECONDS);
  const key = `${RATELIMIT_PREFIX}${ip}:${bucket}`;

  const raw = await env.INTAKE_BLOBS.get(key);
  const count = raw === null ? 0 : Number.parseInt(raw, 10);
  const current = Number.isNaN(count) ? 0 : count;

  if (current >= config.rateLimitPerIpPerMinute) {
    return { allowed: false };
  }

  await env.INTAKE_BLOBS.put(key, String(current + 1), {
    expirationTtl: WINDOW_TTL_SECONDS,
  });
  return { allowed: true };
}
