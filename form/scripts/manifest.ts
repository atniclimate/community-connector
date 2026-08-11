/**
 * D8 canonical deploy-manifest helpers (ADR-005 D8 grammar). Pure, Node-only
 * (node:crypto / node:fs) - never imported by the browser bundle. Used by
 * gen-manifest.mjs (the orchestrator) and exercised directly by
 * manifest.test.ts.
 *
 * D8 grammar (implemented exactly, not paraphrased):
 *  - every deployable file listed with a UTF-8/NFC-normalized, forward-slash,
 *    deploy-root-relative path (no . / .. segments, no backslashes, no
 *    percent-encoding), its byte length, and SHA-256 over the file's exact bytes;
 *  - entries sorted by BYTE-WISE path comparison;
 *  - serialization: JSON, sorted keys, LF newlines, UTF-8 without BOM;
 *  - the manifest is NOT part of its own listed file set (the caller writes it
 *    outside the deploy root);
 *  - the manifest's own hash is SHA-256 over the manifest FILE's exact stored
 *    bytes (see gen-manifest.mjs), never reconstructed from fields.
 */
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";

export type ManifestFileEntry = {
  readonly path: string;
  readonly bytes: number;
  readonly sha256: string;
};

/** SHA-256 (lowercase hex) over exact bytes. */
export function sha256Hex(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

/**
 * Recursively lists every file under `root`, returning deploy-root-relative
 * paths: forward slashes only, NFC-normalized, no leading `./`. Directories are
 * descended, not listed; symlinks are not followed specially (statSync resolves
 * them - a build output has none).
 */
export function listDeployFiles(root: string): string[] {
  const out: string[] = [];
  const walk = (dir: string): void => {
    for (const name of readdirSync(dir)) {
      const full = path.join(dir, name);
      const st = statSync(full);
      if (st.isDirectory()) {
        walk(full);
      } else if (st.isFile()) {
        const rel = path.relative(root, full).split(path.sep).join("/").normalize("NFC");
        out.push(rel);
      }
    }
  };
  walk(root);
  return out;
}

/** Byte-wise (UTF-8) path comparison - the D8 entry sort order. */
export function comparePathsByBytes(a: string, b: string): number {
  return Buffer.compare(Buffer.from(a, "utf8"), Buffer.from(b, "utf8"));
}

/**
 * Builds the sorted file-entry list for every file under `root`. Each entry
 * carries the relative path, byte length, and SHA-256 of the file's exact
 * bytes; entries are sorted by byte-wise path comparison.
 */
export function buildFileEntries(root: string): ManifestFileEntry[] {
  const entries = listDeployFiles(root).map((rel): ManifestFileEntry => {
    const bytes = readFileSync(path.join(root, ...rel.split("/")));
    return { path: rel, bytes: bytes.length, sha256: sha256Hex(bytes) };
  });
  entries.sort((a, b) => comparePathsByBytes(a.path, b.path));
  return entries;
}

/** Recursively sorts object keys; array order is preserved (D8 arrays are ordered). */
function sortDeep(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(sortDeep);
  }
  if (value !== null && typeof value === "object") {
    const sorted: Record<string, unknown> = {};
    for (const key of Object.keys(value).sort()) {
      sorted[key] = sortDeep((value as Record<string, unknown>)[key]);
    }
    return sorted;
  }
  return value;
}

/**
 * Canonical JSON per D8: recursively sorted object keys (array order kept),
 * 2-space indent, LF newlines (JSON.stringify never emits CR). The caller adds
 * a trailing LF and writes UTF-8 without BOM.
 */
export function canonicalJson(value: unknown): string {
  return JSON.stringify(sortDeep(value), null, 2);
}
