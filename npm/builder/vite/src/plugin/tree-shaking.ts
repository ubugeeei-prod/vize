import type { CompiledModule } from "../types.ts";
import type { SourceMapV3 } from "../utils/source-map.ts";
import { toPluginVisibleVirtualId } from "../virtual.ts";
import type { VizePluginState } from "./state.ts";

export type ResolvedSfcId =
  | string
  | { id: string; external?: boolean; moduleSideEffects?: boolean };

/**
 * Whether the module emits nothing but its render export: template-only client
 * output with no `<style>` block and no CSS. Style imports, injected CSS and
 * attached `styles` are side effects, so an SFC imported for its styles stays,
 * as with `@vitejs/plugin-vue` (#11544).
 */
export function isPureTemplateOnlyModule(compiled: CompiledModule | undefined): boolean {
  const shape = compiled?.moduleShape;
  return !!(
    shape?.hasNamedRenderExport &&
    !shape.hasDefaultExport &&
    !shape.hasSfcMainDefined &&
    !compiled.styles?.length &&
    !compiled.css
  );
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
