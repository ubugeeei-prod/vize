import type { PatinaRuleOptions, PatinaSettings } from "./model.js";
import { getRuleOptions } from "./rule-options.js";

export interface RuleSelection {
  names: string[];
  options: PatinaRuleOptions;
  optionsByRule: Map<string, PatinaRuleOptions | undefined>;
  cacheKey: string;
}

// Settings are JSON-compatible and usually shared by every rule visitor.
// Keep only weak references and check authored bytes so in-place config edits
// cannot reuse an old selection in a long-lived host.
const selectionCache = new WeakMap<object, { revision: string; selection: RuleSelection }>();

/** A selection is a batching hint, never a second reporting/severity gate. */
export function parseRuleSelection(rules: PatinaSettings["rules"]): RuleSelection | undefined {
  if (rules === undefined) {
    return undefined;
  }
  const revision = JSON.stringify(rules);
  const cached = selectionCache.get(rules);
  if (cached?.revision === revision) {
    return cached.selection;
  }

  const optionsByRule = new Map<string, PatinaRuleOptions | undefined>();
  if (Array.isArray(rules)) {
    for (const name of rules) {
      if (typeof name === "string" && name.length > 0) {
        optionsByRule.set(normalizeRuleName(name), undefined);
      }
    }
  } else if (typeof rules === "object" && rules !== null) {
    const usesPluginIds = Object.keys(rules).some((id) => id.startsWith("vize/"));
    for (const [id, entry] of Object.entries(rules)) {
      // A complete Oxlint rule map can include core and other-plugin rules.
      if (
        !id.startsWith("vize/") &&
        (usesPluginIds || !/^(vue|script|style|css|type|nuxt|ecosystem)\//u.test(id))
      ) {
        continue;
      }
      const severity = Array.isArray(entry) ? entry[0] : entry;
      if (severity !== "error" && severity !== "warn" && severity !== 1 && severity !== 2) {
        continue;
      }
      const name = normalizeRuleName(id);
      const options = Array.isArray(entry) ? getRuleOptions(name, entry.slice(1)) : undefined;
      optionsByRule.set(name, options);
    }
  } else {
    return undefined;
  }

  const selection = createRuleSelection(optionsByRule);
  selectionCache.set(rules, { revision, selection });
  return selection;
}

export function createRuleSelection(
  optionsByRule: Map<string, PatinaRuleOptions | undefined>,
): RuleSelection {
  const names = [...optionsByRule.keys()].sort();
  const options: PatinaRuleOptions = {};
  for (const name of names) {
    Object.assign(options, optionsByRule.get(name));
  }
  return {
    names,
    options,
    optionsByRule,
    cacheKey: JSON.stringify([names, ruleOptionsKey(options)]),
  };
}

function normalizeRuleName(name: string): string {
  return name.startsWith("vize/") ? name.slice("vize/".length) : name;
}

export function ruleOptionsKey(options: PatinaRuleOptions | undefined): string {
  if (options === undefined) {
    return "{}";
  }
  // Option parsers construct fields in a stable order; nested option objects
  // can arrive with different authored key order, so compare recursively.
  return JSON.stringify(options ?? {}, (_key, value: unknown) => {
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
      return value;
    }
    return Object.fromEntries(
      Object.entries(value).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)),
    );
  });
}

export function selectVizeRuleConfig<T>(rules: Record<string, T>): Record<string, T> {
  return Object.fromEntries(Object.entries(rules).filter(([id]) => id.startsWith("vize/")));
}
