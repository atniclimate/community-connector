import { defineConfig } from "vitest/config";
import { viteSingleFile } from "vite-plugin-singlefile";

export default defineConfig(({ mode }) => ({
  plugins: mode === "snapshot" ? [viteSingleFile()] : [],
  test: {
    environment: "node",
  },
  build: {
    target: "es2022",
    outDir: mode === "smoke" ? "dist-smoke" : "dist",
    rollupOptions: mode === "smoke" ? { input: "smoke/index.html" } : { input: "index.html" },
  },
}));
