import type { VizeVueFeatures } from "../vue-features.ts";

function parseDefine(value: unknown): unknown {
  try {
    return typeof value === "string" ? JSON.parse(value) : value;
  } catch {
    return value;
  }
}

/** Resolve Vue runtime defines with plugin-vue's precedence and defaults. */
export function resolveVueFeatureDefines(
  features: VizeVueFeatures | undefined,
  define: Record<string, unknown> | undefined,
  command: "build" | "serve",
): Record<string, unknown> {
  return {
    __VUE_OPTIONS_API__: features?.optionsAPI ?? parseDefine(define?.__VUE_OPTIONS_API__) ?? true,
    __VUE_PROD_DEVTOOLS__: command === "serve",
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__:
      (features?.prodHydrationMismatchDetails ||
        parseDefine(define?.__VUE_PROD_HYDRATION_MISMATCH_DETAILS__)) ??
      false,
  };
}
