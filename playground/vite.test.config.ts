import { defineConfig, type Plugin } from "vite-plus";
import { playwright } from "vite-plus/test/browser/providers/playwright";
import { vize } from "@vizejs/vite-plugin";

const testOutputIgnorePattern = ["**", "target", "vize-tests", "**"].join("/");
// Vite+ 0.1.24's browser prebundle drops a Vue shared initializer from this entry.
// Apply the exclusion after Vitest has created its separate browser environments.
const vueTestUtilsOptimizerWorkaround: Plugin = {
  name: "vize:vue-test-utils-optimizer-workaround",
  configResolved: {
    order: "post",
    handler(config) {
      const optimizationConfigs = [
        config.optimizeDeps,
        ...Object.values(config.environments).map((environment) =>
          environment ? environment.optimizeDeps : undefined,
        ),
      ];

      for (const optimizeDeps of optimizationConfigs) {
        if (!optimizeDeps) continue;
        optimizeDeps.include = optimizeDeps.include?.filter(
          (dependency) => dependency !== "@vue/test-utils",
        );
        optimizeDeps.exclude ??= [];
        if (!optimizeDeps.exclude.includes("@vue/test-utils")) {
          optimizeDeps.exclude.push("@vue/test-utils");
        }
      }
    },
  },
};

export default defineConfig({
  plugins: [vueTestUtilsOptimizerWorkaround, vize()],
  resolve: {
    alias: [{ find: /^vue$/, replacement: "vue/dist/vue.runtime.esm-bundler.js" }],
    dedupe: ["vue"],
  },
  optimizeDeps: {
    include: ["vue"],
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
