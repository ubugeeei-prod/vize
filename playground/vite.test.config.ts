import { defineConfig } from "vite-plus";
import { playwright } from "vite-plus/test/browser/providers/playwright";
import { vize } from "@vizejs/vite-plugin";

const testOutputIgnorePattern = ["**", "target", "vize-tests", "**"].join("/");

export default defineConfig({
  plugins: [vize()],
  resolve: {
    alias: [{ find: /^vue$/, replacement: "vue/dist/vue.runtime.esm-bundler.js" }],
    dedupe: ["vue"],
  },
  optimizeDeps: {
    include: ["vue", "@vue/test-utils"],
  },
  test: {
    browser: {
      enabled: true,
      provider: playwright(),
      headless: true,
      instances: [{ browser: "chromium" }],
    },
    include: ["src/**/*.test.ts", "e2e/**/*.test.ts"],
    exclude: [testOutputIgnorePattern, "e2e/vite-plugin-vapor.test.ts"],
  },
});
