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
    entry: ["src/index.ts", "src/runtime/server/dev-stylesheet-links.ts"],
    format: "esm",
    dts: true,
    clean: true,
    deps: {
      neverBundle: [
        "@vizejs/vite-plugin",
        "@vizejs/vite-plugin-musea",
        "@vizejs/musea-nuxt",
        "nitropack/runtime",
        "#vizejs/nuxt/dev-stylesheet-links-config",
        "vize",
      ],
    },
  },
});
