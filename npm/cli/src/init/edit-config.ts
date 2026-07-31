import { VITE_LINT_BLOCK, VITE_LINT_IMPORT, VITE_PLUGIN_IMPORT } from "./templates.js";
import { countConfigCalls, findTopLevelKey, readTopLevelArray } from "./top-level.js";

/**
 * Conservative source edits for user-owned `vite.config.*` and `nuxt.config.*`.
 *
 * Every function here returns `null` rather than guessing. A wrong edit to a
 * build config breaks the project; a `null` costs the user one paste of a
 * snippet `init` prints for them.
 */

const VITE_CALLEE = "defineConfig";
const NUXT_CALLEE = "defineNuxtConfig";

/**
 * `defineConfig({` plus the newline that usually follows it.
 *
 * The trailing newline is consumed and re-emitted by the injectors so an
 * inserted key does not leave a stray blank line behind in the user's file.
 */
const VITE_OPENING = /\bdefineConfig\s*\(\s*\{[^\S\r\n]*(?:\r?\n)?/u;
const NUXT_OPENING = /\bdefineNuxtConfig\s*\(\s*\{[^\S\r\n]*(?:\r?\n)?/u;

/**
 * Whether a Vite config is a single plain `defineConfig({ ... })` call that a
 * new top-level key can be inserted into.
 *
 * Anything else -- several `defineConfig` calls, a config built from a variable,
 * or a config that already declares the key -- is left alone.
 */
export function canInjectViteKey(source: string, key: string): boolean {
  if (hasTopLevelKey(source, key)) {
    return false;
  }
  return countConfigCalls(source, VITE_CALLEE) === 1;
}

/** Whether the Vite config declares `key` at the top level of its `defineConfig` call. */
export function hasTopLevelKey(source: string, key: string): boolean {
  return findTopLevelKey(source, VITE_CALLEE, key) !== null;
}

/** Whether the Vite+ `lint` block can be injected into this source. */
export function canInjectViteLint(source: string): boolean {
  if (source.includes("oxlint-plugin-vize")) {
    return false;
  }
  return canInjectViteKey(source, "lint");
}

/**
 * Inserts the `lint` block, and its import, into a Vite config.
 *
 * Returns `null` when the source does not have the shape `canInjectViteLint`
 * accepts, so callers cannot inject blindly.
 */
export function injectViteLint(source: string): string | null {
  if (!canInjectViteLint(source)) {
    return null;
  }
  const withImport = insertImport(source, VITE_LINT_IMPORT);
  if (withImport === null) {
    return null;
  }
  return withImport.replace(VITE_OPENING, () => `defineConfig({\n${VITE_LINT_BLOCK}`);
}

/**
 * Adds `vize()` to a Vite config's top-level `plugins` array, importing the
 * plugin.
 *
 * The array is located by depth-aware scan rather than by regex: a Vite+ config
 * can carry a second `plugins` key inside its `lint` block, and appending Vize's
 * Vite plugin to Oxlint's plugin list would corrupt both.
 */
export function injectVitePlugin(source: string): string | null {
  if (source.includes("@vizejs/vite-plugin")) {
    return null;
  }
  const withImport = insertImport(source, VITE_PLUGIN_IMPORT);
  if (withImport === null) {
    return null;
  }
  const plugins = readTopLevelArray(withImport, VITE_CALLEE, "plugins");
  if (plugins !== null) {
    return insertArrayEntry(withImport, plugins.contentStart, "vize()", plugins.empty);
  }
  if (findTopLevelKey(withImport, VITE_CALLEE, "plugins") !== null) {
    // `plugins` exists but is not an array literal; inserting would change what
    // the config evaluates to.
    return null;
  }
  if (!canInjectViteKey(withImport, "plugins")) {
    return null;
  }
  return withImport.replace(VITE_OPENING, () => `defineConfig({\n  plugins: [vize()],\n`);
}

/**
 * Adds `"@vizejs/nuxt"` to a Nuxt config's top-level `modules` array.
 *
 * Nuxt owns its own Vite instance, so the module is the supported integration
 * point; adding `@vizejs/vite-plugin` to a Nuxt-managed Vite config would fight
 * it.
 */
export function injectNuxtModule(source: string): string | null {
  if (source.includes("@vizejs/nuxt")) {
    return null;
  }
  if (countConfigCalls(source, NUXT_CALLEE) !== 1) {
    return null;
  }
  const modules = readTopLevelArray(source, NUXT_CALLEE, "modules");
  if (modules !== null) {
    return insertArrayEntry(source, modules.contentStart, '"@vizejs/nuxt"', modules.empty);
  }
  if (findTopLevelKey(source, NUXT_CALLEE, "modules") !== null) {
    return null;
  }
  return source.replace(NUXT_OPENING, () => `defineNuxtConfig({\n  modules: ["@vizejs/nuxt"],\n`);
}

/**
 * Inserts `entry` as the first element of an array literal.
 *
 * Prepending keeps the user's existing entries in their original order and
 * leaves their formatting alone.
 */
function insertArrayEntry(
  source: string,
  contentStart: number,
  entry: string,
  empty: boolean,
): string {
  const suffix = empty ? "" : ", ";
  const tail = empty ? source.slice(contentStart).replace(/^\s*/u, "") : source.slice(contentStart);
  return `${source.slice(0, contentStart)}${entry}${suffix}${tail}`;
}

/**
 * Inserts an import after the last existing top-level import.
 *
 * A config with no imports at all returns `null`: the safe insertion point is
 * not obvious, and the file is unusual enough to be worth a human look.
 */
function insertImport(source: string, importLine: string): string | null {
  if (source.includes(importLine.trimEnd())) {
    return source;
  }
  const imports = [
    ...source.matchAll(
      /^import[^\r\n]*(?:from\s+["'][^"']+["']|["'][^"']+["'])\s*;?[^\S\r\n]*(?:\r?\n|$)/gmu,
    ),
  ];
  const lastImport = imports.at(-1);
  if (lastImport === undefined || lastImport.index === undefined) {
    return null;
  }
  const end = lastImport.index + lastImport[0].length;
  return source.slice(0, end) + importLine + source.slice(end);
}
