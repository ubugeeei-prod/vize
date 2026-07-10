import type { ResolvedVizeConfig } from "./types.ts";
import type { ConfigEnv, LoadConfigOptions, UserConfigExport } from "./types.ts";
import { createRequire } from "node:module";

/**
 * Shared Vize configuration helpers for the Vite plugin package.
 *
 * The canonical loader lives in the `vize` package, but Nuxt 2 executes ESM
 * modules through Jiti/CJS resolution. Keep this module import-safe by avoiding
 * a top-level `vize` root import; load `vize/config` only when config IO runs.
 */

type VizeConfigModule = typeof import("vize/config");

const require = createRequire(import.meta.url);

export const CONFIG_FILE_NAMES = [
  "vize.config.pkl",
  "vize.config.ts",
  "vize.config.js",
  "vize.config.mjs",
  "vize.config.json",
] as const;

let vizeConfigModulePromise: Promise<VizeConfigModule> | null = null;

function loadVizeConfigModule(): Promise<VizeConfigModule> {
  vizeConfigModulePromise ??= import("vize/config");
  return vizeConfigModulePromise;
}

export function defineConfig(config: UserConfigExport): UserConfigExport {
  return config;
}

export async function loadConfig(
  root: string,
  options?: LoadConfigOptions,
): Promise<ResolvedVizeConfig | null> {
  return (await loadVizeConfigModule()).loadConfig(root, options);
}

export async function resolveConfigExport(
  exported: UserConfigExport,
  env?: ConfigEnv,
): Promise<ResolvedVizeConfig> {
  return (await loadVizeConfigModule()).resolveConfigExport(exported, env);
}

export const VIZE_CONFIG_JSON_SCHEMA_PATH = require.resolve("vize/schemas/vize.config.schema.json");
export const VIZE_CONFIG_PKL_SCHEMA_PATH = require.resolve("vize/pkl/vize.pkl");

export const CONFIG_FILES = [...CONFIG_FILE_NAMES];
export const VIZE_CONFIG_FILE_ENV = "VIZE_CONFIG_FILE";

/**
 * Shared config store for inter-plugin communication.
 * Key = project root, Value = resolved VizeConfig.
 * Used by musea() and other plugins to access the unified config.
 */
export const vizeConfigStore = new Map<string, ResolvedVizeConfig>();
