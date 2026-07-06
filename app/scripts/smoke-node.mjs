import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { existsSync } from "node:fs";
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
const viewer = JSON.stringify({ kind: "anonymous" });
const template = JSON.stringify({
  schema_version: "0.1.0",
  template_id: "research",
  name: "Research Network",
  description: "Synthetic research network",
  kinds: [
    {
      id: "person",
      label: "Person",
      shape: "sphere",
      color_role: "kind-1",
      attributes: [{ id: "display_name", type: "text", required: true }],
    },
  ],
  edge_kinds: [
    {
      id: "knows",
      label: "Knows",
      from: ["person"],
      to: ["person"],
      directed: false,
      weighted: "forbidden",
    },
  ],
  theme: { mode: "light", roles: { "kind-1": "#224466" } },
});

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
ok("load_group_commit", api.load_group_commit(groupId, Date.now()));
ok("projection", api.projection(groupId, viewer));
