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
  return hasViteHmrChanges(toHmrHashes(prev), toHmrHashes(next));
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
  return detectViteHmrUpdateType(toHmrHashes(prev), toHmrHashes(next)) as HmrUpdateType;
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
