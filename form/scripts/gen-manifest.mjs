/**
 * D8 deploy-manifest generator (ADR-005 D8; blueprint 4.2 step 5). Reads the
 * built deploy set under --dist, emits the canonical manifest to --out, and
 * prints the manifest's own SHA-256 (over its exact stored bytes) plus a
 * reminder that the deploy set must never be committed.
 *
 * The consent_text_digest provenance field is computed with the SAME
 * consentTextDigest() the runtime bundle uses (imported from ../src/consent.ts,
 * not reimplemented) - two implementations of one digest is exactly the drift
 * this manifest exists to prevent. Requires Node >= 22 (native TypeScript
 * import; the repo pins Node 24).
 *
 * The manifest is written OUTSIDE the deploy root (D8: the manifest is not part
 * of its own file set and is not deployed). Its build_timestamp provenance
 * field legitimately varies build-to-build; that does not affect
 * reproducibility, which is defined over the deployed file bytes only.
 */
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { buildFileEntries, canonicalJson, sha256Hex } from "./manifest.ts";
import { consentTextDigest } from "../src/consent.ts";

const here = path.dirname(fileURLToPath(import.meta.url));
const formRoot = path.resolve(here, "..");

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 1) {
    const token = argv[i];
    if (token.startsWith("--")) {
      const key = token.slice(2);
      const next = argv[i + 1];
      if (next === undefined || next.startsWith("--")) {
        args[key] = "true";
      } else {
        args[key] = next;
        i += 1;
      }
    }
  }
  return args;
}

/** Extracts the resolved libsodium-wrappers version + integrity from the lockfile. */
function libsodiumIdentity() {
  const lockPath = path.join(formRoot, "package-lock.json");
  const lock = JSON.parse(readFileSync(lockPath, "utf8"));
  const pkg = lock.packages?.["node_modules/libsodium-wrappers"];
  if (pkg === undefined || typeof pkg.version !== "string") {
    throw new Error(
      "libsodium-wrappers not found in form/package-lock.json - run `npm install` first",
    );
  }
  return {
    package: "libsodium-wrappers",
    version: pkg.version,
    integrity: typeof pkg.integrity === "string" ? pkg.integrity : null,
  };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const distDir = path.resolve(args.dist ?? path.join(formRoot, "dist"));
  const outPath = path.resolve(args.out ?? path.join(formRoot, "dist.manifest.json"));

  const manifest = {
    files: buildFileEntries(distDir),
    provenance: {
      source_commit_sha: args.commit ?? "unknown",
      build_timestamp: new Date().toISOString(),
      builder: args.builder ?? "local-build",
      form_version: args["form-version"] ?? "unknown",
      key_fingerprint: args.fingerprint ?? "unknown",
      consent_text_digest: await consentTextDigest(),
      libsodium: libsodiumIdentity(),
    },
  };

  const text = canonicalJson(manifest) + "\n";
  // utf8 encoding writes no BOM.
  writeFileSync(outPath, text, { encoding: "utf8" });
  const manifestHash = sha256Hex(Buffer.from(text, "utf8"));

  console.log(`wrote ${manifest.files.length} file entries to ${outPath}`);
  console.log(`manifest_sha256: ${manifestHash}`);
  console.log(`consent_text_digest: ${manifest.provenance.consent_text_digest}`);
  console.log(`key_fingerprint: ${manifest.provenance.key_fingerprint}`);
  console.log(
    "REMINDER: the deploy set under dist/ is NOT committed - it deploys directly " +
      "to Pages. The manifest is NOT deployed; pin its hash off-repo (ops config) " +
      "and record it in DECISIONS.md at deploy time (D8).",
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
