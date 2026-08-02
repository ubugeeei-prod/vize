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
    entry: ["src/index.ts", "src/lint/index.ts", "src/runtime/server/dev-stylesheet-links.ts"],
    copy: [{ from: "src/nuxt2-entry.cjs", to: "dist" }],
    format: "esm",
    dts: true,
    clean: true,
    deps: {
      neverBundle: [
        "@vizejs/nuxt-lint-config",
        "@vizejs/vite-plugin",
        "@vizejs/vite-plugin-musea",
        "nitropack/runtime",
        "oxlint-plugin-vize",
        "#vizejs/nuxt/dev-stylesheet-links-config",
        "vize",
      ],
    },
  },
});
