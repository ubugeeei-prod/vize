/**
 * Cache-state control for the Nuxt benchmark.
 *
 * `prepareNuxtDir` gives every generated app a fresh directory under
 * `target/tool-benchmark/nuxt/`, but it symlinks each app's `node_modules` at
 * one shared directory (`npm/framework/nuxt/node_modules`, falling back to
 * `tools/benchmarks/scripts/node_modules`) so the app can resolve `nuxt` and `vue`. Every build
 * cache that lives under `node_modules` is therefore shared by every measured
 * run, by both variants, and across separate invocations of the benchmark --
 * `target/tool-benchmark` is wiped at startup, but this directory is not.
 *
 * Two things live there and both distort the measurement:
 *
 * - `.vize/vite-precompile/` is `@vizejs/vite-plugin`'s persistent pre-compile
 *   manifest. Only the `@vizejs/nuxt` variant can use it, so every measured run
 *   after the first restores its modules from disk while the Nuxt default
 *   compiler recompiles. Measured: after one benchmark invocation, a second
 *   invocation with `--warmups 0 --runs 1` left the manifest's mtime unchanged,
 *   which only happens when nothing was recompiled.
 * - `.cache/` (Nuxt's and jiti's build caches) and `.vite/` (Vite's dependency
 *   optimizer) are shared by *both* variants, so whichever runs first pays to
 *   populate them and the other does not.
 *
 * Clearing all three before each measured build puts both variants in the same
 * state on every run, which is the only state in which their times can be
 * compared.
 */

import { existsSync, readdirSync, realpathSync, rmSync, symlinkSync } from "node:fs";
import { join } from "node:path";

/**
 * Build caches under a Nuxt app's `node_modules`, relative to it.
 *
 * `.vize` is the pre-compile manifest one variant owns; `.cache` and `.vite`
 * are shared by both and still carry state between runs.
 */
export const NUXT_SHARED_CACHE_DIRS = [".vize", ".cache", ".vite"];

/**
 * The directory `<root>/node_modules` actually resolves to.
 *
 * Resolved rather than joined because the benchmark's `node_modules` is a
 * symlink: reporting the link path would hide that separate app directories
 * share one cache.
 */
export function resolveNodeModulesDir(root) {
  const link = join(root, "node_modules");
  if (!existsSync(link)) {
    return null;
  }
  return realpathSync(link);
}

/** Cache directories currently present under `<root>/node_modules`, sorted. */
export function readNuxtBuildCaches(root) {
  const nodeModules = resolveNodeModulesDir(root);
  if (nodeModules === null) {
    return [];
  }
  const present = new Set(readdirSync(nodeModules));
  return NUXT_SHARED_CACHE_DIRS.filter((name) => present.has(name)).sort();
}

/**
 * Remove every shared build cache for `root`. Returns the names removed.
 *
 * Only the known cache directories are touched; the installed packages beside
 * them are what makes the app buildable at all.
 */
export function clearNuxtBuildCaches(root) {
  const nodeModules = resolveNodeModulesDir(root);
  if (nodeModules === null) {
    return [];
  }
  const removed = readNuxtBuildCaches(root);
  for (const name of removed) {
    rmSync(join(nodeModules, name), { recursive: true, force: true });
  }
  return removed;
}

/**
 * Point `<appDir>/node_modules` at the first existing `candidates` entry, then
 * put that shared directory's build caches into the cold state and prove it.
 *
 * Linking and clearing belong together: the link is what makes the caches
 * shared in the first place, so every caller that creates one owes the clear.
 */
export function linkColdNodeModules(appDir, candidates, label) {
  const target = candidates.find((candidate) => existsSync(candidate)) ?? candidates.at(-1);
  symlinkSync(target, join(appDir, "node_modules"), "dir");
  clearNuxtBuildCaches(appDir);
  assertNuxtBuildCachesCold(appDir, label);
}

/** Throw unless every shared build cache for `root` is gone. */
export function assertNuxtBuildCachesCold(root, label) {
  const remaining = readNuxtBuildCaches(root);
  if (remaining.length === 0) {
    return;
  }
  throw new Error(
    `Refusing to measure ${label} against warm build caches: ` +
      `${resolveNodeModulesDir(root)} still holds ${remaining.join(", ")}. ` +
      "That directory is shared by every measured run and by both variants, " +
      "so this would not be the same measurement.",
  );
}
