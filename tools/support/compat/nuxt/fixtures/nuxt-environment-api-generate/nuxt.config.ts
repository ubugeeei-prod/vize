const vizeEnabled = process.env.VIZE_ENABLED !== "false";

// The reported setup: a client-only app generated through Vite's Environment API.
export default defineNuxtConfig({
  ssr: false,
  compatibilityDate: "2024-11-01",
  experimental: { viteEnvironmentApi: process.env.VITE_ENVIRONMENT_API !== "false" },
  modules: vizeEnabled ? ["@vizejs/nuxt"] : [],
  vize: { lint: false },
});
