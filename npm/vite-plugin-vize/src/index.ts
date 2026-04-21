/**
 * High-performance native Vite plugin for Vue SFC compilation powered by Vize.
 */

export { vize } from "./plugin/index.ts";
export { defineConfig, loadConfig, vizeConfigStore } from "./config.ts";
export { rewriteStaticAssetUrls as __internal_rewriteStaticAssetUrls } from "./transform.ts";
export type { VizeOptions, CompiledModule, VizeConfig, LoadConfigOptions } from "./types.ts";

// Test-only export for snapshot coverage (re-exported for backward compat).
import { rewriteStaticAssetUrls } from "./transform.ts";
export const __internal = {
  rewriteStaticAssetUrls,
};

import { vize } from "./plugin/index.ts";
export default vize;
