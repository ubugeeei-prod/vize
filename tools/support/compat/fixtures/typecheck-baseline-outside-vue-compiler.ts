import { existsSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";

import { resolveWithConfigDir } from "./typecheck-baseline-config-dir.ts";
import { loadTsconfigExtendsChain } from "./typecheck-baseline-extends-chain.ts";
import { packageNameFromExtendsSpecifier } from "./typecheck-baseline-isolation-package-extends.ts";
import { resolvePackageExtends } from "./typecheck-baseline-outside-paths.ts";

/**
 * Close the vue-tsc option escape unique isolation cannot see (#4461).
 *
 * `vueCompilerOptions.globalTypesPath`, `typesRoot`, and path-form `plugins`
 * are filesystem or package walks. Overlay `compilerOptions` cannot retarget
 * them. An outside path still loads Vize's `@vue/language-core` or
 * `require("../../node_modules/<pkg>")` beside the fixture Vue. Unique-link
 * answers package-name plugins; relative plugin specifiers still walk out.
 * When the fixture already has that package, this copies every winning
 * `vueCompilerOptions` key and retargets those paths. Overlay replaces the
 * object, so other keys must be preserved. Plugin lists are concatenated
 * across relative extends, matching vue-tsc.
 */

const pathOptionKeys = ["globalTypesPath", "typesRoot"];

/** A `vueCompilerOptions.plugins` entry as a fixture config spells it. */
type VuePlugin = string | { readonly name?: unknown } | null | undefined;

interface PluginEntry {
  readonly plugin: VuePlugin;
  readonly dir: string;
}

interface Retargeted {
  readonly value: unknown;
  readonly changed: boolean;
}

export function rewriteOutsideVueCompilerOptions(
  fixtureRoot: string,
  sourceConfigPath: string,
  configDir: string,
): Record<string, unknown> | null {
  const root = resolve(fixtureRoot);
  const declared = winningVueCompilerOptions(sourceConfigPath, root);
  if (declared == null) return null;
  const rewritten: Record<string, unknown> = { ...declared.options };
  let changed = false;
  for (const key of pathOptionKeys) {
    const entry = declared.options[key];
    if (typeof entry !== "string") continue;
    const retargeted = retargetVueCompilerPath(root, declared.dirs[key], configDir, entry);
    if (retargeted == null) continue;
    rewritten[key] = retargeted;
    changed = true;
  }
  const plugins: unknown[] = [];
  for (const { plugin, dir } of declared.plugins) {
    const next = retargetPluginEntry(root, dir, configDir, plugin);
    if (next.changed) changed = true;
    plugins.push(next.value);
  }
  if (plugins.length > 0) rewritten.plugins = plugins;
  return changed ? rewritten : null;
}

function retargetPluginEntry(
  fixtureRoot: string,
  sourceDir: string,
  configDir: string,
  plugin: VuePlugin,
): Retargeted {
  if (typeof plugin === "string") {
    return retargetPluginName(fixtureRoot, sourceDir, configDir, plugin, plugin);
  }
  if (Array.isArray(plugin) && typeof plugin[0] === "string") {
    const next = retargetPluginName(fixtureRoot, sourceDir, configDir, plugin[0], plugin);
    if (!next.changed) return next;
    return { value: [next.value, ...plugin.slice(1)], changed: true };
  }
  if (plugin != null && typeof plugin.name === "string") {
    const next = retargetPluginName(fixtureRoot, sourceDir, configDir, plugin.name, plugin);
    if (!next.changed) return next;
    return { value: { ...plugin, name: next.value }, changed: true };
  }
  return { value: plugin, changed: false };
}

function retargetPluginName(
  fixtureRoot: string,
  sourceDir: string,
  configDir: string,
  name: string,
  fallback: unknown,
): Retargeted {
  const normalized = name.replaceAll("\\", "/");
  if (!isPathSpecifier(normalized)) return { value: fallback, changed: false };
  const retargeted = retargetVueCompilerPath(fixtureRoot, sourceDir, configDir, normalized);
  return retargeted == null
    ? { value: fallback, changed: false }
    : { value: retargeted, changed: true };
}

function retargetVueCompilerPath(
  fixtureRoot: string,
  sourceDir: string,
  configDir: string,
  entry: string,
): string | null {
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

function winningVueCompilerOptions(
  sourceConfigPath: string,
  fixtureRoot: string,
): {
  options: Record<string, unknown>;
  dirs: Record<string, string>;
  plugins: PluginEntry[];
} | null {
  const options: Record<string, unknown> = {};
  const dirs: Record<string, string> = {};
  const plugins: PluginEntry[] = [];
  for (const { config, dir } of [
    ...loadTsconfigExtendsChain(
      sourceConfigPath,
      (fromConfig, specifier) =>
        resolveRelativeExtends(fromConfig, specifier, fixtureRoot) ??
        resolvePackageExtends(fromConfig, specifier, fixtureRoot),
    ),
  ].reverse()) {
    const current = config?.vueCompilerOptions as Readonly<Record<string, unknown>> | undefined;
    if (current == null || typeof current !== "object" || Array.isArray(current)) continue;
    for (const [key, value] of Object.entries(current)) {
      if (key === "plugins") continue;
      options[key] = value;
      dirs[key] = dir;
    }
    if (Array.isArray(current.plugins)) {
      for (const plugin of current.plugins as readonly VuePlugin[]) plugins.push({ plugin, dir });
    }
  }
  return Object.keys(options).length === 0 && plugins.length === 0
    ? null
    : { options, dirs, plugins };
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
