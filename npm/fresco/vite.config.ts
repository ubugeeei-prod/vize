import { fileURLToPath } from "node:url";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite-plus";

export default defineConfig({
  plugins: [vue()],
  resolve: {
    // The external-package fixture must exercise this checkout even when a
    // shared pnpm store links the workspace name to another clean worktree.
    alias: [
      {
        find: /^@vizejs\/fresco$/u,
        replacement: fileURLToPath(new URL("./src/index.ts", import.meta.url)),
      },
    ],
  },
  test: {
    // SFCs must be compiled to their client render functions because Fresco
    // mounts them through its custom renderer, not Vue's SSR renderer.
    environment: "happy-dom",
    include: ["src/**/*.test.ts", "src/**/*.spec.ts", "examples/external-plugin/**/*.spec.ts"],
  },
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
    entry: [
      "src/index.ts",
      "src/components/index.ts",
      "src/composables/index.ts",
      "src/testing/index.ts",
    ],
    format: "esm",
    dts: { vue: true },
    plugins: [vue()],
    clean: true,
    deps: {
      neverBundle: ["vue"],
    },
  },
});
