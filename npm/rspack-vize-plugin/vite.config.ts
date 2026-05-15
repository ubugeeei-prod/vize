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
    entry: [
      "src/index.ts",
      "src/loader/index.ts",
      "src/loader/style-loader.ts",
      "src/loader/scope-loader.ts",
    ],
    format: "esm",
    dts: true,
    clean: true,
    minify: true,
    sourcemap: false,
  },
});
