import { defineConfig } from "vite-plus";
import { vize } from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  test: {
    environment: "node",
    include: ["e2e/vite-plugin-vapor.test.ts", "e2e/vite-plugin-package-imports-mock.test.ts"],
  },
});
