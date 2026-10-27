import { existsSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";

import { resolveWithConfigDir } from "./typecheck-baseline-config-dir.ts";
import { loadTsconfigExtendsChain } from "./typecheck-baseline-extends-chain.ts";
import { resolvePackageExtends } from "./typecheck-baseline-outside-paths.ts";

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

export function rewriteOutsideCompilerPlugins(
  fixtureRoot: string,
  sourceConfigPath: string,
  configDir: string,
): unknown[] | null {
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

function retargetCompilerPlugin(
  fixtureRoot: string,
  sourceDir: string,
  configDir: string,
  plugin: { readonly name?: unknown } | null,
): { value: unknown; changed: boolean } {
  if (plugin == null || typeof plugin.name !== "string") {
    return { value: plugin, changed: false };
  }
  if (!isPathSpecifier(plugin.name)) return { value: plugin, changed: false };
  const retargeted = retargetCompilerPluginPath(fixtureRoot, sourceDir, configDir, plugin.name);
  if (retargeted == null) return { value: plugin, changed: false };
  return { value: { ...plugin, name: retargeted }, changed: true };
}

function retargetCompilerPluginPath(
  fixtureRoot: string,
  sourceDir: string,
  configDir: string,
  entry: string,
): string | null {
  const original = resolveWithConfigDir(sourceDir, sourceDir, entry);
  if (isInside(fixtureRoot, original)) return null;
  const owned = owningNodeModulePackage(original);
  if (owned == null) return null;
  return localPackageTarget(fixtureRoot, configDir, owned.name, relative(owned.root, original));
}

function localPackageTarget(
  fixtureRoot: string,
  configDir: string,
  name: string,
  subpath: unknown,
): string | null {
  const local = join(fixtureRoot, "node_modules", ...name.split("/"));
  if (!existsSync(join(local, "package.json"))) return null;
  const nested = typeof subpath === "string" ? subpath.replaceAll("\\", "/") : "";
  if (nested.startsWith("..") || nested.startsWith("/")) return null;
  const target = nested === "" ? local : join(local, nested);
  if (nested !== "" && !existsSync(target)) return null;
  return configRelativePath(configDir, target);
}

function isPathSpecifier(entry: string): boolean {
  return (
    entry.startsWith("./") ||
    entry.startsWith("../") ||
    entry.startsWith("/") ||
    entry.includes("/node_modules/") ||
    entry.includes("${configDir}")
  );
}

function winningCompilerPlugins(
  sourceConfigPath: string,
  fixtureRoot: string,
): { plugin: { readonly name?: unknown } | null; dir: string }[] {
  const plugins: { plugin: { readonly name?: unknown } | null; dir: string }[] = [];
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

function resolveRelativeExtends(
  fromConfig: string,
  specifier: unknown,
  fixtureRoot: string,
): string | null {
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

function owningNodeModulePackage(resolvedPath: string): OwnedPackage | null {
  let directory = resolvedPath;
  let previous: string | null = null;
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

/** A package directory directly under a `node_modules` folder. */
interface OwnedPackage {
  readonly name: string;
  readonly root: string;
}

function nodeModulePackageName(packageRoot: string): OwnedPackage | null {
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

function isInside(root: string, target: string): boolean {
  const path = relative(root, target);
  return path !== "" && !path.startsWith("..") && !path.startsWith("/");
}

function configRelativePath(from: string, to: string): string {
  const path = relative(from, to).replaceAll("\\", "/");
  if (path.startsWith("/") || /^[A-Za-z]:\//u.test(path)) return path;
  return path.startsWith(".") ? path : `./${path}`;
}
