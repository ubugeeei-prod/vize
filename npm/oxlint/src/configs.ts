import { getPatinaRules } from "./binding.js";
import type { PatinaPreset, PatinaRuleMeta } from "./model.js";

export type OxlintRuleSeverity = "error" | "warn" | "off";
export type OxlintRuleEntry = OxlintRuleSeverity | [OxlintRuleSeverity, ...unknown[]];
export type OxlintRuleConfig = Record<string, OxlintRuleEntry>;
export type VizeRuleConfigPreset = Exclude<PatinaPreset, "incremental"> | "happy-path" | "all";
export type VizeRuleConfigPresetInput = VizeRuleConfigPreset | readonly VizeRuleConfigPreset[];
type NormalizedVizeRuleConfigPreset = Exclude<PatinaPreset, "incremental"> | "all";

export interface VizeRuleConfigOptions {
  includeTypeAware?: boolean;
  preset?: VizeRuleConfigPresetInput;
  presets?: readonly VizeRuleConfigPreset[];
}

const TYPE_AWARE_RULE_PREFIX = "type/";

export function createVizeRuleConfig(options: VizeRuleConfigOptions = {}): OxlintRuleConfig {
  const presets = resolveRuleConfigPresets(options);
  const includeTypeAware = options.includeTypeAware ?? false;
  const rules: OxlintRuleConfig = {};

  for (const ruleMeta of getPatinaRules()) {
    if (!matchesPresets(ruleMeta, presets)) {
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
  ecosystem: createVizeRuleConfig({ preset: "ecosystem" }),
  ecosystemWithTypeAware: createVizeRuleConfig({
    includeTypeAware: true,
    preset: "ecosystem",
  }),
  essential: createVizeRuleConfig({ preset: "essential" }),
  happyPath: createVizeRuleConfig({ preset: "happy-path" }),
  happyPathWithTypeAware: createVizeRuleConfig({
    includeTypeAware: true,
    preset: "happy-path",
  }),
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

function normalizeRuleConfigPreset(preset: VizeRuleConfigPreset): NormalizedVizeRuleConfigPreset {
  return preset === "happy-path" ? "general-recommended" : preset;
}

function resolveRuleConfigPresets(
  options: Pick<VizeRuleConfigOptions, "preset" | "presets">,
): NormalizedVizeRuleConfigPreset[] {
  const presetInput =
    options.preset === undefined && options.presets === undefined
      ? "general-recommended"
      : options.preset;
  const requestedPresets = [...toPresetArray(presetInput ?? []), ...(options.presets ?? [])].map(
    normalizeRuleConfigPreset,
  );

  return [...new Set(requestedPresets)];
}

function toPresetArray(
  preset: VizeRuleConfigPresetInput | readonly [],
): readonly VizeRuleConfigPreset[] {
  return Array.isArray(preset) ? preset : [preset];
}

function matchesPresets(
  ruleMeta: PatinaRuleMeta,
  presets: readonly NormalizedVizeRuleConfigPreset[],
): boolean {
  return (
    presets.includes("all") ||
    presets.some((preset) => preset !== "all" && ruleMeta.presets.includes(preset))
  );
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

    it("accepts the CLI happy-path preset alias", () => {
      expect(createVizeRuleConfig({ preset: "happy-path" })).toEqual(configs.recommended);
      expect(configs.happyPath).toEqual(configs.recommended);
    });

    it("unions multiple presets", () => {
      const config = createVizeRuleConfig({ presets: ["happy-path", "ecosystem"] });

      expect(config["vize/script/valid-define-props"]).toBe("error");
      expect(config["vize/ecosystem/router-link-require-to"]).toBe("error");
      expect(config["vize/script/no-options-api"]).toBeUndefined();
    });

    it("accepts multiple presets through the preset option", () => {
      expect(createVizeRuleConfig({ preset: ["happy-path", "ecosystem"] })).toEqual(
        createVizeRuleConfig({ presets: ["happy-path", "ecosystem"] }),
      );
    });

    it("filters rules by preset", () => {
      expect(configs.essential["vize/vue/require-v-for-key"]).toBe("error");
      expect(configs.essential["vize/vue/require-scoped-style"]).toBeUndefined();
    });

    it("enables script correctness rules in essential and happy-path", () => {
      expect(configs.essential["vize/script/valid-define-props"]).toBe("error");
      expect(configs.essential["vize/script/no-import-compiler-macros"]).toBe("error");
      expect(configs.recommended["vize/script/valid-define-props"]).toBe("error");
      expect(configs.recommended["vize/script/no-import-compiler-macros"]).toBe("error");
    });

    it("keeps low-noise happy-path script warnings out of essential", () => {
      expect(configs.recommended["vize/script/no-duplicate-attr-inheritance"]).toBe("warn");
      expect(configs.recommended["vize/script/no-reactive-destructure"]).toBeUndefined();
      expect(configs.essential["vize/script/no-duplicate-attr-inheritance"]).toBeUndefined();
      expect(configs.recommended["vize/script/no-unused-emit-declarations"]).toBeUndefined();
      expect(configs.essential["vize/script/no-unused-emit-declarations"]).toBeUndefined();
    });

    it("keeps opinionated script style out of happy-path", () => {
      expect(configs.recommended["vize/script/no-options-api"]).toBeUndefined();
      expect(configs.recommended["vize/script/define-props-declaration"]).toBeUndefined();
      expect(configs.opinionated["vize/script/no-options-api"]).toBe("error");
    });

    it("skips unstable type-aware rules by default", () => {
      expect(configs.opinionated["vize/type/require-typed-props"]).toBeUndefined();
      expect(configs.opinionatedWithTypeAware["vize/type/require-typed-props"]).toBe("warn");
    });

    it("keeps Options API allowed in the Nuxt preset", () => {
      expect(configs.nuxt["vize/script/no-options-api"]).toBeUndefined();
      expect(configs.opinionated["vize/script/no-options-api"]).toBe("error");
    });

    it("enables Nuxt framework rules only in the Nuxt and all presets", () => {
      for (const ruleName of [
        "prefer-import-meta",
        "no-page-meta-runtime-values",
        "no-nuxt-config-test-key",
        "nuxt-config-keys-order",
      ]) {
        const ruleId = `vize/nuxt/${ruleName}`;
        expect(configs.nuxt[ruleId]).toBe("error");
        expect(configs.all[ruleId]).toBe("error");
        expect(configs.ecosystem[ruleId]).toBeUndefined();
        expect(configs.opinionated[ruleId]).toBeUndefined();
        expect(configs.recommended[ruleId]).toBeUndefined();
      }
    });

    it("keeps ecosystem rules out of non-ecosystem presets until explicitly selected", () => {
      expect(configs.recommended["vize/ecosystem/router-link-require-to"]).toBeUndefined();
      expect(configs.nuxt["vize/ecosystem/nuxt-prefer-nuxt-link"]).toBe("warn");
      expect(configs.ecosystem["vize/ecosystem/nuxt-prefer-nuxt-link"]).toBeUndefined();
      expect(configs.all["vize/ecosystem/nuxt-prefer-nuxt-link"]).toBe("warn");
      expect(configs.ecosystem["vize/ecosystem/router-link-require-to"]).toBe("error");
      expect(configs.ecosystem["vize/ecosystem/vue-i18n-no-missing-key"]).toBe("warn");
      expect(configs.ecosystem["vize/ecosystem/void-link-require-href"]).toBe("error");
      expect(configs.all["vize/ecosystem/router-link-require-to"]).toBe("error");
    });
  });
}
