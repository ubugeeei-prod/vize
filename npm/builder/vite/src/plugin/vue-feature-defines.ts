import type { VizeVueFeatures } from "../vue-features.ts";

function parseDefine(value: unknown): unknown {
  try {
    return typeof value === "string" ? JSON.parse(value) : value;
  } catch {
    return value;
  }
}

/**
 * Resolve Vue runtime defines with plugin-vue's precedence and defaults.
 *
 * `__VUE_PROD_DEVTOOLS__` used to be `command === "serve"`, which left no way to
 * enable devtools for a production build (#3227). It now follows plugin-vue's
 * OR semantics, and defaults to `false` in dev as upstream does: a development
 * build enables devtools through Vue's own `__DEV__`, not through this flag.
 */
export function resolveVueFeatureDefines(
  features: VizeVueFeatures | undefined,
  define: Record<string, unknown> | undefined,
): Record<string, unknown> {
  return {
    __VUE_OPTIONS_API__: features?.optionsAPI ?? parseDefine(define?.__VUE_OPTIONS_API__) ?? true,
    __VUE_PROD_DEVTOOLS__:
      (features?.prodDevtools || parseDefine(define?.__VUE_PROD_DEVTOOLS__)) ?? false,
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__:
      (features?.prodHydrationMismatchDetails ||
        parseDefine(define?.__VUE_PROD_HYDRATION_MISMATCH_DETAILS__)) ??
      false,
  };
}
