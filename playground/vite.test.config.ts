import { defineConfig } from "vite-plus";
import { playwright } from "vite-plus/test/browser/providers/playwright";
import { vize } from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  resolve: {
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
    exclude: ["**/__agent_only/**", "e2e/vite-plugin-vapor.test.ts"],
  },
});
