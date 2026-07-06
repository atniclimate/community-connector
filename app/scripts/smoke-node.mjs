import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../..");
const core = path.join(repo, "core");
const pkgNode = path.join(core, "crates/cn-wasm/pkg-node");
const pkgEntry = path.join(pkgNode, "cn_wasm.js");

if (!existsSync(pkgEntry)) {
  execFileSync(
    "wasm-pack",
    ["build", "crates/cn-wasm", "--target", "nodejs", "--out-dir", "pkg-node"],
    { cwd: core, stdio: "inherit" },
  );
}

const require = createRequire(import.meta.url);
const { CnApi } = require(pkgEntry);

const groupId = "00000000-0000-0000-0000-000000000010";
const viewer = JSON.stringify({
  kind: "person",
  person: "00000000-0000-0000-0000-0000000003e9",
});
const template = readFileSync(
  path.join(repo, "fixtures/templates/research-network.template.json"),
  "utf8",
);
const ops = readFileSync(path.join(repo, "fixtures/groups/research-network.ops.jsonl"), "utf8");
const minProjectedEntities = 100;

function ok(name, json) {
  const envelope = JSON.parse(json);
  if (envelope.err) {
    throw new Error(`${name} failed: ${JSON.stringify(envelope.err)}`);
  }
  console.log(`PASS ${name}`);
  return envelope.ok;
}

const api = new CnApi();
ok("core_info", api.core_info());
ok("load_group_begin", api.load_group_begin(groupId, viewer, template));
ok("load_ops_chunk", api.load_ops_chunk(groupId, ops));
ok("load_group_commit", api.load_group_commit(groupId, Date.now()));
const projection = ok("projection", api.projection(groupId, viewer));
if (!Array.isArray(projection.entities) || projection.entities.length <= minProjectedEntities) {
  throw new Error(`projection entity count too low: ${projection.entities?.length ?? "missing"}`);
}
console.log(`PASS projection_entity_count ${projection.entities.length}`);
