/**
 * Write the off-repo puller inputs for the automated form-to-graph rehearsal
 * (blueprint docs/blueprints/intake-relay.md section 6.3; ADR-005 D8): a minimal
 * ceremony-pinned D8 manifest FILE, the versioned `cn intake pull` config that
 * pins that manifest by hash, and the bearer-credential file.
 *
 * Centralized in Node (portable across the .ps1 and .sh entry points) so the
 * manifest-hash pin is computed in exactly one place. Everything written here
 * lands in a system temp dir the e2e cleans up; nothing is committed.
 *
 * Bundle-check posture (a step-9 design point): this points `pages_origin` at a
 * closed local port so the puller's D8 verification exercises the real
 * `verify_bundle` path - it reads and hash-pins the local manifest, matches the
 * embedded key fingerprint, then finds the origin unreachable on the first fetch
 * and returns `Degraded` (proceeds on key-pin only, ADR-005 D8). The `Verified`
 * path (against a served real `form/dist`) is the manual browser runbook's job.
 * So the manifest's per-file bytes/sha256 are placeholders: they are never
 * fetched or compared on the degraded path, but at least one file entry must
 * exist so the fetch loop runs and trips the unreachable-origin branch.
 *
 * Usage:
 *   node scripts/e2e/prepare-pull-config.mjs \
 *     --keydir <dir> --fingerprint <fp> --relay <origin> --pages <closed origin> \
 *     --digest <64-hex> --token <bearer> \
 *     --manifest <out manifest.json> --config <out config.json> --credential <out token file>
 */
import { writeFileSync } from "node:fs";
import { createHash } from "node:crypto";

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 1) {
    const flag = argv[i];
    if (!flag.startsWith("--")) throw new Error(`unexpected token '${flag}'`);
    const value = argv[i + 1];
    if (value === undefined) throw new Error(`${flag} needs a value`);
    args[flag.slice(2)] = value;
    i += 1;
  }
  for (const required of [
    "keydir", "fingerprint", "relay", "pages", "digest", "token",
    "manifest", "config", "credential",
  ]) {
    if (!args[required]) throw new Error(`--${required} is required`);
  }
  return args;
}

function sha256Hex(buf) {
  return createHash("sha256").update(buf).digest("hex");
}

function main() {
  const args = parseArgs(process.argv.slice(2));

  // Minimal D8 manifest. `provenance.key_fingerprint` must equal the pinned key
  // fingerprint (verify_bundle step 4) or the puller halts Failed before any
  // fetch. The single file entry is placeholder-hashed (never fetched on the
  // degraded path); its presence is what makes the fetch loop reach the
  // unreachable origin.
  const placeholder = Buffer.from("<!doctype html><title>e2e placeholder</title>\n", "utf8");
  const manifest = {
    schema_version: "0.1.0",
    files: [
      { path: "index.html", bytes: placeholder.byteLength, sha256: sha256Hex(placeholder) },
    ],
    provenance: {
      key_fingerprint: args.fingerprint,
      commit_sha: "e2e-synthetic-rehearsal",
      form_version: "e2e-remote-rehearsal",
      builder: "scripts/e2e/prepare-pull-config.mjs",
    },
  };
  // Serialize once and pin the hash over those EXACT bytes.
  const manifestBytes = Buffer.from(JSON.stringify(manifest, null, 2), "utf8");
  writeFileSync(args.manifest, manifestBytes);
  const manifestHash = sha256Hex(manifestBytes);

  const config = {
    config_version: "0.1.0",
    key_dir: args.keydir,
    key_fingerprint_pin: args.fingerprint,
    relay_origin: args.relay,
    credential_path: args.credential,
    pages_origin: args.pages,
    manifest_path: args.manifest,
    manifest_pin: {
      manifest_hash: manifestHash,
      commit_sha: "e2e-synthetic-rehearsal",
      pinned_at: new Date().toISOString(),
      pinned_by: "e2e-remote-intake",
    },
    known_consent_digests: [args.digest],
    blob_ttl_seconds: 86400,
    max_pull_interval_seconds: 3600,
  };
  writeFileSync(args.config, JSON.stringify(config, null, 2));

  // The bearer credential file (plain text; the puller trims trailing
  // whitespace). No trailing newline needed, but harmless if present.
  writeFileSync(args.credential, args.token);

  process.stdout.write(
    JSON.stringify({ manifest_hash: manifestHash, config: args.config }) + "\n",
  );
}

try {
  main();
} catch (err) {
  process.stderr.write(`prepare-pull-config: error: ${err.message ?? err}\n`);
  process.exit(1);
}
