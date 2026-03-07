import { defineConfig } from "vite";
import { vize } from "@vizejs/vite-plugin";

export default defineConfig({
  base: process.env.CI ? "/play/" : "/",
  plugins: [vize()],
  server: {
    port: 5180,
    strictPort: false,
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
  optimizeDeps: {
    include: ["monaco-editor", "shiki", "prettier/plugins/html"],
    exclude: ["vize-wasm"],
  },
});
