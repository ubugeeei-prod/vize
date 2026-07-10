/**
 * Visual Regression Testing (VRT) module for Musea.
 *
 * Re-exports all VRT functionality from submodules:
 * - comparison: Pixel comparison utilities (PNG I/O, color delta, anti-aliasing)
 * - runner: MuseaVrtRunner class for browser automation and screenshot comparison
 * - report: HTML and JSON report generation
 */

export {
  MuseaVrtRunner,
  type VrtResult,
  type VrtSummary,
  type ExtendedVrtOptions,
  type PixelCompareOptions,
} from "./runner.js";

export { generateVrtReport, generateVrtJsonReport } from "./report.js";

export {
  readPng,
  writePng,
  colorDelta,
  isAntiAliased,
  fileExists,
  matchGlob,
  escapeHtml,
} from "./comparison.js";

import { MuseaVrtRunner } from "./runner.js";
export default MuseaVrtRunner;
