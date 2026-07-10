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
    styleFingerprint(prev) !== styleFingerprint(next)
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
 * Generate HMR-aware code output based on update type.
 */
export function generateHmrCode(scopeId: string, updateType: HmrUpdateType): string {
  return generateViteHmrCode(scopeId, updateType);
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

function styleFingerprint(module: CompiledModule): string {
  return JSON.stringify({
    css: module.css,
    styles: module.styles?.map((style) => ({
      index: style.index,
      content: style.content,
    })),
  });
}
