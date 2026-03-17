/**
 * Auto rule injection (VueLoaderPlugin-style).
 * Rewrites `module.rules` so users only need a `.vue` rule + CSS rules.
 * Clones CSS rules for `?vue&type=style` sub-requests via `oneOf`.
 */

import type { RuleSetRule, RuleSetUseItem } from "@rspack/core";

// ---------------------------------------------------------------------------

const VIZE_LOADER_IDENT = "@vizejs/rspack-plugin/loader";
const VIZE_STYLE_LOADER_IDENT = "@vizejs/rspack-plugin/style-loader";
const VIZE_SCOPE_LOADER_IDENT = "@vizejs/rspack-plugin/scope-loader";

/** Regex patterns for style extensions we clone. */
const STYLE_EXTENSION_MAP: Record<string, string> = {
  "\\.css$": "css",
  "\\.scss$": "scss",
  "\\.sass$": "sass",
  "\\.less$": "less",
  "\\.styl(us)?$": "styl",
};

// ---------------------------------------------------------------------------

export interface RuleCloningResult {
  /** Whether rule cloning was performed */
  applied: boolean;
  /** Number of CSS rules cloned */
  clonedCount: number;
  /** Warnings to emit through the infrastructure logger */
  warnings: string[];
}

/**
 * Mutates `rules` in-place: wraps .vue rule into `oneOf`, clones CSS rules for style sub-requests.
 */
export function applyRuleCloning(
  rules: (RuleSetRule | "...")[],
  nativeCss: boolean,
): RuleCloningResult {
  const warnings: string[] = [];

  // Step 1: locate .vue rule with vize main loader
  const vueRuleIndex = rules.findIndex((r) => r !== "..." && isVueMainRule(r));

  if (vueRuleIndex === -1) {
    return { applied: false, clonedCount: 0, warnings };
  }

  const vueRule = rules[vueRuleIndex] as RuleSetRule;

  if (vueRule.oneOf) {
    return { applied: false, clonedCount: 0, warnings };
  }

  // Step 2: find CSS / preprocessor rules────
  const cssRuleEntries: Array<{
    index: number;
    rule: RuleSetRule;
    lang: string;
  }> = [];

  for (let i = 0; i < rules.length; i++) {
    if (i === vueRuleIndex) continue;
    const rule = rules[i];
    if (rule === "...") continue;

    const lang = detectStyleLang(rule);
    if (lang) {
      cssRuleEntries.push({ index: i, rule, lang });
    }
  }

  // Step 3: clone CSS rules for vue style sub-requests────
  const clonedStyleRules: RuleSetRule[] = [];

  for (const entry of cssRuleEntries) {
    const cloned = cloneRuleForVueStyle(entry.rule, entry.lang, nativeCss);
    if (cloned) {
      clonedStyleRules.push(cloned);
    }
  }

  // Ensure fallback rule for plain <style lang="css"> blocks
  const hasCssFallback = clonedStyleRules.some(
    (r) =>
      r.resourceQuery instanceof RegExp && r.resourceQuery.test("vue&type=style&index=0&lang=css"),
  );

  if (!hasCssFallback) {
    clonedStyleRules.push(createFallbackStyleRule(nativeCss));
  }

  // Step 4: build oneOf
  const mainLoaderBranch: RuleSetRule = {
    use: normalizeUseFromRule(vueRule),
  };

  const oneOf: RuleSetRule[] = [...clonedStyleRules, mainLoaderBranch];

  // Replace original .vue rule with oneOf version
  rules[vueRuleIndex] = {
    test: vueRule.test,
    oneOf,
  };

  // Step 5: exclude vue sub-requests from original CSS rules────
  for (const entry of cssRuleEntries) {
    addVueExclusion(entry.rule);
  }

  return {
    applied: true,
    clonedCount: clonedStyleRules.length,
    warnings,
  };
}

// ---------------------------------------------------------------------------

/** Check if a rule's use chain contains the vize main loader. */
function isVueMainRule(rule: RuleSetRule): boolean {
  // Must target .vue files
  if (!testMatchesVue(rule.test)) return false;

  const uses = normalizeUseFromRule(rule);
  return uses.some((u) => {
    const loader = typeof u === "string" ? u : (u as { loader?: string }).loader;
    return loader ? isVizeMainLoader(loader) : false;
  });
}

function isVizeMainLoader(loader: string): boolean {
  // Match package export or resolved file path
  return (
    loader === VIZE_LOADER_IDENT ||
    (loader.includes("rspack-vize-plugin") &&
      loader.includes("loader") &&
      !loader.includes("style-loader") &&
      !loader.includes("scope-loader"))
  );
}

function testMatchesVue(test: RuleSetRule["test"]): boolean {
  if (!test) return false;
  if (test instanceof RegExp) {
    return test.test("App.vue") || test.test("foo.vue");
  }
  if (typeof test === "string") {
    return test.includes(".vue");
  }
  return false;
}

/** Detect if a rule targets a CSS/preprocessor extension. Returns lang name or null. */
function detectStyleLang(rule: RuleSetRule): string | null {
  const test = rule.test;
  if (!test || !(test instanceof RegExp)) return null;

  const testStr = test.source;
  for (const [pattern, lang] of Object.entries(STYLE_EXTENSION_MAP)) {
    if (testStr.includes(pattern) || testStr === pattern) {
      return lang;
    }
  }

  // Also check common patterns
  if (test.test("foo.css") && !test.test("foo.vue")) return "css";
  if (test.test("foo.scss") && !test.test("foo.vue")) return "scss";
  if (test.test("foo.sass") && !test.test("foo.vue")) return "sass";
  if (test.test("foo.less") && !test.test("foo.vue")) return "less";
  if (test.test("foo.styl") && !test.test("foo.vue")) return "styl";

  return null;
}

/** Clone a CSS rule for `?vue&type=style` sub-requests, appending scope-loader + style-loader. */
function cloneRuleForVueStyle(
  rule: RuleSetRule,
  lang: string,
  nativeCss: boolean,
): RuleSetRule | null {
  const uses = normalizeUseFromRule(rule);
  if (uses.length === 0) return null;

  const resourceQuery = new RegExp(`(?=.*type=style)(?=.*lang=${lang})`);
  // Chain (right to left): style-loader → user loaders (e.g. sass-loader) → scope-loader
  const clonedUse: RuleSetUseItem[] = [
    { loader: VIZE_SCOPE_LOADER_IDENT },
    ...deepCloneUse(uses),
    { loader: VIZE_STYLE_LOADER_IDENT },
  ];

  const cloned: RuleSetRule = {
    resourceQuery,
    use: clonedUse,
  };

  // Preserve type or infer from nativeCss mode
  if (rule.type) {
    cloned.type = rule.type;
  } else if (nativeCss) {
    cloned.type = "css/auto";
  }

  return cloned;
}

/** Fallback rule for plain `<style>` blocks (lang=css). */
function createFallbackStyleRule(nativeCss: boolean): RuleSetRule {
  const resourceQuery = /(?=.*type=style)(?=.*lang=css)/;

  if (nativeCss) {
    return {
      resourceQuery,
      type: "css/auto",
      use: [{ loader: VIZE_SCOPE_LOADER_IDENT }, { loader: VIZE_STYLE_LOADER_IDENT }],
    };
  }

  // Non-native: extract CSS + scoped transform
  return {
    resourceQuery,
    type: "javascript/auto",
    use: [{ loader: VIZE_SCOPE_LOADER_IDENT }, { loader: VIZE_STYLE_LOADER_IDENT }],
  };
}

/** Exclude Vue style sub-requests from a rule via `resourceQuery: { not: [/vue/] }`. */
function addVueExclusion(rule: RuleSetRule): void {
  const existing = rule.resourceQuery;
  if (existing) {
    // Already has a `not` exclusion for vue, skip
    if (
      typeof existing === "object" &&
      !Array.isArray(existing) &&
      !(existing instanceof RegExp) &&
      "not" in existing
    ) {
      return;
    }
    // Don't overwrite complex existing conditions
    return;
  }
  rule.resourceQuery = { not: [/vue/] };
}

/** Normalize `rule.use` to an array. */
function normalizeUse(use: RuleSetRule["use"]): RuleSetUseItem[] {
  if (!use) return [];
  if (Array.isArray(use)) return use as RuleSetUseItem[];
  return [use as RuleSetUseItem];
}

/** Normalize a rule's loaders from `rule.use` or `rule.loader`+`rule.options` into a flat array. */
function normalizeUseFromRule(rule: RuleSetRule): RuleSetUseItem[] {
  const fromUse = normalizeUse(rule.use);
  if (fromUse.length > 0) return fromUse;

  const loader = (rule as Record<string, unknown>).loader as string | undefined;
  if (loader) {
    const options = (rule as Record<string, unknown>).options as
      | Record<string, unknown>
      | undefined;
    return options ? [{ loader, options }] : [loader];
  }

  return [];
}

/** Deep-clone use items to avoid mutating originals. */
function deepCloneUse(uses: RuleSetUseItem[]): RuleSetUseItem[] {
  return uses.map((u) => {
    if (typeof u === "string") return u;
    if (typeof u === "object" && u !== null) {
      const cloned: Record<string, unknown> = { ...u };
      if ("options" in u && u.options && typeof u.options === "object") {
        cloned.options = { ...(u.options as Record<string, unknown>) };
      }
      return cloned as RuleSetUseItem;
    }
    return u;
  });
}
