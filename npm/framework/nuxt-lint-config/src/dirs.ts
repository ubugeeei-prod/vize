/**
 * Project-state to shareable lint-directory resolution.
 *
 * This is a behavioural port of `getDirs()` in `@nuxt/eslint` plus the `dirs`
 * defaulting in `@nuxt/eslint-config`'s `resolveOptions()`. Every directory the
 * generated lint config globs over is derived here, so this module is the input
 * both the config emitter and the dev-server checker read.
 *
 * The exact resolution is pinned against the real packages by the differential
 * oracle in `test/nuxt-eslint-compat/`.
 */
import { posixRelative, posixResolve } from "./paths.ts";

/** A component directory declaration on a Nuxt layer. */
export type NuxtComponentDirDeclaration =
  | string
  | { path?: string; prefix?: string; [key: string]: unknown };

/** The subset of a Nuxt layer's config that lint directories are derived from. */
export interface NuxtLintLayer {
  srcDir: string;
  imports?: { dirs?: Array<string | undefined | null> };
  components?: boolean | NuxtComponentDirDeclaration[] | { dirs?: NuxtComponentDirDeclaration[] };
}

/** The `nuxt.options.dir` overrides that lint directories honour. */
export interface NuxtLintDirNames {
  pages?: string;
  layouts?: string;
  plugins?: string;
  middleware?: string;
  modules?: string;
}

/** The Nuxt project state lint directories are derived from. */
export interface NuxtLintProjectState {
  /** Directory the emitted globs are relative to. */
  rootDir: string;
  /** `nuxt.options.dir`. Missing entries fall back to their Nuxt defaults. */
  dir?: NuxtLintDirNames;
  /** `nuxt.options._layers`, in Nuxt's own order. */
  layers: NuxtLintLayer[];
}

/** Every directory list the generated lint config globs over. */
export interface NuxtLintDirs {
  pages: string[];
  composables: string[];
  components: string[];
  componentsPrefixed: string[];
  layouts: string[];
  plugins: string[];
  middleware: string[];
  modules: string[];
  servers: string[];
  root: string[];
  src: string[];
}

/**
 * The key order of {@link NuxtLintDirs}.
 *
 * Upstream serialises `dirs` with `Object.entries`, so this order is observable
 * in the generated config file and the oracle asserts it.
 */
export const NUXT_LINT_DIR_KEYS = [
  "pages",
  "composables",
  "components",
  "componentsPrefixed",
  "layouts",
  "plugins",
  "middleware",
  "modules",
  "servers",
  "root",
  "src",
] as const satisfies ReadonlyArray<keyof NuxtLintDirs>;

function emptyDirs(): NuxtLintDirs {
  return {
    pages: [],
    composables: [],
    components: [],
    componentsPrefixed: [],
    layouts: [],
    plugins: [],
    middleware: [],
    modules: [],
    servers: [],
    root: [],
    src: [],
  };
}

/**
 * Normalise one layer-relative path into a root-relative POSIX path.
 *
 * The `~/` (and `~\`) prefix is stripped rather than resolved: upstream treats a
 * tilde-prefixed `imports.dirs` entry as srcDir-relative, which is the same
 * thing Nuxt's own alias resolution does for that option.
 */
function layerRelative(rootDir: string, srcDir: string, target: string): string {
  return posixRelative(rootDir, posixResolve(srcDir, target.replace(/^~[/\\]/, "")));
}

function collectComponentDirs(
  layer: NuxtLintLayer,
  dirs: NuxtLintDirs,
  toRootRelative: (target: string) => string,
): void {
  // `components: true` and an absent `components` both mean "the default
  // `components/` directory"; only an explicit list narrows it.
  if (!layer.components || layer.components === true) {
    dirs.components.push(toRootRelative("components"));
    return;
  }

  const declarations = Array.isArray(layer.components)
    ? layer.components
    : (layer.components.dirs ?? []);

  for (const declaration of declarations) {
    if (typeof declaration === "string") {
      dirs.components.push(toRootRelative(declaration));
      continue;
    }
    if (!declaration || typeof declaration.path !== "string") {
      continue;
    }
    dirs.components.push(toRootRelative(declaration.path));
    if (declaration.prefix) {
      dirs.componentsPrefixed.push(toRootRelative(declaration.path));
    }
  }
}

/**
 * Derive every lint directory from the Nuxt project state.
 *
 * `servers` and `root` are intentionally left empty: upstream's `getDirs` never
 * populates them, and because an empty array is truthy the `||=` defaults in
 * `resolveNuxtLintDirs` do not fire for a module-generated config either. The
 * oracle pins that, so a future upstream change surfaces as a failure rather
 * than as a silently different set of linted files.
 */
export function collectNuxtLintDirs(state: NuxtLintProjectState): NuxtLintDirs {
  const dirs = emptyDirs();
  const dirNames = state.dir ?? {};

  for (const layer of state.layers) {
    const toRootRelative = (target: string) => layerRelative(state.rootDir, layer.srcDir, target);

    dirs.src.push(toRootRelative(""));
    dirs.pages.push(toRootRelative(dirNames.pages || "pages"));
    dirs.layouts.push(toRootRelative(dirNames.layouts || "layouts"));
    dirs.plugins.push(toRootRelative(dirNames.plugins || "plugins"));
    dirs.middleware.push(toRootRelative(dirNames.middleware || "middleware"));
    dirs.modules.push(toRootRelative(dirNames.modules || "modules"));
    // `composables` and `utils` are not configurable through `nuxt.options.dir`
    // upstream, so they stay literal here.
    dirs.composables.push(toRootRelative("composables"));
    dirs.composables.push(toRootRelative("utils"));
    for (const dir of layer.imports?.dirs ?? []) {
      if (dir) {
        dirs.composables.push(toRootRelative(dir));
      }
    }
    collectComponentDirs(layer, dirs, toRootRelative);
  }

  return dirs;
}

/**
 * Apply the directory defaults used when a config is written by hand rather
 * than generated from a Nuxt instance.
 *
 * Upstream uses `||=`, so a directory list that is present but empty keeps its
 * empty value. This port reproduces that, because generated configs rely on it.
 */
export function resolveNuxtLintDirs(dirs: Partial<NuxtLintDirs> | undefined): NuxtLintDirs {
  const resolved: Partial<NuxtLintDirs> = { ...dirs };
  resolved.root ||= [".", "./app"];
  resolved.src ||= resolved.root;
  const src = resolved.src;
  // Upstream interpolates (`${src}/pages`) rather than joining, so a `.` or
  // `./app` source directory keeps its leading segment instead of being
  // normalised away. Joining here would silently change which files match.
  resolved.pages ||= src.map((dir) => `${dir}/pages`);
  resolved.layouts ||= src.map((dir) => `${dir}/layouts`);
  resolved.components ||= src.map((dir) => `${dir}/components`);
  resolved.composables ||= src.map((dir) => `${dir}/composables`);
  resolved.plugins ||= src.map((dir) => `${dir}/plugins`);
  resolved.modules ||= src.map((dir) => `${dir}/modules`);
  resolved.middleware ||= src.map((dir) => `${dir}/middleware`);
  resolved.servers ||= src.map((dir) => `${dir}/servers`);
  resolved.componentsPrefixed ||= [];
  return resolved as NuxtLintDirs;
}
