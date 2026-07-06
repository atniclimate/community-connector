import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";

export default defineConfig(({ mode }) => ({
  plugins: mode === "snapshot" ? [viteSingleFile()] : [],
  build: { target: "es2022" },
}));
