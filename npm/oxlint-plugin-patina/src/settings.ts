import type { Context } from "@oxlint/plugins";

import type { PatinaSettings } from "./model.js";

export function isVueLikeFile(filename: string): boolean {
  return filename.endsWith(".vue");
}

export function getPatinaSettings(context: Context): PatinaSettings {
  const settings = context.settings as Record<string, unknown>;
  const patina = settings.patina;
  if (typeof patina !== "object" || patina === null || Array.isArray(patina)) {
    return {};
  }

  const locale = (patina as Record<string, unknown>).locale;
  return typeof locale === "string" ? { locale } : {};
}

export function getCacheKey(filename: string, settings: PatinaSettings): string {
  return `${filename}::${settings.locale ?? ""}`;
}
