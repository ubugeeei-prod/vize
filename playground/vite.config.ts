import { defineConfig } from "vite-plus";
export default defineConfig({
  lint: {
    ignorePatterns: [
      "dist/**",
      "node_modules/**",
      "src/wasm/**",
      "__agent_only/**",
      "src/**/__agent_only/**",
      "../__agent_only/**",
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
    ignorePatterns: [
      "dist/**",
      "__agent_only/**",
      "src/**/__agent_only/**",
      "playwright-report/**",
      "e2e/vrt/test-results/**",
    ],
  },
});
