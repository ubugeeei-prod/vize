/**
 * Adapter from a Nuxt instance to {@link NuxtLintProjectState}.
 *
 * Keeping this separate from `dirs.ts` is what lets the directory resolver stay
 * a pure function of plain data: the oracle drives it from a JSON corpus, and
 * the module drives it from a live Nuxt instance, with no shared mutable state.
 */
import type {
  NuxtLintDirNames,
  NuxtLintLayer,
  NuxtLintProjectState,
} from "@vizejs/nuxt-lint-config";

/** The subset of a Nuxt instance lint config generation reads. */
export interface NuxtLintSourceOptions {
  rootDir: string;
  srcDir?: string;
  dir?: NuxtLintDirNames;
  _layers?: Array<{ config?: Partial<NuxtLintLayer> & { srcDir?: string } }>;
}

/** Options that can redirect where the emitted globs are anchored. */
export interface NuxtLintStateOverrides {
  /**
   * Anchor the emitted globs somewhere other than the Nuxt root.
   *
   * Needed when the lint config is generated for a directory that is not the
   * project root, because every glob is written relative to it.
   */
  rootDir?: string;
}

/**
 * Reduce a Nuxt instance to the project state lint config generation needs.
 *
 * Nuxt 2 has no `_layers`, so the project is treated as a single layer rooted
 * at `srcDir` (falling back to `rootDir`). That keeps the Nuxt 2, 3 and 4 paths
 * on one code path rather than branching on the detected major version.
 */
export function toNuxtLintProjectState(
  options: NuxtLintSourceOptions,
  overrides: NuxtLintStateOverrides = {},
): NuxtLintProjectState {
  const rootDir = overrides.rootDir || options.rootDir;
  const declaredLayers = options._layers ?? [];
  const layers: NuxtLintLayer[] = declaredLayers
    .map((layer) => layer.config)
    .filter((config): config is Partial<NuxtLintLayer> & { srcDir?: string } => Boolean(config))
    .map((config) => ({
      ...config,
      srcDir: config.srcDir || options.srcDir || options.rootDir,
    }));

  return {
    rootDir,
    dir: options.dir ?? {},
    layers: layers.length > 0 ? layers : [{ srcDir: options.srcDir || options.rootDir }],
  };
}
