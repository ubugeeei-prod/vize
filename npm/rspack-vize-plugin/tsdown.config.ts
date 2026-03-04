import { defineConfig } from "tsdown";

export default defineConfig({
  entry: ["src/index.ts", "src/loader/index.ts", "src/loader/style-loader.ts"],
  format: "esm",
  dts: true,
  clean: true,
  minify: true,
  sourcemap: false,
});
