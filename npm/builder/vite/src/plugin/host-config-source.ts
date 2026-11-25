// Nuxt runs Vite with `configFile: false` and owns the resolved config's `vue`
// key: its schema always fills it with the options it hands to
// @vitejs/plugin-vue, and other modules and Nuxt releases add plugin-vue options
// to it (filters, the app's `compiler` module). Vize's own Vue settings come
// from the project config files instead, so in this host the whole `vue` key
// belongs to the host by where it came from, whatever keys it has. Only the
// explicit constructor context makes this claim; other hosts stay strict.
export function nativeProjectSource(source: unknown, hostContext?: "nuxt"): unknown {
  if (
    hostContext !== "nuxt" ||
    source === null ||
    typeof source !== "object" ||
    Array.isArray(source) ||
    !Object.hasOwn(source, "vue")
  )
    return source;
  // Copy descriptors without evaluating hooks, getters or callbacks, including
  // the host key itself. The original object stays on Vite's config, for the
  // host's Vue plugin to consume.
  const descriptors = Object.getOwnPropertyDescriptors(source);
  delete descriptors.vue;
  return Object.create(Object.getPrototypeOf(source), descriptors);
}
