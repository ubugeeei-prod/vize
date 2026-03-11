import type { Context } from "@oxlint/plugins";

import type { PatinaSettings } from "./model.js";

export function isVueLikeFile(filename: string): boolean {
  return filename.endsWith(".vue");
}

export function getPatinaSettings(context: Context): PatinaSettings {
  return parsePatinaSettings((context.settings as Record<string, unknown>).patina);
}

export function parsePatinaSettings(patina: unknown): PatinaSettings {
  if (typeof patina !== "object" || patina === null || Array.isArray(patina)) {
    return {};
  }

  const patinaRecord = patina as Record<string, unknown>;
  const locale = patinaRecord.locale;
  const showHelp = patinaRecord.showHelp;
  const resolved: PatinaSettings = {};

  if (typeof locale === "string") {
    resolved.locale = locale;
  }

  if (typeof showHelp === "boolean") {
    resolved.showHelp = showHelp;
  }

  return resolved;
}

export function getCacheKey(filename: string, settings: PatinaSettings): string {
  return `${filename}::${settings.locale ?? ""}`;
}

if (import.meta.vitest) {
  const { describe, expect, it } = import.meta.vitest;

  describe("parsePatinaSettings", () => {
    it("reads locale and showHelp flags", () => {
      expect(parsePatinaSettings({ locale: "ja", showHelp: false })).toEqual({
        locale: "ja",
        showHelp: false,
      });
    });

    it("ignores invalid values", () => {
      expect(parsePatinaSettings({ locale: 1, showHelp: "no" })).toEqual({});
    });

    it("ignores non-object patina settings", () => {
      expect(parsePatinaSettings(null)).toEqual({});
      expect(parsePatinaSettings("ja")).toEqual({});
    });
  });
}
