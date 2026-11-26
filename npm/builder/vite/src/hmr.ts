import type { CompiledModule } from "./types.ts";
import { detectViteHmrUpdateType, generateViteHmrCode, hasViteHmrChanges } from "@vizejs/native";

/**
 * HMR update types for granular hot module replacement.
 *
 * - 'template-only': Only template changed, use rerender (preserves state)
 * - 'style-only': Only styles changed, inject CSS without component remount
 * - 'full-reload': Script changed, full component reload required
 */
export type HmrUpdateType = "template-only" | "style-only" | "full-reload";

export function hasHmrChanges(prev: CompiledModule | undefined, next: CompiledModule): boolean {
  if (!prev) {
    return true;
  }

  return (
    hasViteHmrChanges(toHmrHashes(prev), toHmrHashes(next)) ||
    runtimeFingerprint(prev) !== runtimeFingerprint(next) ||
    styleFingerprint(prev) !== styleFingerprint(next) ||
    hasCustomBlockChanges(prev, next)
  );
}

/**
 * Detect the type of HMR update needed based on content hash changes.
 *
 * @param prev - Previously compiled module (undefined if first compile)
 * @param next - Newly compiled module
 * @returns The type of HMR update needed
 */
export function detectHmrUpdateType(
  prev: CompiledModule | undefined,
  next: CompiledModule,
): HmrUpdateType {
  if (!prev) {
    return detectViteHmrUpdateType(undefined, toHmrHashes(next)) as HmrUpdateType;
  }

  if (prev.scriptHash !== next.scriptHash) {
    return "full-reload";
  }

  // A custom block sets component options, so a template or style hash must
  // not keep the update on the rerender/CSS path.
  if (hasCustomBlockChanges(prev, next)) {
    return "full-reload";
  }

  if (prev.templateHash !== next.templateHash) {
    return "template-only";
  }

  if (prev.styleHash !== next.styleHash) {
    return "style-only";
  }

  if (runtimeFingerprint(prev) !== runtimeFingerprint(next)) {
    return "full-reload";
  }

  if (styleFingerprint(prev) !== styleFingerprint(next)) {
    return "style-only";
  }

  return "full-reload";
}

/**
 * The custom event a hot update sends, before Vite's own update, with the HMR
 * ids of the SFCs whose save changed only their template.
 */
export const VIZE_RERENDER_EVENT = "vize:rerender";

/**
 * The compiled module keeps the update type of the save that produced it, but
 * the same code runs again whenever an import changes: an edited composable
 * re-executes the SFC with the cached output, and a rerender would keep the
 * setup state the old composable computed. Like @vitejs/plugin-vue, the
 * module only rerenders when the update that runs it is the save of this SFC.
 * The previous instance's listener records that, and this instance reads it
 * once, so any later execution reloads the component.
 */
function rerenderOnlyForItsOwnSave(): string {
  return `
if (import.meta.hot) {
  const rerenderRequested = import.meta.hot.data.vizeRerender === true;
  import.meta.hot.data.vizeRerender = false;
  if (!rerenderRequested && _sfc_main.__hmrUpdateType === "template-only") {
    _sfc_main.__hmrUpdateType = "full-reload";
  }
  import.meta.hot.on(${JSON.stringify(VIZE_RERENDER_EVENT)}, (data) => {
    import.meta.hot.data.vizeRerender = data.ids.includes(_sfc_main.__hmrId);
  });
}
`;
}

/**
 * Generate HMR-aware code output based on update type.
 */
export function generateHmrCode(scopeId: string, updateType: HmrUpdateType): string {
  return `${generateViteHmrCode(scopeId, updateType)}${rerenderOnlyForItsOwnSave()}`;
}

/**
 * Whether any style block changed between two compilations, regardless of the
 * update type. A save that edits both the template and a style reports
 * "template-only", but the style modules still have to be refreshed.
 */
export function hasStyleChanges(prev: CompiledModule | undefined, next: CompiledModule): boolean {
  if (!prev) return true;
  return prev.styleHash !== next.styleHash || styleFingerprint(prev) !== styleFingerprint(next);
}

/** Whether custom-block content changed, including the first time blocks appear. */
export function hasCustomBlockChanges(
  prev: CompiledModule | undefined,
  next: CompiledModule,
): boolean {
  return customBlockFingerprint(prev) !== customBlockFingerprint(next);
}

function toHmrHashes(module: CompiledModule | undefined) {
  return module
    ? {
        scriptHash: module.scriptHash,
        templateHash: module.templateHash,
        styleHash: module.styleHash,
      }
    : undefined;
}

function runtimeFingerprint(module: CompiledModule): string {
  return JSON.stringify({
    code: module.code,
    scopeId: module.scopeId,
    hasScoped: module.hasScoped,
    macroArtifacts: module.macroArtifacts?.map((artifact) => ({
      kind: artifact.kind,
      name: artifact.name,
      source: artifact.source,
      content: artifact.content,
      moduleCode: artifact.moduleCode,
      start: artifact.start,
      end: artifact.end,
    })),
    styles: module.styles?.map((style) => ({
      lang: style.lang,
      scoped: style.scoped,
      module: style.module,
      index: style.index,
    })),
    dependencies: [...(module.dependencies ?? [])].sort(),
  });
}

function customBlockFingerprint(module: CompiledModule | undefined): string {
  return JSON.stringify(
    (module?.customBlocks ?? []).map((block) => ({
      type: block.type,
      content: block.content,
      src: block.src ?? null,
      attrs: block.attrs,
      index: block.index,
    })),
  );
}

function styleFingerprint(module: CompiledModule): string {
  return JSON.stringify({
    css: module.css,
    styles: module.styles?.map((style) => ({
      index: style.index,
      content: style.content,
    })),
  });
}
