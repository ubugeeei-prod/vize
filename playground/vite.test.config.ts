import { defineConfig } from "vite-plus";
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
    environment: "happy-dom",
    setupFiles: ["e2e/setup.ts"],
    include: ["src/**/*.test.ts", "e2e/**/*.test.ts"],
    exclude: ["**/__agent_only/**", "e2e/vite-plugin-vapor.test.ts"],
  },
});
