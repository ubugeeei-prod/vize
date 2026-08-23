import { existsSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";

import { resolveWithConfigDir } from "./typecheck-baseline-config-dir.mjs";
import { loadTsconfigExtendsChain } from "./typecheck-baseline-extends-chain.mjs";
import {
  isolatedOverlayBaseUrl,
  isolatedTsconfigOverlayPath,
  pathMappingRoot,
  resolvePackageExtends,
} from "./typecheck-baseline-outside-paths.mjs";

/**
 * Close the remaining `compilerOptions.paths` escape for keys that are not
 * package names (#4461). Unique isolation and the package-name overlay only
 * retarget `vue-router`. Nuxt still writes `#app` / `#imports` onto files
 * *above* the fixture, so vue-tsc loads Vize's Nuxt and then Vize's Vue.
 *
 * A mapping is retargeted only when its target sits under an outside
 * `node_modules/<name>` (or `@scope/name`) directory and the fixture already
 * has that package. Interior `*` patterns are not guessed.
 *
 * Package-name `extends` is followed only inside the fixture, matching the
 * package-name overlay. Climbing into Vize would load Vize's `#app` mappings.
 */

const packageNamePattern = /^(?:@[a-z0-9][a-z0-9-._]*\/)?[a-z0-9][a-z0-9-._]*$/u;

export function rewriteOutsideAliasPaths(fixtureRoot, sourceConfigPath, configDir) {
  const root = resolve(fixtureRoot);
  const declared = winningPaths(sourceConfigPath, root);
  if (declared == null) return null;
  const mapping = pathMappingRoot(sourceConfigPath, root, declared.dir);
  const rewritten = {};
  let changed = false;
  for (const [name, targets] of Object.entries(declared.paths)) {
    if (!Array.isArray(targets)) continue;
    rewritten[name] = retargetAliasMapping(
      root,
      mapping,
      declared.dir,
      configDir,
      name,
      targets,
      () => {
        changed = true;
      },
    );
  }
  return changed ? rewritten : null;
}

export function mergePathRewrites(packages, aliases) {
  if (packages == null) return aliases;
  if (aliases == null) return packages;
  const merged = { ...packages };
  for (const [name, targets] of Object.entries(aliases)) {
    if (!isPackageMappingName(name)) merged[name] = targets;
  }
  return merged;
}

export function applyIsolatedAliasOverlay(fixtureRoot, sourceConfigPath, overlay) {
  const sourcePath = resolve(sourceConfigPath);
  const aliases = rewriteOutsideAliasPaths(fixtureRoot, sourcePath, dirname(sourcePath));
  if (aliases == null) return overlay ?? null;
  const paths = mergePathRewrites(overlay?.paths ?? null, aliases);
  const typeRoots = overlay?.typeRoots ?? null;
  const overlayPath = overlay?.path ?? isolatedTsconfigOverlayPath(sourcePath);
  const compilerOptions = { paths };
  if (typeRoots != null) compilerOptions.typeRoots = typeRoots;
  if (isolatedOverlayBaseUrl(sourcePath, fixtureRoot) != null) compilerOptions.baseUrl = ".";
  writeFileSync(
    overlayPath,
    `${JSON.stringify({ extends: `./${basename(sourcePath)}`, compilerOptions }, null, 2)}\n`,
  );
  return { path: overlayPath, paths, typeRoots };
}

function retargetAliasMapping(
  fixtureRoot,
  sourceDir,
  tsconfigDir,
  configDir,
  name,
  targets,
  markChanged,
) {
  const relocated = targets.map((entry) =>
    relocatePathEntry(sourceDir, tsconfigDir, configDir, entry),
  );
  if (isPackageMappingName(name)) return relocated;
  const first = targets.find(
    (entry) => typeof entry === "string" && isExactOrTrailingStarPath(entry),
  );
  if (first == null) return relocated;
  const star = first.endsWith("/*");
  const original = resolveWithConfigDir(sourceDir, tsconfigDir, star ? first.slice(0, -2) : first);
  if (isInside(fixtureRoot, original)) return relocated;
  const owned = owningNodeModulePackage(original);
  if (owned == null || isInside(fixtureRoot, owned.root)) return relocated;
  const local = join(fixtureRoot, "node_modules", ...owned.name.split("/"));
  if (!existsSync(join(local, "package.json"))) return relocated;
  const subpath = relative(owned.root, original).replaceAll("\\", "/");
  if (subpath.startsWith("..") || subpath.startsWith("/")) return relocated;
  const localTarget = subpath === "" ? local : join(local, subpath);
  markChanged();
  const rewritten = star
    ? `${configRelativePath(configDir, localTarget)}/*`
    : configRelativePath(configDir, localTarget);
  return relocated.map((entry, index) => (index === targets.indexOf(first) ? rewritten : entry));
}

function isPackageMappingName(name) {
  if (typeof name !== "string") return false;
  const base = name.endsWith("/*") ? name.slice(0, -2) : name;
  return packageNamePattern.test(base);
}

function isExactOrTrailingStarPath(entry) {
  return !entry.includes("*") || (entry.endsWith("/*") && !entry.slice(0, -2).includes("*"));
}

function relocatePathEntry(sourceDir, tsconfigDir, configDir, entry) {
  if (typeof entry !== "string") return entry;
  if (!entry.includes("*")) {
    return configRelativePath(configDir, resolveWithConfigDir(sourceDir, tsconfigDir, entry));
  }
  if (entry.endsWith("/*") && !entry.slice(0, -2).includes("*")) {
    return `${configRelativePath(
      configDir,
      resolveWithConfigDir(sourceDir, tsconfigDir, entry.slice(0, -2)),
    )}/*`;
  }
  return entry;
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

function winningPaths(sourceConfigPath, fixtureRoot) {
  let paths;
  let dir;
  for (const { config, dir: configDir } of [
    ...loadTsconfigExtendsChain(
      sourceConfigPath,
      (fromConfig, specifier) =>
        resolveRelativeExtends(fromConfig, specifier, fixtureRoot) ??
        resolvePackageExtends(fromConfig, specifier, fixtureRoot),
    ),
  ].reverse()) {
    const candidate = config?.compilerOptions?.paths;
    if (candidate != null && typeof candidate === "object") {
      paths = candidate;
      dir = configDir;
    }
  }
  return paths == null ? null : { paths, dir };
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

function isInside(root, target) {
  const path = relative(root, target);
  return path !== "" && !path.startsWith("..") && !path.startsWith("/");
}

function configRelativePath(from, to) {
  const path = relative(from, to).replaceAll("\\", "/");
  if (path.startsWith("/") || /^[A-Za-z]:\//u.test(path)) return path;
  return path.startsWith(".") ? path : `./${path}`;
}
