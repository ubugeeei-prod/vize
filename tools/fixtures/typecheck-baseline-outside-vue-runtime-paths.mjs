import { existsSync, readdirSync } from "node:fs";
import { join, relative, resolve } from "node:path";

/**
 * Pin the Vue runtime vue-tsc injects onto the fixture copy (#4461).
 *
 * Unique-link answers climbing from fixture files. vue-tsc's language-core
 * still resolves `@vue/runtime-dom` from Vize's pnpm store, so elk and reka-ui
 * load two Vue identities in one program. Overlay `paths` apply to the whole
 * program. When the fixture already has a direct package link, or exactly one
 * pnpm virtual-store copy for a package without that link, this writes the
 * mapping.
 * Overlay replaces `compilerOptions.paths`, so these keys must win over any
 * outside mapping the source tsconfig still carries.
 */

const vueRuntimePackages = ["@vue/runtime-core", "@vue/runtime-dom", "vue"];

export function rewriteLocalVueRuntimePaths(fixtureRoot, configDir) {
  const root = resolve(fixtureRoot);
  const from = resolve(configDir);
  const rewritten = {};
  let changed = false;
  for (const name of vueRuntimePackages) {
    const local = localPackageRoot(root, name);
    if (local == null) continue;
    rewritten[name] = [configRelativePath(from, local)];
    changed = true;
  }
  return changed ? rewritten : null;
}

export function mergeLocalVueRuntimePaths(paths, runtimePaths) {
  if (runtimePaths == null) return paths;
  if (paths == null) return runtimePaths;
  return { ...paths, ...runtimePaths };
}

function configRelativePath(from, to) {
  const path = relative(from, to).replaceAll("\\", "/");
  if (path.startsWith("/") || /^[A-Za-z]:\//u.test(path)) return path;
  return path.startsWith(".") ? path : `./${path}`;
}

function localPackageRoot(root, name) {
  const linked = join(root, "node_modules", ...name.split("/"));
  if (existsSync(join(linked, "package.json"))) return linked;
  return uniquePnpmStorePackageRoot(root, name);
}

function uniquePnpmStorePackageRoot(root, name) {
  const store = join(root, "node_modules", ".pnpm");
  if (!existsSync(store)) return null;
  const matches = [];
  for (const entry of readdirSync(store, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const candidate = join(store, entry.name, "node_modules", ...name.split("/"));
    if (existsSync(join(candidate, "package.json"))) matches.push(candidate);
  }
  matches.sort(codeUnitOrder);
  return matches.length === 1 ? matches[0] : null;
}

function codeUnitOrder(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
