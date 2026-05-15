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
    entry: ["src/index.ts", "src/esbuild.ts", "src/rollup.ts", "src/webpack.ts"],
    format: "esm",
    dts: {
      resolver: "tsc",
    },
    clean: true,
    deps: {
      neverBundle: ["webpack"],
    },
  },
});
