import { existsSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";

import { resolveWithConfigDir } from "./typecheck-baseline-config-dir.mjs";
import { loadTsconfigExtendsChain } from "./typecheck-baseline-extends-chain.mjs";
import { resolvePackageExtends } from "./typecheck-baseline-outside-paths.mjs";

/**
 * Close the TypeScript plugin require unique isolation cannot retarget (#4461).
 *
 * Unique-link answers `require("typescript-plugin-css-modules")`. Generated
 * configs also write `{ "name": "../../node_modules/<pkg>" }`, which loads
 * from outside the fixture. Overlay `paths` cannot rewrite
 * `compilerOptions.plugins`. When the fixture already has that package,
 * retarget path-form `{ name }` specifiers. Package-name plugins stay with
 * unique-link. Plugin lists are concatenated across relative extends,
 * matching tsc.
 */

export function rewriteOutsideCompilerPlugins(fixtureRoot, sourceConfigPath, configDir) {
  const root = resolve(fixtureRoot);
  const declared = winningCompilerPlugins(sourceConfigPath, root);
  if (declared.length === 0) return null;
  let changed = false;
  const plugins = declared.map(({ plugin, dir }) => {
    const next = retargetCompilerPlugin(root, dir, configDir, plugin);
    if (next.changed) changed = true;
    return next.value;
  });
  return changed ? plugins : null;
}

function retargetCompilerPlugin(fixtureRoot, sourceDir, configDir, plugin) {
  if (plugin == null || typeof plugin.name !== "string") {
    return { value: plugin, changed: false };
  }
  if (!isPathSpecifier(plugin.name)) return { value: plugin, changed: false };
  const retargeted = retargetCompilerPluginPath(fixtureRoot, sourceDir, configDir, plugin.name);
  if (retargeted == null) return { value: plugin, changed: false };
  return { value: { ...plugin, name: retargeted }, changed: true };
}

function retargetCompilerPluginPath(fixtureRoot, sourceDir, configDir, entry) {
  const original = resolveWithConfigDir(sourceDir, sourceDir, entry);
  if (isInside(fixtureRoot, original)) return null;
  const owned = owningNodeModulePackage(original);
  if (owned == null) return null;
  return localPackageTarget(fixtureRoot, configDir, owned.name, relative(owned.root, original));
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

function winningCompilerPlugins(sourceConfigPath, fixtureRoot) {
  const plugins = [];
  for (const { config, dir } of [
    ...loadTsconfigExtendsChain(
      sourceConfigPath,
      (fromConfig, specifier) =>
        resolveRelativeExtends(fromConfig, specifier, fixtureRoot) ??
        resolvePackageExtends(fromConfig, specifier, fixtureRoot),
    ),
  ].reverse()) {
    const current = config?.compilerOptions?.plugins;
    if (!Array.isArray(current)) continue;
    for (const plugin of current) plugins.push({ plugin, dir });
  }
  return plugins;
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
