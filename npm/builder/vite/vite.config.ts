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
      "src/vite-plus.ts",
      "src/internal/config-bridge.ts",
      "src/internal/vite-plus-task.ts",
    ],
    format: "esm",
    dts: true,
    clean: true,
  },
});
