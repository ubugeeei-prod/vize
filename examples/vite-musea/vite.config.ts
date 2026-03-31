import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: {
    ignorePatterns: ["dist/**", "playwright-report/**", "e2e/vrt/test-results/**"],
  },
  fmt: {
    ignorePatterns: ["dist/**", "playwright-report/**", "e2e/vrt/test-results/**"],
  },
});
