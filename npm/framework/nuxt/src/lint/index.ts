/** Nuxt runtime integration plus the shared `@vizejs/nuxt-lint-config` API. */
export * from "@vizejs/nuxt-lint-config";
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
