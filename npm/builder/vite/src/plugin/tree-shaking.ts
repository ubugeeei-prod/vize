import type { CompiledModule } from "../types.ts";

/** Template-only client output has no authored top-level script effects. */
export function isPureTemplateOnlyModule(compiled: CompiledModule | undefined): boolean {
  const shape = compiled?.moduleShape;
  return !!(shape?.hasNamedRenderExport && !shape.hasDefaultExport && !shape.hasSfcMainDefined);
}
