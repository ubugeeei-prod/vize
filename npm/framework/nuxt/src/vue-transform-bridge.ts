import { isVizeGeneratedVueModuleId } from "./module-id.ts";

type ViteTransformFunction = (
  this: unknown,
  code: string,
  id: string,
  ...args: unknown[]
) => unknown;
type ViteTransformObject = { filter?: unknown; handler?: ViteTransformFunction };

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
  "nuxt:navigate-to-early-return",
]);

/**
 * Nuxt transforms of the SFC text, ordered before the compiler, whose gate
 * admits only the whole file or a template block. `nuxt:components-loader-pre`
 * and `nuxt:server-only-component-transform` gate on `isVue(id)`, which a Vize
 * id passes as it is.
 */
const NUXT_SFC_SOURCE_TRANSFORMS = new Set(["nuxt:server-devonly:transform"]);

/** The id Vize gives a compiled SFC module (`Foo.vue?vue&vize`). */
export const VIZE_VUE_MODULE_ID_RE = /\.vue\?vue&vize(?:-ssr)?(?:&|$)/;

/**
 * Give Nuxt's `isVue(type)` gate a script identity while the plugin runs, in
 * @vitejs/plugin-vue's shape: Nuxt 4.6's gates match `vue&type=script` with the
 * two keys adjacent (`/[?&]vue&type=script\b/`).
 */
export function toNuxtVueTransformId(id: string): string {
  if (!isVizeGeneratedVueModuleId(id)) return id;

  const marker = /([?&])vue=?(?=&|$)/;
  if (marker.test(id)) return id.replace(marker, "$1vue&type=script");
  return `${id}${id.includes("?") ? "&" : "?"}vue&type=script`;
}

/** @vitejs/plugin-vue's id for a transform of the whole SFC text: the file itself. */
export function toNuxtSfcSourceTransformId(id: string): string {
  if (!isVizeGeneratedVueModuleId(id)) return id;
  const queryIndex = id.indexOf("?");
  return (queryIndex === -1 ? id : id.slice(0, queryIndex)).replace(/\.vue\.tsx?$/, ".vue");
}

type FilterPattern = string | RegExp;

function isFilterPatternList(value: unknown): value is FilterPattern | FilterPattern[] {
  return typeof value === "string" || value instanceof RegExp || Array.isArray(value);
}

/**
 * Widen an object hook's `filter.id` include so the host still calls the handler
 * for Vize modules (Nuxt 4+ moved its `isVue` gates into filters). Excludes,
 * code and module type filters stay as the host wrote them. Returns a new
 * filter and leaves the host's objects untouched.
 */
export function admitVizeModulesInHookFilter(filter: unknown): unknown {
  // Composable (array) filters have no stable shape to extend.
  if (!filter || typeof filter !== "object" || Array.isArray(filter)) return filter;
  const id = (filter as { id?: unknown }).id;

  if (isFilterPatternList(id)) {
    const include = Array.isArray(id) ? id : [id];
    // An empty include admits every id already.
    if (include.length === 0) return filter;
    return { ...filter, id: [...include, VIZE_VUE_MODULE_ID_RE] };
  }

  if (!id || typeof id !== "object") return filter;
  const { include } = id as { include?: unknown };
  if (!isFilterPatternList(include)) return filter;
  const patterns = Array.isArray(include) ? include : [include];
  if (patterns.length === 0) return filter;
  return { ...filter, id: { ...id, include: [...patterns, VIZE_VUE_MODULE_ID_RE] } };
}

/**
 * Nuxt's core transforms classify Vue submodules by their `type=` query. Vize
 * owns a single module for the complete SFC, so expose it as the script
 * submodule to the transforms after the compiler and as the file itself to
 * those of the SFC text, and retain its canonical Vite identity everywhere else.
 */
export function patchNuxtVueTypeGatedTransform(plugin: VitePluginWithTransform | undefined): void {
  if (!plugin || plugin.__vizeNuxtVueTransformPatched || !plugin.name) return;
  const toHostId = NUXT_VUE_TYPE_GATED_TRANSFORMS.has(plugin.name)
    ? toNuxtVueTransformId
    : NUXT_SFC_SOURCE_TRANSFORMS.has(plugin.name)
      ? toNuxtSfcSourceTransformId
      : null;
  if (!toHostId) return;

  if (typeof plugin.transform === "function") {
    const original = plugin.transform as ViteTransformFunction;
    plugin.transform = function (this: unknown, code: string, id: string, ...args: unknown[]) {
      return original.call(this, code, toHostId(id), ...args);
    };
    plugin.__vizeNuxtVueTransformPatched = true;
    return;
  }

  const transform = plugin.transform as ViteTransformObject | undefined;
  if (!transform || typeof transform.handler !== "function") return;

  // The host checks the filter before the handler, so the id rewrite alone never runs.
  if (transform.filter !== undefined) {
    transform.filter = admitVizeModulesInHookFilter(transform.filter);
  }
  const original = transform.handler;
  transform.handler = function (this: unknown, code: string, id: string, ...args: unknown[]) {
    return original.call(this, code, toHostId(id), ...args);
  };
  plugin.__vizeNuxtVueTransformPatched = true;
}
