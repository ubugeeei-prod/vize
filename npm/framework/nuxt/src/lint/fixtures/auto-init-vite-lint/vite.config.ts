// vite.config.ts
import { defineConfig } from "vite-plus";
export default defineConfig({
  lint: {
    plugins: ["typescript", "vue"],
    jsPlugins: ["oxlint-plugin-vize"],
    rules: { "vize/vue/require-v-for-key": "error" },
  },
});
