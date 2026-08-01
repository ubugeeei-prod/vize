/** Portable checker options mirrored from `@nuxt/eslint`. */
export interface VizeNuxtLintCheckerOptions {
  /** Reuse the long-lived checker and lint only changed files after startup. */
  cache?: boolean;
  /** Files considered by the checker. */
  include?: string[];
  /** Files and directories excluded from checker runs. */
  exclude?: string[];
  /** Oxlint output formatter. */
  formatter?: string;
  /** Run a complete lint pass when the dev server starts. */
  lintOnStart?: boolean;
  /** Print warning diagnostics and surface them in the browser overlay. */
  emitWarning?: boolean;
  /** Print error diagnostics and surface them in the browser overlay. */
  emitError?: boolean;
  /** Apply safe fixes before reporting diagnostics. */
  fix?: boolean;
}

/** Project paths used by upstream's checker defaults. */
export interface NuxtLintCheckerProject {
  buildDir: string;
  srcDir: string;
}

/** Fully defaulted checker options consumed by the worker integration. */
export interface ResolvedVizeNuxtLintCheckerOptions {
  cache: boolean;
  include: string[];
  exclude: string[];
  formatter: string;
  lintOnStart: boolean;
  emitWarning: boolean;
  emitError: boolean;
  fix: boolean;
}

/**
 * Resolve the engine-neutral part of `@nuxt/eslint`'s checker contract.
 *
 * `configType` and `eslintPath` intentionally do not exist: they choose an
 * ESLint implementation, while this checker always executes oxlint + Patina.
 */
export function resolveNuxtLintCheckerOptions(
  checker: boolean | VizeNuxtLintCheckerOptions | undefined,
  project: NuxtLintCheckerProject,
): ResolvedVizeNuxtLintCheckerOptions | false {
  if (checker !== true && (checker === false || checker == null)) return false;

  const overrides = typeof checker === "object" ? checker : {};
  return {
    cache: overrides.cache ?? true,
    include: [...(overrides.include ?? [`${project.srcDir}/**/*.{js,jsx,ts,tsx,vue}`])],
    exclude: [...(overrides.exclude ?? ["**/node_modules/**", project.buildDir])],
    formatter: overrides.formatter ?? "stylish",
    lintOnStart: overrides.lintOnStart ?? true,
    emitWarning: overrides.emitWarning ?? true,
    emitError: overrides.emitError ?? true,
    fix: overrides.fix ?? false,
  };
}
