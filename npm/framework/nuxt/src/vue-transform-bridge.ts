import { isVizeGeneratedVueModuleId } from "./module-id.ts";

type ViteTransformFunction = (
  this: unknown,
  code: string,
  id: string,
  ...args: unknown[]
) => unknown;
type ViteTransformObject = { handler?: ViteTransformFunction };

type VitePluginWithTransform = {
  name?: string;
  transform?: unknown;
  __vizeNuxtVueTransformPatched?: boolean;
};

const NUXT_VUE_TYPE_GATED_TRANSFORMS = new Set([
  "nuxt:route-injection-plugin",
  "nuxt:head:unhead-imports",
  "nuxt:components-loader",
  "nuxt:lazy-hydration-macro",
  "nuxt:imports-transform",
  "unctx:transform",
  "nuxt:tree-shake-composables:transform",
  "nuxt:vue-async-context",
  "nuxt:prehydrate-transform",
]);

/** Give Nuxt's `isVue(type)` gate a script identity while the plugin runs. */
export function toNuxtVueTransformId(id: string): string {
  if (!isVizeGeneratedVueModuleId(id)) return id;

  const hasQuery = id.includes("?");
  const hasVueMarker = /[?&]vue(?:[=&]|$)/.test(id);
  const suffix = `${hasQuery ? "&" : "?"}${hasVueMarker ? "" : "vue&"}type=script`;
  return id + suffix;
}

/**
 * Nuxt's core transforms classify Vue submodules by their `type=` query. Vize
 * owns a single generated JavaScript module for the complete SFC, so expose it
 * as the script submodule only for those transforms and retain its canonical
 * Vite identity everywhere else.
 */
export function patchNuxtVueTypeGatedTransform(plugin: VitePluginWithTransform): void {
  if (
    plugin.__vizeNuxtVueTransformPatched ||
    !plugin.name ||
    !NUXT_VUE_TYPE_GATED_TRANSFORMS.has(plugin.name)
  ) {
    return;
  }

  if (typeof plugin.transform === "function") {
    const original = plugin.transform as ViteTransformFunction;
    plugin.transform = function (this: unknown, code: string, id: string, ...args: unknown[]) {
      return original.call(this, code, toNuxtVueTransformId(id), ...args);
    };
    plugin.__vizeNuxtVueTransformPatched = true;
    return;
  }

  const transform = plugin.transform as ViteTransformObject | undefined;
  if (!transform || typeof transform.handler !== "function") return;

  const original = transform.handler;
  transform.handler = function (this: unknown, code: string, id: string, ...args: unknown[]) {
    return original.call(this, code, toNuxtVueTransformId(id), ...args);
  };
  plugin.__vizeNuxtVueTransformPatched = true;
}
