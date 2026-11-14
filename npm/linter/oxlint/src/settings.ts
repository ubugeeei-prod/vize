import type { Context } from "@oxlint/plugins";

import type { HelpLevel, PatinaPreset, PatinaSettings } from "./model.js";
import { parseRuleSelection } from "./rule-selection.js";
export { isPatinaFile } from "./file-kinds.js";

const HELP_LEVELS = new Set<HelpLevel>(["none", "short", "full"]);
const TYPE_AWARE_RULE_PREFIX = "type/";
const PRESET_ALIASES = new Map<string, PatinaPreset>([
  ["generalrecommended", "general-recommended"],
  ["happypath", "general-recommended"],
  ["happy", "general-recommended"],
  ["default", "general-recommended"],
  ["recommended", "general-recommended"],
  ["essential", "essential"],
  ["ecosystem", "ecosystem"],
  ["eco", "ecosystem"],
  ["incremental", "incremental"],
  ["all", "incremental"],
  ["opinionated", "opinionated"],
  ["opnionated", "opinionated"],
  ["strict", "opinionated"],
  ["nuxt", "nuxt"],
]);

interface SettingsCacheEntry {
  readonly locale: unknown;
  readonly helpLevel: unknown;
  readonly preset: unknown;
  readonly showHelp: unknown;
  readonly typeAware: unknown;
  readonly corsaPath: unknown;
  readonly rules: unknown;
  readonly resolved: PatinaSettings;
}

// Every active Oxlint rule receives a context, but the settings object is
// shared for the file. Cache its shallow projection; nested rule mutations are
// still observed because the resolved value retains the same rules object and
// rule-selection owns its byte revision.
const settingsCache = new WeakMap<object, SettingsCacheEntry>();

export function getVizeSettings(context: Context): PatinaSettings {
  const settings = context.settings as Record<string, unknown>;
  return parseVizeSettings(settings.vize ?? settings.patina);
}

export function parseVizeSettings(vize: unknown): PatinaSettings {
  if (typeof vize !== "object" || vize === null || Array.isArray(vize)) {
    return {};
  }

  const vizeRecord = vize as Record<string, unknown>;
  const locale = vizeRecord.locale;
  const helpLevel = vizeRecord.helpLevel;
  const preset = vizeRecord.preset;
  const showHelp = vizeRecord.showHelp;
  const typeAware = vizeRecord.typeAware;
  const corsaPath = vizeRecord.corsaPath;
  const rules = vizeRecord.rules;
  const cached = settingsCache.get(vize);
  if (
    cached !== undefined &&
    cached.locale === locale &&
    cached.helpLevel === helpLevel &&
    cached.preset === preset &&
    cached.showHelp === showHelp &&
    cached.typeAware === typeAware &&
    cached.corsaPath === corsaPath &&
    cached.rules === rules
  ) {
    return cached.resolved;
  }
  const resolved: PatinaSettings = {};

  if (Array.isArray(rules) || (typeof rules === "object" && rules !== null)) {
    resolved.rules = rules as NonNullable<PatinaSettings["rules"]>;
  }

  if (typeof locale === "string") {
    resolved.locale = locale;
  }

  if (typeof preset === "string") {
    const normalizedPreset = normalizePreset(preset);
    if (normalizedPreset) {
      resolved.preset = normalizedPreset;
    }
  }

  if (typeof typeAware === "boolean") {
    resolved.typeAware = typeAware;
  }

  if (typeof corsaPath === "string") {
    resolved.corsaPath = corsaPath;
  }

  if (typeof helpLevel === "string" && HELP_LEVELS.has(helpLevel as HelpLevel)) {
    resolved.helpLevel = helpLevel as HelpLevel;
  } else if (typeof showHelp === "boolean") {
    resolved.helpLevel = showHelp ? "full" : "none";
  }

  settingsCache.set(vize, {
    locale,
    helpLevel,
    preset,
    showHelp,
    typeAware,
    corsaPath,
    rules,
    resolved,
  });
  return resolved;
}

export function getActivePreset(settings: PatinaSettings): PatinaPreset {
  // Oxlint already selected these rules. Inherited configs can lose settings,
  // so an absent preset must not silently suppress that explicit selection.
  return settings.preset ?? "incremental";
}

export function isIncrementalPreset(settings: PatinaSettings): boolean {
  return getActivePreset(settings) === "incremental";
}

export function getCacheKey(
  filename: string,
  settings: PatinaSettings,
  selection = parseRuleSelection(settings.rules),
): string {
  return JSON.stringify([
    filename,
    settings.locale ?? "",
    settings.helpLevel ?? "",
    getActivePreset(settings),
    settings.typeAware ? "type-aware" : "",
    settings.corsaPath ?? "",
    selection?.cacheKey,
  ]);
}

export function isTypeAwareRuleName(ruleName: string): boolean {
  return ruleName.startsWith(TYPE_AWARE_RULE_PREFIX);
}

function normalizePreset(value: string): PatinaPreset | undefined {
  return PRESET_ALIASES.get(value.replaceAll(/[-_\s]/gu, "").toLowerCase());
}

if (import.meta.vitest) {
  const { describe, expect, it } = import.meta.vitest;

  describe("parseVizeSettings", () => {
    it("reads locale, help level, preset, and type-aware settings", () => {
      expect(
        parseVizeSettings({
          locale: "ja",
          helpLevel: "short",
          preset: "essential",
          typeAware: true,
          corsaPath: "./node_modules/@typescript/typescript-linux-x64/lib/tsc",
        }),
      ).toEqual({
        locale: "ja",
        helpLevel: "short",
        preset: "essential",
        typeAware: true,
        corsaPath: "./node_modules/@typescript/typescript-linux-x64/lib/tsc",
      });
    });

    it("normalizes preset aliases", () => {
      expect(parseVizeSettings({ preset: "recommended" })).toEqual({
        preset: "general-recommended",
      });
      expect(parseVizeSettings({ preset: "incremental" })).toEqual({
        preset: "incremental",
      });
      expect(parseVizeSettings({ preset: "all" })).toEqual({
        preset: "incremental",
      });
      expect(parseVizeSettings({ preset: "ecosystem" })).toEqual({
        preset: "ecosystem",
      });
      expect(parseVizeSettings({ preset: "strict" })).toEqual({
        preset: "opinionated",
      });
      expect(parseVizeSettings({ preset: "Opnionated" })).toEqual({
        preset: "opinionated",
      });
    });

    it("falls back to showHelp for compatibility", () => {
      expect(parseVizeSettings({ locale: "ja", showHelp: false, preset: "Nuxt" })).toEqual({
        locale: "ja",
        helpLevel: "none",
        preset: "nuxt",
      });
    });

    it("ignores invalid values", () => {
      expect(
        parseVizeSettings({
          locale: 1,
          showHelp: "no",
          helpLevel: "verbose",
          preset: "wide",
          typeAware: "yes",
          corsaPath: false,
        }),
      ).toEqual({});
    });

    it("ignores non-object vize settings", () => {
      expect(parseVizeSettings(null)).toEqual({});
      expect(parseVizeSettings("ja")).toEqual({});
    });

    it("reuses one shallow projection across active rule contexts", () => {
      const rules = { "vize/vue/no-v-html": "error" };
      const authored = { helpLevel: "none", preset: "incremental", rules };
      const first = parseVizeSettings(authored);
      expect(parseVizeSettings(authored)).toBe(first);

      rules["vize/vue/no-v-html"] = "warn";
      expect(parseVizeSettings(authored)).toBe(first);
      expect(first.rules).toBe(rules);

      authored.helpLevel = "short";
      expect(parseVizeSettings(authored)).not.toBe(first);
      expect(parseVizeSettings(authored)).toEqual({
        helpLevel: "short",
        preset: "incremental",
        rules,
      });
    });
  });
}
