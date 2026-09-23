import { createRequire } from "node:module";
import { parse as parseQuery } from "node:querystring";
import type { RuleSetRule } from "@rspack/core";

const loaderName = "@vizejs/rspack-plugin/loader";
let resolvedLoader: string | undefined;
try {
  resolvedLoader = createRequire(import.meta.url)
    .resolve(loaderName)
    .replaceAll("\\", "/");
} catch {
  // Self-references may be unavailable before the package is built.
}

export function isVizeMainLoader(loader: string): boolean {
  const normalized = loader.replaceAll("\\", "/");
  return (
    loader === loaderName ||
    normalized === resolvedLoader ||
    ((normalized.includes("@vizejs/rspack-plugin/") ||
      normalized.includes("rspack-vize-plugin/")) &&
      /\/dist\/loader\/index\.[cm]?js$/.test(normalized))
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

/** Match Rspack's getOptions parsing without coercing query-string values. */
function parseLoaderOptions(value: unknown): Record<string, unknown> {
  if (typeof value === "string") {
    return value.startsWith("{") && value.endsWith("}") ? JSON.parse(value) : parseQuery(value);
  }
  return isRecord(value) ? value : {};
}

/** Enforce automatic CSS integration, or supply defaults for manually routed SFCs. */
export function applyNativeCssMode(
  rules: (RuleSetRule | "...")[],
  native: boolean,
  autoRules = true,
): void {
  function configure(entry: Record<string, unknown>): void {
    if (typeof entry.loader !== "string" || !isVizeMainLoader(entry.loader)) return;
    const options = parseLoaderOptions(entry.options);
    const css = isRecord(options.css) ? options.css : {};
    if (css.native !== undefined) {
      if (!autoRules) return;
      if (css.native !== native) {
        throw new Error(
          "[vize] Loader css.native conflicts with VizePlugin's automatic CSS mode. " +
            "Configure a matching mode on VizePlugin, or set autoRules: false and " +
            "provide matching style sub-request rules. See MIGRATION.md.",
        );
      }
    }
    entry.options = { ...options, css: { ...css, native } };
  }
  function configureUse(entry: unknown): unknown {
    if (typeof entry === "string" && isVizeMainLoader(entry)) {
      const configured = { loader: entry };
      configure(configured);
      return configured;
    }
    if (isRecord(entry)) configure(entry);
    return entry;
  }
  function visit(children: unknown[]): void {
    for (const rule of children) {
      if (!isRecord(rule)) continue;
      configure(rule);
      if (Array.isArray(rule.use)) rule.use = rule.use.map(configureUse);
      else if (rule.use !== undefined && typeof rule.use !== "function") {
        rule.use = configureUse(rule.use);
      }
      if (Array.isArray(rule.oneOf)) visit(rule.oneOf);
      if (Array.isArray(rule.rules)) visit(rule.rules);
    }
  }
  visit(rules);
}
