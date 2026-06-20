/**
 * Type definitions for VRT runner.
 */

import type {
  VrtOptions,
  CaptureConfig,
  ComparisonConfig,
  CiConfig,
  ViewportConfig,
} from "../types/index.js";

/**
 * VRT test result for a single variant.
 */
export interface VrtResult {
  artPath: string;
  variantName: string;
  viewport: ViewportConfig;
  passed: boolean;
  snapshotPath: string;
  currentPath?: string;
  diffPath?: string;
  diffPercentage?: number;
  diffPixels?: number;
  totalPixels?: number;
  error?: string;
  isNew?: boolean;
}

/**
 * VRT summary for reporting.
 */
export interface VrtSummary {
  total: number;
  passed: number;
  failed: number;
  new: number;
  skipped: number;
  duration: number;
}

/**
 * Extended VRT options aligned with Rust VrtConfig.
 */
export interface ExtendedVrtOptions extends VrtOptions {
  capture?: CaptureConfig;
  comparison?: ComparisonConfig;
  ci?: CiConfig;
  /** Enable a11y auditing during VRT */
  a11y?: boolean;
}

/**
 * Pixel comparison options.
 */
export interface PixelCompareOptions {
  /** Threshold for color difference (0-1). Default: 0.1 */
  threshold?: number;
  /** Include anti-aliasing in diff. Default: false */
  includeAA?: boolean;
  /** Alpha channel comparison. Default: true */
  alpha?: boolean;
  /** Diff highlight color */
  diffColor?: { r: number; g: number; b: number };
}
