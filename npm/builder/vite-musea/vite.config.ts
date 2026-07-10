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
      "src/vrt.ts",
      "src/cli/index.ts",
      "src/a11y/index.ts",
      "src/autogen/index.ts",
    ],
    format: "esm",
    dts: true,
    clean: false,
  },
});
