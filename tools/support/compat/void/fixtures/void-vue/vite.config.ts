import { voidVue } from "@void/vue/plugin";
import { vize } from "@vizejs/vite-plugin";
import { defineConfig } from "vite";
import { voidPlugin } from "void";

export default defineConfig({
  plugins: [voidPlugin(), vize(), voidVue()],
});
