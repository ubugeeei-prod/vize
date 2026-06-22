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
      "src/babel.ts",
      "src/esbuild.ts",
      "src/rollup.ts",
      "src/rolldown.ts",
      "src/webpack.ts",
      "src/webpack-cjs.ts",
    ],
    format: ["esm", "cjs"],
    dts: {
      resolver: "tsc",
    },
    clean: true,
    deps: {
      neverBundle: ["webpack"],
    },
  },
});
