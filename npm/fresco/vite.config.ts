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
      "src/components/index.ts",
      "src/composables/index.ts",
      "src/testing/index.ts",
    ],
    format: "esm",
    dts: true,
    clean: true,
  },
});
