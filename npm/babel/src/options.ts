import type { VizeBabelOptions } from "./types.ts";

export function normalizeOptions(value: VizeBabelOptions): Required<VizeBabelOptions> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new TypeError("Vize Babel options must be an object");
  }
  for (const key of Object.keys(value)) {
    if (
      ![
        "jsxMode",
        "jsxCompat",
        "ssr",
        "scopedStyles",
        "sfcStyles",
        "root",
        "isProduction",
      ].includes(key)
    ) {
      throw new TypeError(`Unknown Vize Babel option: ${key}`);
    }
  }
  const {
    jsxMode = "vdom",
    jsxCompat = "native",
    ssr = false,
    scopedStyles = "error",
    sfcStyles = "inject",
    root = process.cwd(),
    isProduction = process.env.NODE_ENV === "production",
  } = value;
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
  if (sfcStyles !== "inject" && sfcStyles !== "metadata" && sfcStyles !== "error") {
    throw new TypeError('sfcStyles must be "inject", "metadata", or "error"');
  }
  if (typeof root !== "string" || root.length === 0) {
    throw new TypeError("root must be a non-empty string");
  }
  if (typeof isProduction !== "boolean") {
    throw new TypeError("isProduction must be a boolean");
  }
  return { jsxMode, jsxCompat, ssr, scopedStyles, sfcStyles, root, isProduction };
}
