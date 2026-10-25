import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
export default defineConfig({
  plugins: [vize()],
  test: {
    projects: [
      {
        extends: true,
        resolve: { alias: { "#x": "/dev/null" } },
        test: { name: "unit", environment: "node", include: ["src/**/*.spec.ts"] },
      },
    ],
  },
});
