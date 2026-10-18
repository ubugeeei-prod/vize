// Nuxt's schema owns the compiler fields; modules such as @vue-macros/nuxt
// extend the same @vitejs/plugin-vue options with include/exclude filters.
// Only its explicit constructor context admits this already-resolved host view.
const hostVueFields = new Set([
  "isProduction",
  "template",
  "script",
  "features",
  "include",
  "exclude",
]);

export function nativeProjectSource(source: unknown, hostContext?: "nuxt"): unknown {
  if (
    hostContext !== "nuxt" ||
    source === null ||
    typeof source !== "object" ||
    Array.isArray(source)
  )
    return source;
  const descriptor = Object.getOwnPropertyDescriptor(source, "vue");
  if (!descriptor) return source;
  const host = (source as { vue?: unknown }).vue;
  // The downstream strict projector must see the same accessor value we inspect.
  // Keep every original descriptor intact and capture only this one read in a view.
  let inspected = source;
  if (!("value" in descriptor)) {
    const captured = Object.getOwnPropertyDescriptors(source);
    captured.vue = {
      value: host,
      enumerable: descriptor.enumerable,
      configurable: descriptor.configurable,
      writable: false,
    };
    inspected = Object.create(Object.getPrototypeOf(source), captured) as object;
  }
  if (host === null || typeof host !== "object") return inspected;
  const prototype = Object.getPrototypeOf(host);
  if (prototype !== Object.prototype && prototype !== null) return inspected;
  const keys = Reflect.ownKeys(host);
  if (!keys.length || !keys.every((key) => typeof key === "string" && hostVueFields.has(key))) {
    return inspected;
  }
  // Copy descriptors without evaluating hooks, getters or callbacks. The original
  // full host object stays on Vite's config, for Nuxt's own Vue plugin to consume.
  const descriptors = Object.getOwnPropertyDescriptors(source);
  delete descriptors.vue;
  return Object.create(Object.getPrototypeOf(source), descriptors);
}
