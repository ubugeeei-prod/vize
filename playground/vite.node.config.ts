import { defineConfig } from "vite-plus";

export default defineConfig({
  test: {
    environment: "node",
    include: ["e2e/vite-plugin-vapor.test.ts"],
  },
});
