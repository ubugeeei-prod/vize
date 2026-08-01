/**
 * Feature-flag resolution for the shareable Nuxt lint config.
 *
 * This is a behavioural port of `resolveOptions().features` in
 * `@nuxt/eslint-config` (see `test/nuxt-eslint-compat/`). The defaults are
 * load-bearing: they decide which config blocks the generator emits, so they
 * are pinned against the real package by the differential oracle rather than
 * restated from documentation.
 */

/** Options for the Nuxt-specific rule block. */
export interface NuxtLintNuxtOptions {
  /**
   * Enforce a recommended key order in `nuxt.config`.
   *
   * @default true when `stylistic` is enabled
   */
  sortConfigKeys?: boolean;
}

/** Options for the tooling (module/library author) rule blocks. */
export interface NuxtLintToolingOptions {
  /** @default true */
  regexp?: boolean;
  /** @default true */
  unicorn?: boolean;
  /** @default true */
  jsdoc?: boolean;
}

/** Options for the import rule block. */
export interface NuxtLintImportOptions {
  /** @default "eslint-plugin-import-x" */
  package?: "eslint-plugin-import-lite" | "eslint-plugin-import-x";
}

/** Options for the TypeScript rule block. */
export interface NuxtLintTypeScriptOptions {
  /** @default true */
  strict?: boolean;
  /** Enables type-aware rules when set. */
  tsconfigPath?: string;
}

/**
 * The `features` surface of the generated lint config.
 *
 * Mirrors `NuxtESLintFeaturesOptions` from `@nuxt/eslint-config`.
 */
export interface NuxtLintFeatures {
  /**
   * Set up the baseline JavaScript, TypeScript and Vue rule blocks.
   *
   * @default true
   */
  standalone?: boolean;
  /**
   * Enable rules aimed at Nuxt module and library authors.
   *
   * @default false
   */
  tooling?: boolean | NuxtLintToolingOptions;
  /**
   * Enable the import rule block.
   *
   * @default true
   */
  import?: boolean | NuxtLintImportOptions;
  /**
   * Enable stylistic (formatting) rules.
   *
   * @default false
   */
  stylistic?: boolean | Record<string, unknown>;
  /**
   * Enable formatter delegation for non-script file types.
   *
   * @default false
   */
  formatters?: boolean | Record<string, unknown>;
  /** Options for the Nuxt-specific rule block. */
  nuxt?: NuxtLintNuxtOptions;
  /**
   * Enable TypeScript support.
   *
   * Defaults to whether `typescript` is resolvable from the project.
   */
  typescript?: boolean | NuxtLintTypeScriptOptions;
}

/** Every feature flag with its default applied. */
export interface ResolvedNuxtLintFeatures {
  standalone: boolean;
  stylistic: boolean | Record<string, unknown>;
  typescript: boolean | NuxtLintTypeScriptOptions;
  tooling: boolean | NuxtLintToolingOptions;
  formatters: boolean | Record<string, unknown>;
  nuxt: NuxtLintNuxtOptions;
  import: boolean | NuxtLintImportOptions;
}

/**
 * Whether TypeScript is present in the project.
 *
 * `@nuxt/eslint-config` calls `local-pkg`'s `isPackageExists("typescript")`
 * here. The probe is injected so the resolver stays a pure function: the
 * generator passes the real detection, tests pass a fixed answer.
 */
export type TypeScriptProbe = () => boolean;

/**
 * Apply `@nuxt/eslint-config`'s feature defaults.
 *
 * Note that only *missing* keys take a default: an explicitly passed
 * `undefined` value is spread over the defaults by the upstream implementation
 * and therefore wins, which this port reproduces by spreading the same way.
 */
export function resolveNuxtLintFeatures(
  features: NuxtLintFeatures | undefined,
  hasTypeScript: TypeScriptProbe,
): ResolvedNuxtLintFeatures {
  return {
    standalone: true,
    stylistic: false,
    typescript: hasTypeScript(),
    tooling: false,
    formatters: false,
    nuxt: {},
    import: {},
    ...features,
  } as ResolvedNuxtLintFeatures;
}

/**
 * Whether `nuxt/nuxt-config-keys-order` is enabled.
 *
 * Upstream reads `features.nuxt.sortConfigKeys` and falls back to
 * `!!features.stylistic`, so enabling stylistic rules also turns on config-key
 * sorting unless the project opts out explicitly.
 */
export function shouldSortNuxtConfigKeys(features: ResolvedNuxtLintFeatures): boolean {
  const { sortConfigKeys = !!features.stylistic } = features.nuxt || {};
  return sortConfigKeys;
}
