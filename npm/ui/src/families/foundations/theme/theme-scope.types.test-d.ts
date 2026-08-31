/** Compile-only assertions for theme scope and bootstrap helpers. */

import {
  applyThemeScope,
  createThemeBootstrapScript,
  themeScopeAttributes,
  themeScopeStorageKeys,
} from "./theme-scope.ts";
import type {
  ThemeBootstrapOptions,
  ThemeScopeAttributes,
  ThemeScopeOptions,
  ThemeScopeStorageKeyName,
} from "./theme-scope.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const scopeOptions: ThemeScopeOptions = {
  presets: ["atelier", "paper"],
  density: "compact",
};
export const scopeAttributes: Readonly<ThemeScopeAttributes> = themeScopeAttributes(scopeOptions);
export const bootstrapOptions: ThemeBootstrapOptions = { fallback: scopeOptions };
export const bootstrap: string = createThemeBootstrapScript(bootstrapOptions);
export const restoreScope: () => void = applyThemeScope(
  document.createElement("section"),
  scopeOptions,
);
export const storageKey: ThemeScopeStorageKeyName = "presets";

type _StorageKeysAreClosed = Expect<Equal<ThemeScopeStorageKeyName, "presets" | "density">>;

// @ts-expect-error theme scopes reject arbitrary preset names.
themeScopeAttributes({ presets: ["atelier", "baroque"] });
// @ts-expect-error theme scopes reject arbitrary density names.
themeScopeAttributes({ density: "cozy" });
// @ts-expect-error storage key names are closed.
createThemeBootstrapScript({ storageKeys: { mode: "theme" } });
// @ts-expect-error storage keys are readonly.
themeScopeStorageKeys.presets = "theme";
