import { existsSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";

import { resolveWithConfigDir } from "./typecheck-baseline-config-dir.mjs";
import { loadTsconfigExtendsChain } from "./typecheck-baseline-extends-chain.mjs";
import { packageNameFromExtendsSpecifier } from "./typecheck-baseline-isolation-package-extends.mjs";
import { resolvePackageExtends } from "./typecheck-baseline-outside-paths.mjs";

/**
 * Close the vue-tsc option escape unique isolation cannot see (#4461).
 *
 * `vueCompilerOptions.globalTypesPath` and `typesRoot` are filesystem or
 * package walks. Overlay `compilerOptions` cannot retarget them. An outside
 * path still loads Vize's `@vue/language-core` beside the fixture Vue.
 * When the fixture already has that package, this copies every winning
 * `vueCompilerOptions` key and retargets only those two paths. Overlay
 * replaces the object, so other keys must be preserved.
 */

const pathOptionKeys = ["globalTypesPath", "typesRoot"];

export function rewriteOutsideVueCompilerOptions(fixtureRoot, sourceConfigPath, configDir) {
  const root = resolve(fixtureRoot);
  const declared = winningVueCompilerOptions(sourceConfigPath, root);
  if (declared == null) return null;
  const rewritten = { ...declared.options };
  let changed = false;
  for (const key of pathOptionKeys) {
    const entry = declared.options[key];
    if (typeof entry !== "string") continue;
    const retargeted = retargetVueCompilerPath(root, declared.dirs[key], configDir, entry);
    if (retargeted == null) continue;
    rewritten[key] = retargeted;
    changed = true;
  }
  return changed ? rewritten : null;
}

function retargetVueCompilerPath(fixtureRoot, sourceDir, configDir, entry) {
  if (isPathSpecifier(entry)) {
    const original = resolveWithConfigDir(sourceDir, sourceDir, entry);
    if (isInside(fixtureRoot, original)) return null;
    const owned = owningNodeModulePackage(original);
    if (owned == null) return null;
    return localPackageTarget(fixtureRoot, configDir, owned.name, relative(owned.root, original));
  }
  const name = packageNameFromExtendsSpecifier(entry);
  if (name == null) return null;
  const rest = entry.startsWith(`${name}/`) ? entry.slice(name.length + 1) : "";
  return localPackageTarget(fixtureRoot, configDir, name, rest);
}

function localPackageTarget(fixtureRoot, configDir, name, subpath) {
  const local = join(fixtureRoot, "node_modules", ...name.split("/"));
  if (!existsSync(join(local, "package.json"))) return null;
  const nested = typeof subpath === "string" ? subpath.replaceAll("\\", "/") : "";
  if (nested.startsWith("..") || nested.startsWith("/")) return null;
  const target = nested === "" ? local : join(local, nested);
  if (nested !== "" && !existsSync(target)) return null;
  return configRelativePath(configDir, target);
}

function isPathSpecifier(entry) {
  return (
    entry.startsWith("./") ||
    entry.startsWith("../") ||
    entry.startsWith("/") ||
    entry.includes("/node_modules/") ||
    entry.includes("${configDir}")
  );
}

function winningVueCompilerOptions(sourceConfigPath, fixtureRoot) {
  const options = {};
  const dirs = {};
  for (const { config, dir } of [
    ...loadTsconfigExtendsChain(
      sourceConfigPath,
      (fromConfig, specifier) =>
        resolveRelativeExtends(fromConfig, specifier, fixtureRoot) ??
        resolvePackageExtends(fromConfig, specifier, fixtureRoot),
    ),
  ].reverse()) {
    const current = config?.vueCompilerOptions;
    if (current == null || typeof current !== "object" || Array.isArray(current)) continue;
    for (const [key, value] of Object.entries(current)) {
      options[key] = value;
      dirs[key] = dir;
    }
  }
  return Object.keys(options).length === 0 ? null : { options, dirs };
}

function resolveRelativeExtends(fromConfig, specifier, fixtureRoot) {
  if (typeof specifier !== "string") return null;
  if (!(specifier.startsWith("./") || specifier.startsWith("../"))) return null;
  const resolved = resolve(dirname(fromConfig), specifier);
  const file = existsSync(resolved)
    ? resolved
    : !resolved.endsWith(".json") && existsSync(`${resolved}.json`)
      ? `${resolved}.json`
      : null;
  if (file == null || !isInside(fixtureRoot, file)) return null;
  return file;
}

function owningNodeModulePackage(resolvedPath) {
  let directory = resolvedPath;
  let previous = null;
  while (directory !== previous) {
    if (existsSync(join(directory, "package.json"))) {
      const owned = nodeModulePackageName(directory);
      if (owned != null) return owned;
    }
    previous = directory;
    directory = dirname(directory);
  }
  return null;
}

function nodeModulePackageName(packageRoot) {
  const parent = dirname(packageRoot);
  const grandparent = dirname(parent);
  if (basename(parent) === "node_modules") {
    return { name: basename(packageRoot), root: packageRoot };
  }
  if (basename(grandparent) === "node_modules" && basename(parent).startsWith("@")) {
    return {
      name: `${basename(parent)}/${basename(packageRoot)}`,
      root: packageRoot,
    };
  }
  return null;
}

function isInside(root, target) {
  const path = relative(root, target);
  return path !== "" && !path.startsWith("..") && !path.startsWith("/");
}

function configRelativePath(from, to) {
  const path = relative(from, to).replaceAll("\\", "/");
  if (path.startsWith("/") || /^[A-Za-z]:\//u.test(path)) return path;
  return path.startsWith(".") ? path : `./${path}`;
}
