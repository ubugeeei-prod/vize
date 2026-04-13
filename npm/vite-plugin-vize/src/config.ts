/**
 * Shared Vize configuration helpers for the Vite plugin package.
 *
 * The canonical loader lives in the `vize` package so the Vite plugin and the
 * npm CLI resolve the same `vize.config.*` files with the same behavior.
 */

import type { VizeConfig } from "./types.js";
import {
  CONFIG_FILE_NAMES,
  defineConfig,
  loadConfig,
  VIZE_CONFIG_JSON_SCHEMA_PATH,
  VIZE_CONFIG_PKL_SCHEMA_PATH,
} from "vize";

export { defineConfig, loadConfig, VIZE_CONFIG_JSON_SCHEMA_PATH, VIZE_CONFIG_PKL_SCHEMA_PATH };

export const CONFIG_FILES = [...CONFIG_FILE_NAMES];

/**
 * Shared config store for inter-plugin communication.
 * Key = project root, Value = resolved VizeConfig.
 * Used by musea() and other plugins to access the unified config.
 */
export const vizeConfigStore = new Map<string, VizeConfig>();
