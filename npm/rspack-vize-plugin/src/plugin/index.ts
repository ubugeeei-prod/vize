/** Vize Rspack Plugin — injects Vue feature flags, auto-clones style rules, dev logging. */

import type { Compiler, RuleSetRule } from "@rspack/core";
import type { VizeRspackPluginOptions } from "../types/index.js";
import { matchesPattern } from "../shared/utils.js";
import { applyRuleCloning } from "./ruleCloning.js";

export class VizePlugin {
  static readonly name = "VizePlugin";

  private options: VizeRspackPluginOptions;

  constructor(options: VizeRspackPluginOptions = {}) {
    this.options = options;
  }

  apply(compiler: Compiler): void {
    const logger = compiler.getInfrastructureLogger(VizePlugin.name);
    const isProduction = this.options.isProduction ?? compiler.options.mode === "production";

    if (this.options.vapor && !isProduction) {
      logger.debug("Vapor mode is enabled.");
    }

    const isCssNativeEnabled = Boolean(
      (compiler.options as { experiments?: { css?: boolean } }).experiments?.css,
    );

    if (this.options.css?.native && !isCssNativeEnabled) {
      logger.warn(
        "`css.native: true` is set but `experiments.css` is not enabled in rspack config.",
      );
    }

    // 1. Auto-inject style sub-request rules
    const autoRules = this.options.autoRules ?? true;
    if (autoRules) {
      const rules = compiler.options.module?.rules;
      if (rules) {
        const result = applyRuleCloning(rules as (RuleSetRule | "...")[], isCssNativeEnabled);
        if (result.applied) {
          logger.debug(
            `Auto-injected ${result.clonedCount} style rule(s) for Vue SFC sub-requests.`,
          );
        }
        for (const w of result.warnings) {
          logger.warn(w);
        }
      }
    }

    // 2. Auto-inject TypeScript post-processing for .vue files
    const autoTs = this.options.typescript ?? true;
    if (autoTs) {
      const rules = compiler.options.module?.rules;
      if (rules) {
        const alreadyHasPostRule = rules.some((r) => {
          if (r === "..." || typeof r !== "object" || r === null) return false;
          const rule = r as RuleSetRule;
          if (rule.enforce !== "post") return false;
          const t = rule.test;
          if (t instanceof RegExp) return t.test("App.vue");
          if (typeof t === "string") return t.includes(".vue");
          return false;
        });
        if (!alreadyHasPostRule) {
          rules.push({
            test: /\.vue$/,
            resourceQuery: { not: [/type=/] },
            enforce: "post" as const,
            loader: "builtin:swc-loader",
            options: {
              jsc: {
                parser: {
                  syntax: "typescript",
                },
              },
            },
            type: "javascript/auto",
          });
          logger.debug("Auto-injected TypeScript post-processing rule for .vue files.");
        }
      }
    }

    // 3. Inject Vue feature flags (skip already-defined)
    const { DefinePlugin } = compiler.webpack;
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

    // 4. Dev mode file-change logging
    if (!isProduction) {
      compiler.hooks.watchRun.tap(VizePlugin.name, (comp) => {
        const changed = comp.modifiedFiles;
        const removed = comp.removedFiles;

        if (changed) {
          for (const file of changed) {
            if (file.endsWith(".vue") && this.shouldHandleFile(file)) {
              logger.debug(`Vue file changed: ${file}`);
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
