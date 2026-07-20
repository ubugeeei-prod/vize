import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: {
    ignorePatterns: ["dist/**"],
    options: { typeAware: true },
  },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: {
      index: "src/index.ts",
      button: "src/button.ts",
      checkbox: "src/checkbox.ts",
      "controllable-state": "src/controllable-state.ts",
      primitive: "src/primitive.ts",
      "visually-hidden": "src/visually-hidden.ts",
    },
    format: "esm",
    dts: { vue: true },
    plugins: [vue()],
    css: {
      inject: true,
      minify: true,
    },
    clean: true,
    deps: {
      neverBundle: ["vue"],
    },
  },
});
