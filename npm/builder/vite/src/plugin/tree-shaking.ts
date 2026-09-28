import type { CompiledModule } from "../types.ts";
import type { SourceMapV3 } from "../utils/source-map.ts";
import { toPluginVisibleVirtualId } from "../virtual.ts";
import type { VizePluginState } from "./state.ts";

export type ResolvedSfcId =
  | string
  | { id: string; external?: boolean; moduleSideEffects?: boolean };

/** Template-only client output has no authored top-level script effects. */
export function isPureTemplateOnlyModule(compiled: CompiledModule | undefined): boolean {
  const shape = compiled?.moduleShape;
  return !!(shape?.hasNamedRenderExport && !shape.hasDefaultExport && !shape.hasSfcMainDefined);
}

export function resolveTemplateOnlySfcId(
  state: VizePluginState,
  source: string,
  isSsr: boolean,
  querySuffix: string,
): ResolvedSfcId {
  const id = toPluginVisibleVirtualId(source, isSsr, querySuffix);
  // Only authored template-only client modules can be dropped with their CSS.
  return state.isProduction && !isSsr && isPureTemplateOnlyModule(state.cache.get(source))
    ? { id, moduleSideEffects: false }
    : id;
}

export function loadedSfcModule(
  code: string,
  map: SourceMapV3 | null,
  isProduction: boolean,
  isSsr: boolean,
  compiled: CompiledModule,
): { code: string; map: SourceMapV3 | null; moduleSideEffects?: false } {
  return {
    code,
    map,
    ...(isProduction && !isSsr && isPureTemplateOnlyModule(compiled)
      ? { moduleSideEffects: false as const }
      : {}),
  };
}
