// Size budget check (AGENTS.md I8): the single-file snapshot stays under 5MB,
// enforced here in the build script, never by eyeball.
import { statSync } from "node:fs";

const path = process.argv[2] ?? "dist/index.html";
const BUDGET_BYTES = 5 * 1024 * 1024;
const size = statSync(path).size;
const mb = (size / 1024 / 1024).toFixed(2);

if (size >= BUDGET_BYTES) {
  console.error(`SIZE BUDGET FAIL: ${path} is ${mb}MB (budget 5.00MB)`);
  process.exit(1);
}
console.log(`size ok: ${path} is ${mb}MB (budget 5.00MB)`);
