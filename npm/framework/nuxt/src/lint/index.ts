/** Nuxt runtime integration plus the shared `@vizejs/nuxt-lint-config` API. */
export * from "@vizejs/nuxt-lint-config";
export {
  setupNuxtLintConfigAddons,
  VIZE_NUXT_LINT_CONFIG_ADDONS_HOOK,
  type NuxtLintAwaitable,
  type NuxtLintConfigAddon,
  type NuxtLintConfigAddonNuxt,
  type NuxtLintImport,
  type ResolveNuxtLintConfigAddons,
} from "./addons.ts";
export {
  setupNuxtLintChecker,
  type NuxtLintCheckerSetup,
  type NuxtLintCheckerSetupDependencies,
} from "./checker/setup.ts";
export {
  resolveNuxtLintCheckerOptions,
  type ResolvedVizeNuxtLintCheckerOptions,
  type VizeNuxtLintCheckerOptions,
} from "./checker/options.ts";
export { createNuxtLintCheckerVitePlugin } from "./checker/vite.ts";
export { createNuxtLintCheckerWebpackPlugin } from "./checker/webpack.ts";
export {
  toNuxtLintProjectState,
  type NuxtLintSourceOptions,
  type NuxtLintStateOverrides,
} from "./nuxt-state.ts";
export { renderNuxtOxlintConfig } from "./emitter.ts";
export {
  ROOT_OXLINT_CONFIG_NAMES,
  setupNuxtLintConfigGeneration,
  writeFileIfChanged,
  type NuxtLintConfigGeneration,
  type NuxtLintGenerationDependencies,
} from "./generation.ts";
export type { VizeNuxtLintOptions } from "./options.ts";
