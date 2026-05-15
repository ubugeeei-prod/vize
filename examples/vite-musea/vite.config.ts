import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: {
    ignorePatterns: [
      "dist/**",
      "node_modules/**",
      "playwright-report/**",
      "e2e/vrt/test-results/**",
    ],
    options: {
      typeAware: true,
    },
  },
  fmt: {
    ignorePatterns: ["dist/**", "playwright-report/**", "e2e/vrt/test-results/**"],
  },
});
