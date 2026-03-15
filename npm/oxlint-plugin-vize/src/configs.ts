import { getPatinaRules } from "./binding.js";
import type { PatinaPreset, PatinaRuleMeta } from "./model.js";

export type OxlintRuleSeverity = "error" | "warn";
export type OxlintRuleConfig = Record<string, OxlintRuleSeverity>;
export type VizeRuleConfigPreset = Exclude<PatinaPreset, "incremental"> | "all";

export interface VizeRuleConfigOptions {
  includeTypeAware?: boolean;
  preset?: VizeRuleConfigPreset;
}

const TYPE_AWARE_RULE_PREFIX = "type/";

export function createVizeRuleConfig(options: VizeRuleConfigOptions = {}): OxlintRuleConfig {
  const preset = options.preset ?? "general-recommended";
  const includeTypeAware = options.includeTypeAware ?? false;
  const rules: OxlintRuleConfig = {};

  for (const ruleMeta of getPatinaRules()) {
    if (!matchesPreset(ruleMeta, preset)) {
      continue;
    }
    if (!includeTypeAware && isTypeAwareRule(ruleMeta)) {
      continue;
    }

    rules[`vize/${ruleMeta.name}`] = toOxlintSeverity(ruleMeta.defaultSeverity);
  }

  return rules;
}

export const configs = {
  all: createVizeRuleConfig({ preset: "all" }),
  essential: createVizeRuleConfig({ preset: "essential" }),
  nuxt: createVizeRuleConfig({ preset: "nuxt" }),
  opinionated: createVizeRuleConfig({ preset: "opinionated" }),
  opinionatedWithTypeAware: createVizeRuleConfig({
    includeTypeAware: true,
    preset: "opinionated",
  }),
  recommended: createVizeRuleConfig({ preset: "general-recommended" }),
  recommendedWithTypeAware: createVizeRuleConfig({
    includeTypeAware: true,
    preset: "general-recommended",
  }),
} as const;

function matchesPreset(ruleMeta: PatinaRuleMeta, preset: VizeRuleConfigPreset): boolean {
  return preset === "all" || ruleMeta.presets.includes(preset);
}

function isTypeAwareRule(ruleMeta: PatinaRuleMeta): boolean {
  return ruleMeta.name.startsWith(TYPE_AWARE_RULE_PREFIX);
}

function toOxlintSeverity(severity: PatinaRuleMeta["defaultSeverity"]): OxlintRuleSeverity {
  return severity === "warning" ? "warn" : severity;
}

if (import.meta.vitest) {
  const { describe, expect, it } = import.meta.vitest;

  describe("createVizeRuleConfig", () => {
    it("normalizes Vize warning severity to Oxlint's warn", () => {
      expect(configs.recommended["vize/vue/no-multi-spaces"]).toBe("warn");
    });

    it("filters rules by preset", () => {
      expect(configs.essential["vize/vue/require-v-for-key"]).toBe("error");
      expect(configs.essential["vize/vue/require-scoped-style"]).toBeUndefined();
    });

    it("skips unstable type-aware rules by default", () => {
      expect(configs.opinionated["vize/type/require-typed-props"]).toBeUndefined();
      expect(configs.opinionatedWithTypeAware["vize/type/require-typed-props"]).toBe("warn");
    });
  });
}
