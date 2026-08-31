/**
 * Cache-state control for the Vite plugin benchmark.
 *
 * `@vizejs/vite-plugin` persists its pre-compile output under
 * `<vite root>/node_modules/.vize/vite-precompile/` (see
 * `npm/builder/vite/src/plugin/precompile-cache.ts`), so a build in a directory
 * an earlier build already touched restores compiled modules from disk instead
 * of compiling them. `@vitejs/plugin-vue` has no equivalent, so a benchmark that
 * lets one arm inherit that cache is not measuring the same work on both sides.
 *
 * `tools/benchmarks/scripts/vite.ts` runs a warmup build before the measured build in the same
 * working directory, which is exactly that situation. These helpers let the
 * harness put the cache into a stated, asserted condition before each measured
 * build rather than inheriting whatever the previous build left behind.
 */

import { existsSync, readdirSync, rmSync } from "node:fs";
import { join } from "node:path";

/** Manifest directory the plugin writes, relative to the Vite root. */
export const PRECOMPILE_CACHE_RELATIVE_DIR = join("node_modules", ".vize", "vite-precompile");

/** Absolute manifest directory for a Vite root. */
export function precompileCacheDir(root) {
  return join(root, PRECOMPILE_CACHE_RELATIVE_DIR);
}

/**
 * Manifest file names currently persisted for `root`, sorted.
 *
 * Empty when the directory does not exist, which is the state a build that has
 * never run in `root` sees.
 */
export function readPrecompileCacheEntries(root) {
  const dir = precompileCacheDir(root);
  if (!existsSync(dir)) {
    return [];
  }
  return readdirSync(dir).sort();
}

/** Remove every persisted manifest for `root`. Returns the names removed. */
export function clearPrecompileCache(root) {
  const removed = readPrecompileCacheEntries(root);
  rmSync(precompileCacheDir(root), { recursive: true, force: true });
  return removed;
}

/**
 * Throw unless `root` carries no persisted pre-compile manifest.
 *
 * Called immediately before a measured build so the harness can never again
 * silently report a warm-cache time as if it were comparable to
 * `@vitejs/plugin-vue`.
 */
export function assertPrecompileCacheCold(root, label) {
  const entries = readPrecompileCacheEntries(root);
  if (entries.length === 0) {
    return;
  }
  throw new Error(
    `Refusing to measure ${label} against a warm pre-compile cache: ` +
      `${precompileCacheDir(root)} still holds ${entries.join(", ")}. ` +
      "@vitejs/plugin-vue has no persistent cache, so this would not be the same measurement.",
  );
}
