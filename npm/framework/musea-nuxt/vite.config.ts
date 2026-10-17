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
    // One graph keeps direct exports and virtual modules on the same mock context.
    entry: {
      index: "src/index.ts",
      "auto-imports": "src/auto-imports.ts",
      context: "src/context.ts",
      "mocks/components": "src/mocks/components.ts",
    },
    format: "esm",
    dts: true,
    clean: true,
  },
});
