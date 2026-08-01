/**
 * Engine-neutral Nuxt lint preset core.
 *
 * This package can be used without a Nuxt runtime. It resolves the same
 * project directories, feature defaults, and ordered Nuxt-aware rule blocks as
 * `@nuxt/eslint-config`, while leaving emission to Vize's oxlint integration.
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
  NUXT_CONFIG_GLOBS,
  NUXT_LINT_GLOB_EXTS,
  NUXT_LINT_IGNORES,
  NUXT_RUNTIME_GLOBALS,
  buildNuxtLintPlan,
  type NuxtLintConfigItem,
  type NuxtLintSeverity,
} from "./plan.ts";
