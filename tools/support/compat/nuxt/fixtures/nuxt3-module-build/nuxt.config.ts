export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  vue: { compilerOptions: { whitespace: "preserve" } },
  modules: ["@vizejs/nuxt"],
  vize: {
    lint: false,
    musea: false,
  },
});
