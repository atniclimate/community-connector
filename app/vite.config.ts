import { defineConfig } from "vitest/config";
import { viteSingleFile } from "vite-plugin-singlefile";
import { createReadStream, existsSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const FIXTURE_PREFIX = "/fixtures/";
const NOT_FOUND_STATUS = 404;
const here = path.dirname(fileURLToPath(import.meta.url));

function contentType(file: string): string {
  if (file.endsWith(".json")) {
    return "application/json";
  }
  if (file.endsWith(".jsonl")) {
    return "application/jsonl";
  }
  return "text/plain";
}

export default defineConfig(({ mode }) => ({
  plugins: [
    ...(mode === "snapshot" ? [viteSingleFile()] : []),
    {
      name: "cn-dev-fixtures",
      configureServer(server) {
        server.middlewares.use((req, res, next) => {
          if (req.url === undefined || !req.url.startsWith(FIXTURE_PREFIX)) {
            next();
            return;
          }
          const root = path.resolve(here, "..", "fixtures");
          const relative = req.url.split("?")[0]?.slice(FIXTURE_PREFIX.length) ?? "";
          const file = path.resolve(root, relative);
          if (!file.startsWith(root) || !existsSync(file) || !statSync(file).isFile()) {
            res.statusCode = NOT_FOUND_STATUS;
            res.end("fixture not found");
            return;
          }
          res.setHeader("Content-Type", contentType(file));
          createReadStream(file).pipe(res);
        });
      },
    },
  ],
  test: {
    environment: "node",
  },
  server: {
    fs: {
      allow: [path.resolve(here, "..")],
    },
  },
  build: {
    target: "es2022",
    outDir: mode === "smoke" ? "dist-smoke" : "dist",
    rollupOptions: mode === "smoke" ? { input: "smoke/index.html" } : { input: "index.html" },
  },
}));
