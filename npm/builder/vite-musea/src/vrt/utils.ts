/**
 * Utility functions for VRT runner.
 *
 * Includes summary generation, URL builders, and helpers
 * used across the runner modules.
 */

import type { VrtResult, VrtSummary } from "./types.js";
import type { ViewportConfig } from "../types/index.js";
import path from "node:path";

/**
 * Build URL for variant preview.
 */
export function buildVariantUrl(baseUrl: string, artPath: string, variantName: string): string {
  const encodedPath = encodeURIComponent(artPath);
  const encodedVariant = encodeURIComponent(variantName);
  return `${baseUrl}/__musea__/preview?art=${encodedPath}&variant=${encodedVariant}`;
}

/**
 * Build a filesystem-safe snapshot file name for a VRT capture.
 */
export function buildSnapshotName(
  artPath: string,
  variantName: string,
  viewport: ViewportConfig,
): string {
  const artBaseName = path.basename(artPath, ".art.vue");
  const viewportName = viewport.name || `${viewport.width}x${viewport.height}`;
  return `${encodeSnapshotNamePart(artBaseName)}--${encodeSnapshotNamePart(
    variantName,
  )}--${encodeSnapshotNamePart(viewportName)}.png`;
}

function encodeSnapshotNamePart(value: string): string {
  const encoded = encodeURIComponent(value);
  return encoded.length === 0 ? "unnamed" : encoded;
}

/**
 * Compute VRT summary statistics from a list of results.
 */
export function computeSummary(results: VrtResult[], startTime: number): VrtSummary {
  const errors = results.filter((r) => r.error).length;
  return {
    total: results.length,
    passed: results.filter((r) => r.passed && !r.isNew).length,
    failed: results.filter((r) => !r.passed && !r.error).length,
    new: results.filter((r) => r.isNew).length,
    skipped: 0,
    errors,
    duration: Date.now() - startTime,
  };
}
