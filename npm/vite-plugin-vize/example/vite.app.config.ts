import { defineConfig } from "vite-plus";
import { vize } from "@vizejs/vite-plugin";
import Inspect from "vite-plugin-inspect";

export default defineConfig({
  plugins: [vize(), Inspect()],
});
