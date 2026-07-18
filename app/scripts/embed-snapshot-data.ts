// P2.3 snapshot data embed (D-044.5, D-046.4; docs/design/snapshot-viewer-scope.md).
//
// Runs as a Vite plugin in snapshot mode, after vite-plugin-singlefile has
// written dist/index.html (the data-free shell). For each bakeable fixture it:
//   1. computes a REAL cn-perm projection through cn-api export_snapshot via
//      the node wasm build (pkg-node) - never a hand-rolled one (I2);
//   2. derives the resolved theme with the same deriveTheme the app uses;
//   3. assembles the snapshot data envelope, validates it against
//      schemas/snapshot-envelope.schema.json (loud on failure, I3/I12);
//   4. inlines the worker chunk as a blob-URL bootstrap so the artifact is
//      truly single-file (D-046.4), injects the envelope as an inline JSON
//      script tag, and writes one artifact per fixture (D-044.5):
//      dist/snapshot.<fixture-id>.html.
// A machine-readable report lands at dist/snapshot-embed-report.json (I12).
//
// Scope guard: the generator bakes ONLY the anonymous viewer scope; any
// fixture whose anonymous projection is empty is skipped loudly (or fails the
// build when requested explicitly) so a snapshot can never quietly ship an
// empty viewer. Rationale and the acceptance predicate live in
// docs/design/snapshot-viewer-scope.md (P2.5).
//
// Knobs (build-time only):
//   CN_SNAPSHOT_FIXTURE       comma-separated fixture ids (explicit mode:
//                             an empty anonymous projection is a hard error).
//   CN_SNAPSHOT_FIXTURES_ROOT alternate fixtures root (testing only).

import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { readFile, readdir, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import Ajv2020 from "ajv/dist/2020.js";
import type { ValidateFunction } from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import type { Plugin } from "vite";
import { deriveTheme } from "../src/theme/derive";
import type { GroupTemplateDto, Theme } from "../src/theme/tokens";

const SNAPSHOT_ENVELOPE_SCHEMA_ID =
  "https://community-navigator.example.test/schemas/snapshot-envelope/0.1.0";
const ANONYMOUS_VIEWER_JSON = JSON.stringify({ kind: "anonymous" });
// Fixed clock for the load commit so the embed step is deterministic (I8).
const FIXED_BUILD_CLOCK_MS = 1_000_000_000_000;
const WORKER_FILE_PATTERN = /worker-[A-Za-z0-9_-]+\.js/g;
const MAX_PRINTED_SCHEMA_ERRORS = 10;
const OPS_SUFFIX = ".ops.jsonl";

type CnApiInstance = {
  load_group_begin(groupId: string, viewerJson: string, templateJson: string): string;
  load_ops_chunk(groupId: string, opsJsonl: string): string;
  load_group_commit(groupId: string, nowMs: number): string;
  export_snapshot(groupId: string, viewerJson: string, optionsJson: string): string;
};

type CnApiConstructor = new () => CnApiInstance;

type ProjectionDto = {
  readonly group_id: string;
  readonly viewer_fingerprint: string;
  readonly revision: number;
  readonly entities: readonly unknown[];
  readonly edges: readonly unknown[];
  readonly stories: readonly unknown[];
};

type ExportSnapshotDto = {
  readonly boundary_version: string;
  readonly schema_version: string;
  readonly template: Record<string, unknown>;
  readonly projection: ProjectionDto;
};

type SnapshotEnvelope = {
  readonly schema_version: string;
  readonly viewer_scope: "anonymous";
  readonly viewer_context: { readonly kind: "anonymous" };
  readonly export: ExportSnapshotDto;
  readonly theme: Theme;
};

type EmbedRow = {
  readonly fixture: string;
  readonly group_id: string;
  readonly artifact: string;
  readonly viewer_scope: "anonymous";
  readonly entities: number;
  readonly edges: number;
  readonly stories: number;
  readonly envelope_bytes: number;
  readonly artifact_bytes: number;
  readonly worker_scripts_inlined: readonly string[];
  readonly validated: true;
};

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function unwrapWasmEnvelope(step: string, json: string): unknown {
  let parsed: unknown;
  try {
    parsed = JSON.parse(json);
  } catch (error) {
    throw new Error(`cn-wasm ${step} returned invalid JSON: ${errorMessage(error)}`);
  }
  if (parsed === null || typeof parsed !== "object") {
    throw new Error(`cn-wasm ${step} returned a non-object envelope`);
  }
  const envelope = parsed as { ok?: unknown; err?: unknown };
  if (envelope.err !== undefined) {
    throw new Error(`cn-wasm ${step} failed: ${JSON.stringify(envelope.err)}`);
  }
  if (!("ok" in envelope)) {
    throw new Error(`cn-wasm ${step} returned an envelope without ok or err`);
  }
  return envelope.ok;
}

function assertExportSnapshot(value: unknown, fixtureId: string): ExportSnapshotDto {
  const exported = value as Partial<ExportSnapshotDto> | null;
  if (
    exported === null ||
    typeof exported !== "object" ||
    typeof exported.schema_version !== "string" ||
    typeof exported.template !== "object" ||
    exported.template === null ||
    typeof exported.projection !== "object" ||
    exported.projection === null ||
    typeof exported.projection.group_id !== "string" ||
    !Array.isArray(exported.projection.entities) ||
    !Array.isArray(exported.projection.edges) ||
    !Array.isArray(exported.projection.stories)
  ) {
    throw new Error(`fixture ${fixtureId}: cn-api export_snapshot returned an unexpected shape`);
  }
  return exported as ExportSnapshotDto;
}

function loadCnApi(repoRoot: string): CnApiConstructor {
  const core = path.join(repoRoot, "core");
  const pkgEntry = path.join(core, "crates", "cn-wasm", "pkg-node", "cn_wasm.js");
  if (!existsSync(pkgEntry)) {
    // Mirrors app/scripts/smoke-node.mjs: the node wasm build is created
    // lazily when absent so the snapshot build stays one command.
    execFileSync(
      "wasm-pack",
      ["build", "crates/cn-wasm", "--target", "nodejs", "--out-dir", "pkg-node"],
      { cwd: core, stdio: "inherit" },
    );
  }
  const requireFromPkg = createRequire(pkgEntry);
  const module = requireFromPkg(pkgEntry) as { CnApi?: CnApiConstructor };
  if (module.CnApi === undefined) {
    throw new Error(`node wasm package at ${pkgEntry} does not export CnApi`);
  }
  return module.CnApi;
}

function groupIdFromOps(opsJsonl: string, fixtureId: string): string {
  const firstLine = opsJsonl.split("\n").find((line) => line.trim() !== "");
  if (firstLine === undefined) {
    throw new Error(`fixture ${fixtureId}: ops log is empty`);
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(firstLine);
  } catch (error) {
    throw new Error(`fixture ${fixtureId}: first op line is invalid JSON: ${errorMessage(error)}`);
  }
  const groupId = (parsed as { group_id?: unknown }).group_id;
  if (typeof groupId !== "string") {
    throw new Error(`fixture ${fixtureId}: first op line carries no group_id`);
  }
  return groupId;
}

async function computeAnonymousExport(
  CnApi: CnApiConstructor,
  fixturesRoot: string,
  fixtureId: string,
): Promise<ExportSnapshotDto> {
  const templateJson = await readFile(
    path.join(fixturesRoot, "templates", `${fixtureId}.template.json`),
    "utf8",
  );
  const opsJsonl = await readFile(
    path.join(fixturesRoot, "groups", `${fixtureId}${OPS_SUFFIX}`),
    "utf8",
  );
  const groupId = groupIdFromOps(opsJsonl, fixtureId);
  const api = new CnApi();
  unwrapWasmEnvelope(
    "load_group_begin",
    api.load_group_begin(groupId, ANONYMOUS_VIEWER_JSON, templateJson),
  );
  unwrapWasmEnvelope("load_ops_chunk", api.load_ops_chunk(groupId, opsJsonl));
  unwrapWasmEnvelope("load_group_commit", api.load_group_commit(groupId, FIXED_BUILD_CLOCK_MS));
  const exported = unwrapWasmEnvelope(
    "export_snapshot",
    api.export_snapshot(groupId, ANONYMOUS_VIEWER_JSON, "{}"),
  );
  return assertExportSnapshot(exported, fixtureId);
}

function buildEnvelope(exported: ExportSnapshotDto): SnapshotEnvelope {
  // Same derivation path the app boot uses on the same serialized template,
  // so the baked theme and the rendered theme cannot drift.
  const derived = deriveTheme(exported.template as unknown as GroupTemplateDto);
  return {
    schema_version: exported.schema_version,
    viewer_scope: "anonymous",
    viewer_context: { kind: "anonymous" },
    export: exported,
    theme: derived.theme,
  };
}

async function compileEnvelopeValidator(schemasDir: string): Promise<ValidateFunction> {
  const ajv = new Ajv2020({ allErrors: true, strict: false });
  addFormats(ajv);
  const schemaFiles = (await readdir(schemasDir))
    .filter((fileName) => fileName.endsWith(".schema.json"))
    .sort();
  for (const fileName of schemaFiles) {
    const raw = await readFile(path.join(schemasDir, fileName), "utf8");
    ajv.addSchema(JSON.parse(raw) as Record<string, unknown>);
  }
  return ajv.compile({ $ref: SNAPSHOT_ENVELOPE_SCHEMA_ID });
}

function validateEnvelope(
  validate: ValidateFunction,
  envelope: SnapshotEnvelope,
  fixtureId: string,
): void {
  if (validate(envelope)) {
    return;
  }
  const errors = (validate.errors ?? [])
    .slice(0, MAX_PRINTED_SCHEMA_ERRORS)
    .map((error) => `  ${error.instancePath || "/"} ${error.message ?? "invalid"}`);
  throw new Error(
    `fixture ${fixtureId}: snapshot envelope failed schema validation:\n${errors.join("\n")}`,
  );
}

// Escapes text for safe embedding inside inline <script> content. Valid in
// both JSON and JS string literals, and prevents </script> breakout.
function escapeInlineScript(text: string): string {
  return text.replace(/</g, "\\u003c");
}

async function collectWorkerSources(
  shellHtml: string,
  distDir: string,
): Promise<ReadonlyMap<string, string>> {
  const names = [...new Set(shellHtml.match(WORKER_FILE_PATTERN) ?? [])].sort();
  const sources = new Map<string, string>();
  for (const name of names) {
    const candidates = [path.join(distDir, name), path.join(distDir, "assets", name)];
    const found = candidates.find((candidate) => existsSync(candidate));
    if (found === undefined) {
      throw new Error(
        `snapshot shell references worker script ${name} but it was not emitted to ${distDir}`,
      );
    }
    sources.set(name, await readFile(found, "utf8"));
  }
  return sources;
}

// Classic script that intercepts Worker construction for the emitted worker
// chunk(s) and substitutes a blob URL built from the inlined source, so the
// artifact needs no sibling files (D-046.4). The worker chunk is a
// self-contained IIFE with the wasm already inlined as a data: URI.
function workerInlineScript(sources: ReadonlyMap<string, string>): string {
  const entries = [...sources.entries()]
    .map(([name, source]) => `${JSON.stringify(name)}:${escapeInlineScript(JSON.stringify(source))}`)
    .join(",");
  return [
    '<script id="cn-snapshot-worker-inline">',
    "(function () {",
    '  "use strict";',
    `  var sources = {${entries}};`,
    "  var urls = {};",
    "  var NativeWorker = window.Worker;",
    "  function inlineUrl(name) {",
    "    if (!Object.prototype.hasOwnProperty.call(urls, name)) {",
    "      urls[name] = URL.createObjectURL(",
    '        new Blob([sources[name]], { type: "text/javascript" })',
    "      );",
    "    }",
    "    return urls[name];",
    "  }",
    "  function matchName(href) {",
    "    var names = Object.keys(sources);",
    "    for (var i = 0; i < names.length; i += 1) {",
    "      if (href.indexOf(names[i]) !== -1) { return names[i]; }",
    "    }",
    "    return null;",
    "  }",
    "  function CnSnapshotWorker(scriptUrl, options) {",
    "    var name = matchName(String(scriptUrl));",
    "    if (name !== null) { return new NativeWorker(inlineUrl(name), options); }",
    "    return new NativeWorker(scriptUrl, options);",
    "  }",
    "  CnSnapshotWorker.prototype = NativeWorker.prototype;",
    "  window.Worker = CnSnapshotWorker;",
    "}());",
    "</scr" + "ipt>",
  ].join("\n");
}

function envelopeScriptTag(envelopeJson: string): string {
  return (
    '<script type="application/json" id="cn-snapshot-envelope">' +
    escapeInlineScript(envelopeJson) +
    "</scr" +
    "ipt>"
  );
}

function injectBeforeHeadClose(html: string, fragment: string): string {
  const marker = "</head>";
  const index = html.indexOf(marker);
  if (index === -1) {
    throw new Error("snapshot shell has no </head> to inject into");
  }
  return html.slice(0, index) + fragment + "\n" + html.slice(index);
}

function formatBytes(bytes: number): string {
  return `${(bytes / 1024 / 1024).toFixed(2)}MB`;
}

function parseRequestedFixtures(raw: string | undefined): readonly string[] | null {
  if (raw === undefined || raw.trim() === "") {
    return null;
  }
  return raw
    .split(",")
    .map((id) => id.trim())
    .filter((id) => id !== "");
}

async function discoverFixtureIds(fixturesRoot: string): Promise<readonly string[]> {
  const entries = await readdir(path.join(fixturesRoot, "groups"));
  return entries
    .filter((fileName) => fileName.endsWith(OPS_SUFFIX))
    .map((fileName) => fileName.slice(0, -OPS_SUFFIX.length))
    .sort();
}

function emptyProjectionMessage(fixtureId: string): string {
  return (
    `fixture ${fixtureId}: the anonymous projection is empty - a snapshot built from it ` +
    "would ship an empty viewer. The fixture needs a public layer " +
    "(node app/scripts/generate-demo-ops.mjs --public-layer) or a bakeable scope decision; " +
    "see docs/design/snapshot-viewer-scope.md"
  );
}

async function embedFixture(args: {
  readonly CnApi: CnApiConstructor;
  readonly fixturesRoot: string;
  readonly fixtureId: string;
  readonly shellHtml: string;
  readonly workerFragment: string;
  readonly workerNames: readonly string[];
  readonly validate: ValidateFunction;
  readonly distDir: string;
}): Promise<EmbedRow> {
  const exported = await computeAnonymousExport(args.CnApi, args.fixturesRoot, args.fixtureId);
  const envelope = buildEnvelope(exported);
  validateEnvelope(args.validate, envelope, args.fixtureId);
  const envelopeJson = JSON.stringify(envelope);
  const fragment = args.workerFragment + "\n" + envelopeScriptTag(envelopeJson);
  const artifactHtml = injectBeforeHeadClose(args.shellHtml, fragment);
  const artifactName = `snapshot.${args.fixtureId}.html`;
  await writeFile(path.join(args.distDir, artifactName), artifactHtml);
  return {
    fixture: args.fixtureId,
    group_id: exported.projection.group_id,
    artifact: artifactName,
    viewer_scope: "anonymous",
    entities: exported.projection.entities.length,
    edges: exported.projection.edges.length,
    stories: exported.projection.stories.length,
    envelope_bytes: Buffer.byteLength(envelopeJson),
    artifact_bytes: Buffer.byteLength(artifactHtml),
    worker_scripts_inlined: args.workerNames,
    validated: true,
  };
}

export async function embedSnapshotData(appRoot: string, outDirName: string): Promise<void> {
  const repoRoot = path.resolve(appRoot, "..");
  const distDir = path.resolve(appRoot, outDirName);
  const shellPath = path.join(distDir, "index.html");
  if (!existsSync(shellPath)) {
    throw new Error(`snapshot shell not found at ${shellPath} - did the snapshot build run?`);
  }
  const shellHtml = await readFile(shellPath, "utf8");
  const workers = await collectWorkerSources(shellHtml, distDir);
  const workerNames = [...workers.keys()];
  const workerFragment = workers.size > 0 ? workerInlineScript(workers) : "";
  const fixturesRoot = process.env.CN_SNAPSHOT_FIXTURES_ROOT ?? path.join(repoRoot, "fixtures");
  const requested = parseRequestedFixtures(process.env.CN_SNAPSHOT_FIXTURE);
  const fixtureIds = requested ?? (await discoverFixtureIds(fixturesRoot));
  const validate = await compileEnvelopeValidator(path.join(repoRoot, "schemas"));
  const CnApi = loadCnApi(repoRoot);
  const rows: EmbedRow[] = [];
  const warnings: string[] = [];
  if (workers.size === 0) {
    warnings.push("shell references no worker script - nothing was inlined (D-046.4)");
  }
  for (const fixtureId of fixtureIds) {
    const probe = await computeAnonymousExport(CnApi, fixturesRoot, fixtureId);
    if (probe.projection.entities.length === 0) {
      if (requested !== null) {
        throw new Error(emptyProjectionMessage(fixtureId));
      }
      warnings.push(`${emptyProjectionMessage(fixtureId)} (fixture skipped)`);
      console.log(`SKIP snapshot embed ${fixtureId} (empty anonymous projection)`);
      continue;
    }
    const row = await embedFixture({
      CnApi,
      fixturesRoot,
      fixtureId,
      shellHtml,
      workerFragment,
      workerNames,
      validate,
      distDir,
    });
    rows.push(row);
    console.log(
      `PASS snapshot embed ${row.fixture} -> ${row.artifact} ` +
        `(entities ${row.entities}, edges ${row.edges}, stories ${row.stories}, ` +
        `envelope ${formatBytes(row.envelope_bytes)}, artifact ${formatBytes(row.artifact_bytes)})`,
    );
  }
  if (rows.length === 0) {
    throw new Error(
      "snapshot embed produced no artifacts - every candidate fixture had an empty anonymous " +
        "projection, so the snapshot would ship an empty viewer. Regenerate fixtures with " +
        "node app/scripts/generate-demo-ops.mjs --public-layer, or bake a different fixture " +
        "(docs/design/snapshot-viewer-scope.md).",
    );
  }
  const report = {
    schema_version: "0.1.0",
    generated_by: "app/scripts/embed-snapshot-data.ts (P2.3)",
    viewer_scope: "anonymous",
    fixtures: rows,
    warnings,
  };
  await writeFile(
    path.join(distDir, "snapshot-embed-report.json"),
    JSON.stringify(report, null, 2) + "\n",
  );
  for (const warning of warnings) {
    console.warn(`WARN ${warning}`);
  }
}

/** Vite plugin wrapper: runs the embed after the snapshot shell is written. */
export function cnSnapshotEmbedPlugin(): Plugin {
  let appRoot = "";
  let outDirName = "dist";
  return {
    name: "cn-snapshot-embed",
    apply: "build",
    enforce: "post",
    configResolved(config) {
      appRoot = config.root;
      outDirName = config.build.outDir;
    },
    async closeBundle() {
      await embedSnapshotData(appRoot, outDirName);
    },
  };
}
