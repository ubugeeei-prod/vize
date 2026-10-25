import { defineNuxtConfig } from "@nuxt/bridge";

const vite = process.env.VIZE_NUXT2_BRIDGE_BUILDER === "vite";
export default defineNuxtConfig({
  // The causal control runs the identical pinned host without loading Vize.
  modules: process.env.VIZE_NUXT2_BRIDGE_PLAIN_HOST === "true" ? [] : ["@vizejs/nuxt"],
  bridge: { vite, nitro: vite },
  // This pinned legacy host mutates Vue 2 platform hooks at runtime. Preserve
  // those assignments and call arguments in Nitro's production CJS bundle.
  ...(vite ? { nitro: { rollupConfig: { treeshake: false } } } : {}),
  build: { loaders: { imgUrl: { limit: 0 } } },
  vite: { build: { assetsInlineLimit: 0 } },
  vize: {
    lint: { autoInit: false },
    checker: { include: ["components/**/*.vue"] },
    musea: false,
  },
});
