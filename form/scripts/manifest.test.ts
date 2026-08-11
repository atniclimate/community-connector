import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { buildFileEntries, canonicalJson, comparePathsByBytes, sha256Hex } from "./manifest";

let root = "";

beforeAll(() => {
  root = mkdtempSync(path.join(tmpdir(), "cn-form-manifest-"));
  mkdirSync(path.join(root, "assets"), { recursive: true });
  writeFileSync(path.join(root, "index.html"), "<!doctype html><title>x</title>\n", "utf8");
  writeFileSync(path.join(root, "assets", "app.js"), "console.log(1)\n", "utf8");
  writeFileSync(path.join(root, "assets", "app.css"), "body{color:#000}\n", "utf8");
});

afterAll(() => {
  if (root !== "") {
    rmSync(root, { recursive: true, force: true });
  }
});

describe("sha256Hex", () => {
  it("matches node crypto over the same bytes", () => {
    const bytes = new TextEncoder().encode("hello");
    expect(sha256Hex(bytes)).toBe(createHash("sha256").update(bytes).digest("hex"));
  });
});

describe("comparePathsByBytes", () => {
  it("orders by UTF-8 byte value (uppercase before lowercase)", () => {
    expect(comparePathsByBytes("B", "a")).toBeLessThan(0); // 0x42 < 0x61
    expect(comparePathsByBytes("assets/a", "index")).toBeLessThan(0);
    expect(comparePathsByBytes("z", "z")).toBe(0);
  });
});

describe("buildFileEntries", () => {
  it("lists every deployable file with correct length + sha256, byte-sorted, forward-slash paths", () => {
    const entries = buildFileEntries(root);
    expect(entries.map((e) => e.path)).toEqual([
      "assets/app.css",
      "assets/app.js",
      "index.html",
    ]);
    for (const e of entries) {
      expect(e.path).not.toContain("\\");
      expect(e.path.normalize("NFC")).toBe(e.path);
    }
    const css = entries.find((e) => e.path === "assets/app.css")!;
    expect(css.bytes).toBe("body{color:#000}\n".length);
    expect(css.sha256).toBe(
      createHash("sha256").update("body{color:#000}\n", "utf8").digest("hex"),
    );
  });

  it("does not list a manifest written outside the deploy root (self-exclusion)", () => {
    // The generator writes the manifest OUTSIDE root, so it is never among the
    // listed files. Verify the entry set is exactly the three deploy files.
    const entries = buildFileEntries(root);
    expect(entries).toHaveLength(3);
    expect(entries.some((e) => e.path.includes("manifest"))).toBe(false);
  });
});

describe("canonicalJson", () => {
  it("sorts object keys recursively and preserves array order", () => {
    const out = canonicalJson({ b: 1, a: { d: 2, c: 3 }, list: [{ y: 1, x: 2 }] });
    expect(out).toBe('{\n  "a": {\n    "c": 3,\n    "d": 2\n  },\n  "b": 1,\n  "list": [\n    {\n      "x": 2,\n      "y": 1\n    }\n  ]\n}');
  });

  it("uses LF newlines and no BOM when written UTF-8", () => {
    const text = canonicalJson({ files: buildFileEntries(root) }) + "\n";
    expect(text).not.toContain("\r");
    const outPath = path.join(root, "..", `m-${path.basename(root)}.json`);
    writeFileSync(outPath, text, { encoding: "utf8" });
    const raw = new Uint8Array(readFileSync(outPath)); // exact stored bytes
    rmSync(outPath, { force: true });
    expect(raw[0]).not.toBe(0xef); // no UTF-8 BOM (EF BB BF)
    expect(Array.from(raw)).not.toContain(0x0d); // no CR
  });
});
