// Minimal Nuxt config for the `vize check` parity fixture. The `.nuxt`
// artifacts checked in alongside this file are what a real `nuxi prepare`
// would generate for this project, so `vize check` resolves auto-imports,
// auto-registered components and path aliases from Nuxt's own generated
// types rather than from `any` stubs.
export default defineNuxtConfig({
  typescript: {
    strict: true,
  },
});
