import type { Context } from "@oxlint/plugins";

import type { HelpLevel, PatinaPreset, PatinaSettings } from "./model.js";

const HELP_LEVELS = new Set<HelpLevel>(["none", "short", "full"]);
const PRESET_ALIASES = new Map<string, PatinaPreset>([
  ["generalrecommended", "general-recommended"],
  ["happypath", "general-recommended"],
  ["happy", "general-recommended"],
  ["default", "general-recommended"],
  ["recommended", "general-recommended"],
  ["essential", "essential"],
  ["incremental", "incremental"],
  ["opinionated", "opinionated"],
  ["opnionated", "opinionated"],
  ["strict", "opinionated"],
  ["all", "opinionated"],
  ["nuxt", "nuxt"],
]);

export function isVueLikeFile(filename: string): boolean {
  return filename.endsWith(".vue");
}

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
  const resolved: PatinaSettings = {};

  if (typeof locale === "string") {
    resolved.locale = locale;
  }

  if (typeof preset === "string") {
    const normalizedPreset = normalizePreset(preset);
    if (normalizedPreset) {
      resolved.preset = normalizedPreset;
    }
  }

  if (typeof helpLevel === "string" && HELP_LEVELS.has(helpLevel as HelpLevel)) {
    resolved.helpLevel = helpLevel as HelpLevel;
    return resolved;
  }

  if (typeof showHelp === "boolean") {
    resolved.helpLevel = showHelp ? "full" : "none";
  }

  return resolved;
}

export function getActivePreset(settings: PatinaSettings): PatinaPreset {
  return settings.preset ?? "general-recommended";
}

export function isIncrementalPreset(settings: PatinaSettings): boolean {
  return getActivePreset(settings) === "incremental";
}

export function getCacheKey(filename: string, settings: PatinaSettings): string {
  return `${filename}::${settings.locale ?? ""}::${settings.helpLevel ?? ""}::${getActivePreset(settings)}`;
}

function normalizePreset(value: string): PatinaPreset | undefined {
  return PRESET_ALIASES.get(value.replaceAll(/[-_\s]/gu, "").toLowerCase());
}

if (import.meta.vitest) {
  const { describe, expect, it } = import.meta.vitest;

  describe("parseVizeSettings", () => {
    it("reads locale, help level, and preset", () => {
      expect(parseVizeSettings({ locale: "ja", helpLevel: "short", preset: "essential" })).toEqual({
        locale: "ja",
        helpLevel: "short",
        preset: "essential",
      });
    });

    it("normalizes preset aliases", () => {
      expect(parseVizeSettings({ preset: "recommended" })).toEqual({
        preset: "general-recommended",
      });
      expect(parseVizeSettings({ preset: "incremental" })).toEqual({
        preset: "incremental",
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
        parseVizeSettings({ locale: 1, showHelp: "no", helpLevel: "verbose", preset: "wide" }),
      ).toEqual({});
    });

    it("ignores non-object vize settings", () => {
      expect(parseVizeSettings(null)).toEqual({});
      expect(parseVizeSettings("ja")).toEqual({});
    });
  });
}
