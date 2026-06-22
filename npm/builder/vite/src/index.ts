/**
 * High-performance native Vite plugin for Vue SFC compilation powered by Vize.
 */

export { vize } from "./plugin/index.ts";
export {
  VIZE_CONFIG_FILE_ENV,
  defineConfig,
  loadConfig,
  resolveConfigExport,
  vizeConfigStore,
} from "./config.ts";
export { rewriteStaticAssetUrls as __internal_rewriteStaticAssetUrls } from "./transform.ts";
export type {
  VizeOptions,
  CompiledModule,
  MacroArtifact,
  VizeConfig,
  ResolvedVizeConfig,
  UserConfigExport,
  LoadConfigOptions,
  VizeVueVersion,
  VizeCompatibilityOptions,
} from "./types.ts";

// Test-only export for snapshot coverage (re-exported for backward compat).
import { rewriteStaticAssetUrls } from "./transform.ts";
export const __internal = {
  rewriteStaticAssetUrls,
};

import { vize } from "./plugin/index.ts";
export default vize;
