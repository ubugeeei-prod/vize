// Prepared Nuxt project in the shape a Nuxt i18n module produces when it
// exposes its composables through its own virtual module instead of
// re-exporting `vue-i18n`: `useI18n` and `useLocalePath` are auto-imported, but
// nothing in the generated graph augments `ComponentCustomProperties`, so `$t`
// is genuinely undeclared. `vue-tsc` reports it, and `vize check` must report
// the same thing instead of inventing a `$t` from the presence of `useI18n`.
export default defineNuxtConfig({
  modules: ["@nuxtjs/i18n"],
  typescript: {
    strict: true,
  },
});
