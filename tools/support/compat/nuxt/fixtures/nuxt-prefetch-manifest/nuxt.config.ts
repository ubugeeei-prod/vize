export default defineNuxtConfig({
  ssr: false,
  modules: ["@vizejs/nuxt"],
  vize: { compiler: process.env.VIZE_COMPILER === "1", lint: false, checker: false, musea: false }
});
