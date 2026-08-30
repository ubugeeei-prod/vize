import type { PatinaLintResult, PatinaRuleMeta, PatinaSettings } from "./model.js";
import { loadBinding } from "./native.js";

export function lintPatina(
  source: string,
  filename: string,
  settings: PatinaSettings,
  enabledRules?: readonly string[],
): PatinaLintResult {
  return loadBinding().lintPatinaSfc(source, {
    filename,
    locale: settings.locale,
    helpLevel: settings.helpLevel,
    preset: settings.preset,
    enabledRules: enabledRules ? [...enabledRules] : undefined,
    typeAware: settings.typeAware,
    corsaPath: settings.corsaPath,
  });
}

export function getPatinaRules(): PatinaRuleMeta[] {
  return loadBinding().getPatinaRules();
}
