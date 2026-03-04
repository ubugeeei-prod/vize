/**
 * Vize Rspack Plugin
 *
 * Responsibilities (minimal):
 * 1. Inject Vue feature flags via DefinePlugin
 * 2. Development mode logging/debugging
 *
 * NOT responsible for:
 * - Style processing (handled by Loader chain)
 * - CSS extraction (handled by Rspack native or CssExtractRspackPlugin)
 * - HMR (handled by Rspack watch mode automatically)
 */

import type { Compiler } from "@rspack/core";
import type { VizeRspackPluginOptions } from "../types/index.js";
import { matchesPattern } from "../shared/utils.js";

export class VizePlugin {
  static readonly name = "VizePlugin";

  private options: VizeRspackPluginOptions;

  constructor(options: VizeRspackPluginOptions = {}) {
    this.options = options;
  }

  apply(compiler: Compiler): void {
    const logger = compiler.getInfrastructureLogger(VizePlugin.name);
    const isProduction = this.options.isProduction ?? compiler.options.mode === "production";

    // Vapor mode: SFC-level compilation is not yet supported in @vizejs/native compileSfc.
    // Template-level compileVapor exists, but full SFC Vapor output requires Rust-side changes.
    if (this.options.vapor) {
      logger.warn(
        "Vapor mode is enabled but SFC-level Vapor compilation is not yet supported by @vizejs/native. " +
          "Only `__VUE_PROD_HYDRATION_MISMATCH_DETAILS__` will be set. " +
          "Track https://github.com/nicepkg/vize for updates.",
      );
    }

    const isCssNativeEnabled = Boolean(
      (compiler.options as { experiments?: { css?: boolean } }).experiments?.css,
    );

    if (this.options.css?.native && !isCssNativeEnabled) {
      logger.warn(
        "`css.native: true` is set but `experiments.css` is not enabled in rspack config.",
      );
    }

    // 1. Inject Vue feature flags (only if not already defined by another plugin)
    // Use compiler.webpack to get DefinePlugin for webpack/Rspack compatibility
    const { DefinePlugin } = compiler.webpack;

    // Collect existing definitions from all DefinePlugin instances
    const existingDefines = new Set<string>();
    for (const plugin of compiler.options.plugins ?? []) {
      const defs = (plugin as unknown as { definitions?: Record<string, unknown> })?.definitions;
      if (defs) {
        for (const key of Object.keys(defs)) {
          existingDefines.add(key);
        }
      }
    }

    const vueDefines: Record<string, string> = {};
    if (!existingDefines.has("__VUE_OPTIONS_API__")) {
      vueDefines["__VUE_OPTIONS_API__"] = JSON.stringify(true);
    }
    if (!existingDefines.has("__VUE_PROD_DEVTOOLS__")) {
      vueDefines["__VUE_PROD_DEVTOOLS__"] = JSON.stringify(!isProduction);
    }
    if (!existingDefines.has("__VUE_PROD_HYDRATION_MISMATCH_DETAILS__")) {
      vueDefines["__VUE_PROD_HYDRATION_MISMATCH_DETAILS__"] = JSON.stringify(!isProduction);
    }

    if (Object.keys(vueDefines).length > 0) {
      new DefinePlugin(vueDefines).apply(compiler);
    }

    // 2. Development mode logging (using Rspack infrastructure logger)
    if (!isProduction) {
      compiler.hooks.watchRun.tap(VizePlugin.name, (comp) => {
        const changed = comp.modifiedFiles;
        const removed = comp.removedFiles;

        if (changed) {
          for (const file of changed) {
            if (file.endsWith(".vue") && this.shouldHandleFile(file)) {
              logger.debug(`Vue file changed: ${file}`);
              // Rspack will automatically re-run the loader matching this file,
              // and the style imports injected by the loader will be re-resolved.
              // No need to manually operate virtual modules or trigger recompilation.
            }
          }
        }

        if (removed) {
          for (const file of removed) {
            if (file.endsWith(".vue") && this.shouldHandleFile(file)) {
              logger.debug(`Vue file removed: ${file}`);
            }
          }
        }
      });
    }
  }

  private shouldHandleFile(file: string): boolean {
    if (!matchesPattern(file, this.options.include, true)) {
      return false;
    }

    if (matchesPattern(file, this.options.exclude, false)) {
      return false;
    }

    return true;
  }
}
