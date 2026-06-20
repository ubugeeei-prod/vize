import {
  resolveViteCssImports,
  scopeViteCssForPipeline,
  transformViteCssVarsForPipeline,
} from "@vizejs/native";

export interface CssAliasRule {
  find: string | RegExp;
  replacement: string;
}

export interface NativeCssAliasRule {
  find: string;
  replacement: string;
  isRegex: boolean;
  flags?: string;
}

export function scopeCssForPipeline(css: string, scopeId: string): string {
  return scopeViteCssForPipeline(css, scopeId);
}

export function transformCssVarsForPipeline(css: string, scopeId: string): string {
  return transformViteCssVarsForPipeline(css, scopeId);
}

/**
 * Resolve CSS @import statements by inlining the imported files,
 * then resolve @custom-media definitions within the combined CSS.
 */
export function resolveCssImports(
  css: string,
  importer: string,
  aliasRules: CssAliasRule[],
  isDev?: boolean,
  devUrlBase?: string,
): string {
  return resolveViteCssImports(
    css,
    importer,
    aliasRules.map(toNativeCssAliasRule),
    isDev,
    devUrlBase,
  );
}

export function toNativeCssAliasRule(rule: CssAliasRule): NativeCssAliasRule {
  return typeof rule.find === "string"
    ? {
        find: rule.find,
        replacement: rule.replacement,
        isRegex: false,
      }
    : {
        find: rule.find.source,
        replacement: rule.replacement,
        isRegex: true,
        flags: rule.find.flags.replace(/[gy]/g, ""),
      };
}
