import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: false,
  typecheck: { tsconfig: "tsconfig.app.json" }
});
