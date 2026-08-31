import {
  themeDensityAttribute,
  themeDensityScales,
  themePresetAttribute,
  themePresets,
} from "./theme-constants.ts";
import type {
  ThemeBootstrapOptions,
  ThemeDensityScale,
  ThemePresetName,
  ThemeScopeAttributes,
  ThemeScopeOptions,
  ThemeScopeStorageKeyName,
} from "./theme-types.ts";

const invalidScopeDiagnostic = "VIZE_UI_THEME_SCOPE";

/** Storage keys read by {@link createThemeBootstrapScript}. */
export const themeScopeStorageKeys: Readonly<Record<ThemeScopeStorageKeyName, string>> =
  Object.freeze({
    presets: "vize-ui-theme",
    density: "vize-ui-density",
  });

function assertPresetName(name: string): asserts name is ThemePresetName {
  if (!themePresets.includes(name as ThemePresetName)) {
    throw new TypeError(`${invalidScopeDiagnostic}: unknown theme preset "${name}"`);
  }
}

function assertDensityName(name: string): asserts name is ThemeDensityScale {
  if (!Object.hasOwn(themeDensityScales, name)) {
    throw new TypeError(`${invalidScopeDiagnostic}: unknown theme density "${name}"`);
  }
}

function assertStorageKey(name: string, value: string): void {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${invalidScopeDiagnostic}: ${name} storage key needs a non-empty string`);
  }
}

function normalizePresets(presets: ThemeScopeOptions["presets"]): string | undefined {
  if (presets === undefined) return undefined;
  const values = typeof presets === "string" ? presets.trim().split(/\s+/) : [...presets];
  const normalized: ThemePresetName[] = [];
  const seen = new Set<ThemePresetName>();

  for (const value of values) {
    if (typeof value !== "string" || value.trim().length === 0) continue;
    const preset = value.trim();
    assertPresetName(preset);
    if (!seen.has(preset)) {
      seen.add(preset);
      normalized.push(preset);
    }
  }

  return normalized.length === 0 ? undefined : normalized.join(" ");
}

function normalizeDensity(density: ThemeScopeOptions["density"]): ThemeDensityScale | undefined {
  if (density === undefined) return undefined;
  assertDensityName(density);
  return density;
}

/** Normalize theme scope options into renderable attributes. */
export function themeScopeAttributes(
  scope: ThemeScopeOptions = {},
): Readonly<ThemeScopeAttributes> {
  const attributes: ThemeScopeAttributes = {};
  const presets = normalizePresets(scope.presets);
  const density = normalizeDensity(scope.density);

  if (presets !== undefined) attributes[themePresetAttribute] = presets;
  if (density !== undefined) attributes[themeDensityAttribute] = density;

  return Object.freeze(attributes);
}

/**
 * Apply theme scope attributes to one element and return a restorer.
 *
 * This is the imperative counterpart to rendering {@link themeScopeAttributes}
 * in Vue. Only provided attributes are touched; unrelated consumer attributes
 * and nested scopes remain owned by the application.
 */
export function applyThemeScope(element: Element, scope: ThemeScopeOptions): () => void {
  if (
    typeof element?.setAttribute !== "function" ||
    typeof element?.removeAttribute !== "function" ||
    typeof element?.getAttribute !== "function"
  ) {
    throw new TypeError(`${invalidScopeDiagnostic}: theme scopes need an Element`);
  }

  const attributes = themeScopeAttributes(scope);
  const previous = new Map<string, string | null>();
  for (const [attribute, value] of Object.entries(attributes)) {
    previous.set(attribute, element.getAttribute(attribute));
    element.setAttribute(attribute, value);
  }

  return () => {
    for (const [attribute, value] of previous) {
      if (value === null) element.removeAttribute(attribute);
      else element.setAttribute(attribute, value);
    }
    previous.clear();
  };
}

/**
 * Create a small inline script that applies persisted theme attributes before paint.
 *
 * The script validates stored values against this package's published preset
 * and density lists, falls back to server-rendered defaults when storage is
 * empty or blocked, and otherwise does nothing on non-browser runtimes.
 */
export function createThemeBootstrapScript(options: ThemeBootstrapOptions = {}): string {
  const storageKeys = { ...themeScopeStorageKeys, ...options.storageKeys };
  assertStorageKey("presets", storageKeys.presets);
  assertStorageKey("density", storageKeys.density);

  const fallback = themeScopeAttributes(options.fallback);
  const fallbackPresets = fallback[themePresetAttribute] ?? "";
  const fallbackDensity = fallback[themeDensityAttribute] ?? "";

  return `(function(){try{var root=document.documentElement;if(!root)return;var presets=${JSON.stringify(
    themePresets,
  )};var densities=${JSON.stringify(
    Object.keys(themeDensityScales),
  )};function read(key){try{return globalThis.localStorage&&globalThis.localStorage.getItem(key)||""}catch(_){return""}}function normalizeTheme(value){var out=[];var seen=Object.create(null);String(value||"").trim().split(/\\s+/).forEach(function(part){if(!part)return;if(presets.indexOf(part)<0){out=[];seen=null;return}if(seen&&!seen[part]){seen[part]=1;out.push(part)}});return seen?out.join(" "):""}var theme=normalizeTheme(read(${JSON.stringify(
    storageKeys.presets,
  )}))||${JSON.stringify(fallbackPresets)};if(theme)root.setAttribute(${JSON.stringify(
    themePresetAttribute,
  )},theme);var density=read(${JSON.stringify(
    storageKeys.density,
  )});if(densities.indexOf(density)<0)density=${JSON.stringify(
    fallbackDensity,
  )};if(density)root.setAttribute(${JSON.stringify(themeDensityAttribute)},density)}catch(_){}})();`;
}

export type {
  ThemeBootstrapOptions,
  ThemeDensityScale,
  ThemePresetName,
  ThemeScopeAttributeName,
  ThemeScopeAttributes,
  ThemeScopeOptions,
  ThemeScopeStorageKeyName,
} from "./theme-types.ts";
