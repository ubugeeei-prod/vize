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
      "src/validate.ts",
      "src/test-run.ts",
      "src/test-run-validate.ts",
      "src/test-run-canonical.ts",
    ],
    format: "esm",
    dts: true,
    clean: true,
  },
});
