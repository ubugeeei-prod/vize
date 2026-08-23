import { existsSync, lstatSync, mkdirSync, readdirSync, realpathSync, symlinkSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";

import { readDeclaredPackagePaths } from "./typecheck-baseline-isolation.mjs";

/**
 * Close the remaining type-reference escape when Nuxt (or another generator)
 * bakes `compilerOptions.paths` to a package *outside* the fixture (#4461).
 *
 * `typecheck-baseline-isolation.mjs` will not materialize an outside target —
 * that would import the contamination. If the fixture's own pnpm store holds
 * exactly one copy of that name, this links that copy instead of walking out to
 * Vize's. Multiple copies are different peer suffixes; this does not guess
 * which Vue they were built against.
 *
 * Declared names come from the same `extends` / `references` walk isolation
 * uses, so a check tsconfig that only extends the generated app config still
 * sees the outside mapping (reka-ui's `tsconfig.check.json`).
 */

export function isolateUniqueLocalTypePackages(fixtureRoot, sourceConfigPath) {
  const root = resolve(fixtureRoot);
  const declared = readDeclaredPackagePaths(root, sourceConfigPath);
  if (declared.size === 0) return [];
  const reachable = collectAncestorPackageNames(root);
  const shadowed = [];
  for (const [name, target] of [...declared].sort(([left], [right]) => compare(left, right))) {
    if (!reachable.has(name)) continue;
    const link = join(root, "node_modules", name);
    if (existsSync(link) || isDanglingLink(link)) continue;
    if (isPackageDirectory(target) && isInside(root, target)) continue;
    const copies = findLocalCopies(root, name);
    if (copies.length !== 1) continue;
    const local = copies[0];
    mkdirSync(dirname(link), { recursive: true });
    symlinkSync(relative(dirname(link), local), link);
    shadowed.push({ name, target: relative(root, local).replaceAll("\\", "/") });
  }
  return shadowed;
}

function findLocalCopies(fixtureRoot, name) {
  const copies = new Map();
  const store = join(fixtureRoot, "node_modules", ".pnpm");
  const segments = name.split("/");
  for (const entry of readDirectory(store)) {
    if (entry.name.startsWith(".")) continue;
    const packageDir = join(store, entry.name, "node_modules", ...segments);
    if (!isPackageDirectory(packageDir) || !isInside(fixtureRoot, packageDir)) continue;
    let real;
    try {
      real = realpathSync(packageDir);
    } catch {
      continue;
    }
    if (!isInside(fixtureRoot, real)) continue;
    copies.set(real, packageDir);
  }
  return [...copies.values()].sort(compare);
}

function collectAncestorPackageNames(fixtureRoot) {
  const names = new Set();
  let directory = dirname(fixtureRoot);
  let previous = null;
  while (directory !== previous) {
    collectPackageNames(join(directory, "node_modules"), names);
    previous = directory;
    directory = dirname(directory);
  }
  return names;
}

function collectPackageNames(nodeModules, names) {
  for (const entry of readDirectory(nodeModules)) {
    if (entry.name.startsWith(".")) continue;
    if (!entry.name.startsWith("@")) {
      names.add(entry.name);
      continue;
    }
    for (const scoped of readDirectory(join(nodeModules, entry.name))) {
      names.add(`${entry.name}/${scoped.name}`);
    }
  }
}

function readDirectory(directory) {
  try {
    return readdirSync(directory, { withFileTypes: true });
  } catch {
    return [];
  }
}

function isPackageDirectory(target) {
  return existsSync(join(target, "package.json"));
}

function isInside(root, target) {
  const path = relative(root, target);
  return path !== "" && !path.startsWith("..") && !path.startsWith("/");
}

function isDanglingLink(link) {
  try {
    lstatSync(link);
    return true;
  } catch {
    return false;
  }
}

function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
