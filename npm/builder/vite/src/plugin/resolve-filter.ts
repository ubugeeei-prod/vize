import { id, importerId, include, or } from "@rolldown/pluginutils";
import type { Plugin } from "vite";

// A resolver can act on an SFC's imports whatever the specifier. An id-only
// filter would drop bare packages, #imports and relative script dependencies.
// Keep the id side a superset of resolve.ts's fast gate; the handler still
// checks that gate on hosts that ignore this optional composable metadata.
export const RESOLVE_ID_FILTER = [
  include(
    or(
      id(
        /^(?:\0|vize:|\/@fs|vue(?:[/?-]|$)|virtual:vize-styles$)|\.vue|vize-file=|\?macro=true|\?definePage/u,
      ),
      importerId(/^(?:\0|vize:)|\.vue/u),
    ),
  ),
];

type ResolveHandler = Exclude<NonNullable<Plugin["resolveId"]>, { handler: unknown }>;

/** Vite's public types still expose only id filters; Rolldown builds read this metadata. */
export function withResolveFilter(handler: ResolveHandler): { handler: ResolveHandler } {
  return Object.assign({ handler }, { filter: RESOLVE_ID_FILTER });
}
