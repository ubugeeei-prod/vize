import { normalizeViteCssModuleFilename } from "@vizejs/native";

type GenerateScopedName = (this: unknown, name: string, filename: string, css: string) => string;

interface CssModulesConfig {
  generateScopedName?: string | GenerateScopedName;
}

interface CssUserConfig {
  css?: {
    modules?: CssModulesConfig;
  };
}

export function patchCssModuleGenerateScopedName(userConfig: CssUserConfig): void {
  const cssModules = userConfig.css?.modules;
  if (!cssModules || typeof cssModules.generateScopedName !== "function") {
    return;
  }

  const origFn = cssModules.generateScopedName;
  cssModules.generateScopedName = function (name: string, filename: string, css: string) {
    return origFn.call(this, name, normalizeCssModuleFilename(filename), css);
  };
}

function normalizeCssModuleFilename(filename: string): string {
  const normalized = normalizeViteCssModuleFilename(filename);
  if (normalized.startsWith("/@fs/")) {
    return normalized.slice("/@fs".length);
  }
  return normalized;
}
