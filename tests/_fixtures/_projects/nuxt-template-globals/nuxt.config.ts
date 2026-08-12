// Prepared Nuxt project whose generated `.nuxt` artifacts declare the template
// globals a Nuxt app really gets: `$t` and friends come from the `vue-i18n`
// package the generated auto-import manifest re-exports, and `$shout` comes
// from a plugin declaration `nuxi prepare` writes. `vize check` must resolve
// both from those declarations and must not invent stand-ins for anything they
// do not declare.
export default defineNuxtConfig({
  modules: ["@nuxtjs/i18n"],
  typescript: {
    strict: true,
  },
});
