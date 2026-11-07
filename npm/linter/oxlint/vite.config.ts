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
    entry: ["src/index.ts", "src/cli.ts", "src/eslint.ts"],
    format: "esm",
    dts: true,
    clean: true,
  },
});
