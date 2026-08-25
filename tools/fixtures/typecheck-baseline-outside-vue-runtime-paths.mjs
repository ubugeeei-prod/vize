import { existsSync } from "node:fs";
import { join, relative, resolve } from "node:path";

/**
 * Pin the Vue runtime vue-tsc injects onto the fixture copy (#4461).
 *
 * Unique-link answers climbing from fixture files. vue-tsc's language-core
 * still resolves `@vue/runtime-dom` from Vize's pnpm store, so elk and reka-ui
 * load two Vue identities in one program. Overlay `paths` apply to the whole
 * program. When the fixture already has that package, this writes the mapping.
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
    const local = join(root, "node_modules", ...name.split("/"));
    if (!existsSync(join(local, "package.json"))) continue;
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
