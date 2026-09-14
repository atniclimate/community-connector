// Build-artifact gates on the single-file snapshot, enforced in the build
// script, never by eyeball:
//   1. Size budget (AGENTS.md I8): the snapshot stays under 5MB.
//   2. DEV-only hooks stay out of the built bundle: `window.__cn_state_snapshot`
//      (main.ts) and `window.__cn_visible_labels` (viz/index.ts) are gated on
//      `import.meta.env.DEV`, which Vite derives from NODE_ENV rather than
//      `--mode`; this belt-and-suspenders grep fails the build if an ambient
//      NODE_ENV ever lets either name into any built .html/.js file.
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

const target = process.argv[2] ?? "dist/index.html";
const BUDGET_BYTES = 5 * 1024 * 1024;
const FORBIDDEN_STRINGS = ["__cn_visible_labels", "__cn_state_snapshot"];
const SCANNED_EXTENSIONS = new Set([".html", ".js", ".mjs"]);

const size = statSync(target).size;
const mb = (size / 1024 / 1024).toFixed(2);

if (size >= BUDGET_BYTES) {
  console.error(`SIZE BUDGET FAIL: ${target} is ${mb}MB (budget 5.00MB)`);
  process.exit(1);
}
console.log(`size ok: ${target} is ${mb}MB (budget 5.00MB)`);

function builtFiles(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      return builtFiles(full);
    }
    return SCANNED_EXTENSIONS.has(path.extname(entry.name)) ? [full] : [];
  });
}

const distDir = path.dirname(path.resolve(target));
const leaks = builtFiles(distDir).flatMap((file) => {
  const text = readFileSync(file, "utf8");
  return FORBIDDEN_STRINGS.filter((needle) => text.includes(needle)).map((needle) => `${needle} in ${file}`);
});
if (leaks.length > 0) {
  console.error(`DEV HOOK LEAK FAIL: ${leaks.join("; ")}`);
  process.exit(1);
}
console.log(`dev hooks absent: ${FORBIDDEN_STRINGS.join(", ")} not in ${distDir}`);
