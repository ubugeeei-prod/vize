export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  compatibilityDate: "2024-09-19",
  nitro: { prerender: { routes: ["/"] } },
  vize: { lint: false },
});
