import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: {
    ignorePatterns: ["dist/**"],
    options: {
      typeAware: true,
    },
  },
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  pack: {
    entry: ["src/extension.ts"],
    outDir: "dist",
    format: "cjs",
    platform: "node",
    minify: true,
    deps: {
      neverBundle: ["vscode"],
    },
  },
});
