import { defineConfig } from "vite-plus";
import { vize } from "@vizejs/vite-plugin";
import { playwright } from "vite-plus/test/browser-playwright";

export default defineConfig({
  plugins: [vize()],
  resolve: {
    dedupe: ["vue"],
  },
  optimizeDeps: {
    include: ["vue", "@vue/test-utils", "monaco-editor", "shiki", "prettier/plugins/html"],
  },
  test: {
    browser: {
      enabled: true,
      provider: playwright(),
      instances: [{ browser: "chromium" }],
    },
    include: ["src/**/*.test.ts", "e2e/**/*.test.ts"],
    exclude: ["**/__agent_only/**", "e2e/vite-plugin-vapor.test.ts"],
  },
  server: {
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
});
