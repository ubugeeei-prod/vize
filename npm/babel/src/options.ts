import type { VizeBabelOptions } from "./types.ts";

export function normalizeOptions(value: VizeBabelOptions): Required<VizeBabelOptions> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new TypeError("Vize Babel options must be an object");
  }
  for (const key of Object.keys(value)) {
    if (!["jsxMode", "jsxCompat", "ssr", "scopedStyles"].includes(key)) {
      throw new TypeError(`Unknown Vize Babel option: ${key}`);
    }
  }
  const { jsxMode = "vdom", jsxCompat = "native", ssr = false, scopedStyles = "error" } = value;
  if (jsxMode !== "vdom" && jsxMode !== "vapor") {
    throw new TypeError('jsxMode must be "vdom" or "vapor"');
  }
  if (jsxCompat !== "native" && jsxCompat !== "babel") {
    throw new TypeError('jsxCompat must be "native" or "babel"');
  }
  if (typeof ssr !== "boolean") throw new TypeError("ssr must be a boolean");
  if (scopedStyles !== "error" && scopedStyles !== "metadata") {
    throw new TypeError('scopedStyles must be "error" or "metadata"');
  }
  return { jsxMode, jsxCompat, ssr, scopedStyles };
}
