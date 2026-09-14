import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite-plus";

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: "happy-dom",
    include: ["src/**/*.test.ts"],
  },
  lint: {
    ignorePatterns: ["dist/**"],
    options: { typeAware: true },
  },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: {
      index: "src/index.ts",
      panel: "src/panel.ts",
    },
    format: "esm",
    dts: { vue: true },
    plugins: [vue()],
    clean: true,
    deps: {
      neverBundle: ["vue"],
    },
  },
});
