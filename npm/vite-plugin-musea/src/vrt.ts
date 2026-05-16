/**
 * Visual Regression Testing (VRT) module for Musea.
 * Uses Playwright for browser automation and pixel comparison.
 *
 * This file re-exports from the split VRT submodules for backward compatibility.
 */

export {
  MuseaVrtRunner,
  type VrtResult,
  type VrtSummary,
  type ExtendedVrtOptions,
  type PixelCompareOptions,
} from "./vrt/runner.js";

export { generateVrtReport, generateVrtJsonReport } from "./vrt/report.js";

import { MuseaVrtRunner } from "./vrt/runner.js";
export default MuseaVrtRunner;
