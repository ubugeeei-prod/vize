/**
 * Nuxt-aware lint config generation for `@vizejs/nuxt`.
 *
 * The pieces here turn Nuxt project state into an engine-neutral lint plan:
 * which directories exist, which feature blocks are on, and which named rule
 * blocks apply to which globs, then emit it for oxlint. Running the plan
 * against the dev server remains a separate integration layer.
 */
export {
  NUXT_LINT_DIR_KEYS,
  collectNuxtLintDirs,
  resolveNuxtLintDirs,
  type NuxtComponentDirDeclaration,
  type NuxtLintDirNames,
  type NuxtLintDirs,
  type NuxtLintLayer,
  type NuxtLintProjectState,
} from "./dirs.ts";
export {
  resolveNuxtLintFeatures,
  shouldSortNuxtConfigKeys,
  type NuxtLintFeatures,
  type NuxtLintImportOptions,
  type NuxtLintNuxtOptions,
  type NuxtLintToolingOptions,
  type NuxtLintTypeScriptOptions,
  type ResolvedNuxtLintFeatures,
  type TypeScriptProbe,
} from "./features.ts";
export {
  toNuxtLintProjectState,
  type NuxtLintSourceOptions,
  type NuxtLintStateOverrides,
} from "./nuxt-state.ts";
export {
  NUXT_CONFIG_GLOBS,
  NUXT_LINT_GLOB_EXTS,
  NUXT_LINT_IGNORES,
  NUXT_RUNTIME_GLOBALS,
  buildNuxtLintPlan,
  type NuxtLintConfigItem,
  type NuxtLintSeverity,
} from "./plan.ts";
export { renderNuxtOxlintConfig } from "./emitter.ts";
export {
  ROOT_OXLINT_CONFIG_NAMES,
  setupNuxtLintConfigGeneration,
  writeFileIfChanged,
  type NuxtLintConfigGeneration,
  type NuxtLintGenerationDependencies,
} from "./generation.ts";
export type { VizeNuxtLintOptions } from "./options.ts";
