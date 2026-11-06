const vizeEnabled = process.env.VIZE_ENABLED !== "false";

export default defineNuxtConfig({
  compatibilityDate: "2024-09-19",
  experimental: { clientFallback: true },
  modules: vizeEnabled ? ["@vizejs/nuxt"] : [],
  vize: { lint: false },
});
