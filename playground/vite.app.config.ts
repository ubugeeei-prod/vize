import { defineConfig } from "vite-plus";
import { vize } from "@vizejs/vite-plugin";

export default defineConfig({
  base: process.env.CI ? "/play/" : "/",
  plugins: [vize({ vapor: true })],
  resolve: {
    alias: [
      { find: "@mdi/js", replacement: "@mdi/js/mdi.js" },
      { find: /^monaco-editor$/, replacement: "monaco-editor/esm/vs/editor/editor.main.js" },
      { find: /^vue$/, replacement: "vue/dist/vue.runtime.esm-bundler.js" },
    ],
    dedupe: ["vue"],
  },
  build: {
    // The playground intentionally ships Monaco workers, the TypeScript compiler,
    // and the Vize WASM bundle. After route- and formatter-level code splitting,
    // the remaining large chunks are expected vendor assets rather than regressions.
    chunkSizeWarningLimit: 7000,
  },
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
