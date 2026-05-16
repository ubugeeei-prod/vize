/**
 * Utility functions for VRT runner.
 *
 * Includes summary generation, URL builders, and helpers
 * used across the runner modules.
 */

import type { VrtResult, VrtSummary } from "./types.js";

/**
 * Build URL for variant preview.
 */
export function buildVariantUrl(baseUrl: string, artPath: string, variantName: string): string {
  const encodedPath = encodeURIComponent(artPath);
  const encodedVariant = encodeURIComponent(variantName);
  return `${baseUrl}/__musea__/preview?art=${encodedPath}&variant=${encodedVariant}`;
}

/**
 * Compute VRT summary statistics from a list of results.
 */
export function computeSummary(results: VrtResult[], startTime: number): VrtSummary {
  return {
    total: results.length,
    passed: results.filter((r) => r.passed && !r.isNew).length,
    failed: results.filter((r) => !r.passed && !r.error).length,
    new: results.filter((r) => r.isNew).length,
    skipped: results.filter((r) => r.error).length,
    duration: Date.now() - startTime,
  };
}
