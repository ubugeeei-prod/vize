import { defineNuxtConfig } from "@nuxt/bridge";

const vite = process.env.VIZE_NUXT2_BRIDGE_BUILDER === "vite";
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  bridge: { vite, nitro: vite },
  build: { loaders: { imgUrl: { limit: 0 } } },
  vite: { build: { assetsInlineLimit: 0 } },
  vize: {
    lint: { autoInit: false },
    checker: { include: ["components/**/*.vue"] },
    musea: false,
  },
});
