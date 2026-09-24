import vue from "@vitejs/plugin-vue";
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: false,
  plugins: [vue(), Promise.resolve([{ name: "async-plugin" }, false]), null],
  server: { port: 4321 },
});

defineConfig(async () => ({
  compiler: false,
  plugins: [vue()],
}));

defineConfig({
  // @ts-expect-error unnamed plugins are invalid
  plugins: [123],
});
