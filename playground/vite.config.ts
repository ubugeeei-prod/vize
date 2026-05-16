import { defineConfig } from "vite-plus";
export default defineConfig({
  lint: {
    ignorePatterns: [
      "dist/**",
      "node_modules/**",
      "src/wasm/**",
      "playwright-report/**",
      "e2e/vrt/test-results/**",
    ],
    options: {
      typeAware: true,
    },
    rules: {
      "no-unused-vars": "warn",
      "no-console": "off",
    },
  },
  fmt: {
    ignorePatterns: ["dist/**", "playwright-report/**", "e2e/vrt/test-results/**"],
  },
});
