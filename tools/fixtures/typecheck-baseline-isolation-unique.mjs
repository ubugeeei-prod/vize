import {
  existsSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  symlinkSync,
} from "node:fs";
import { dirname, join, relative, resolve } from "node:path";

import { readDeclaredPackagePaths } from "./typecheck-baseline-isolation.mjs";
import { ancestorPackagePath } from "./typecheck-baseline-isolation-package-extends.mjs";

/**
 * Close the remaining type-reference escape when Nuxt (or another generator)
 * bakes `compilerOptions.paths` to a package *outside* the fixture (#4461).
 *
 * `typecheck-baseline-isolation.mjs` will not materialize an outside target —
 * that would import the contamination. If the fixture's own pnpm store holds
 * exactly one copy of that name, this links that copy instead of walking out to
 * Vize's. Several copies are different Vue peer suffixes: this then picks the
 * copy whose pnpm id matches the fixture's own `vue` version, and still does
 * not guess when zero or several copies match.
 *
 * Declared names come from the same `extends` / `references` walk isolation
 * uses, so a check tsconfig that only extends the generated app config still
 * sees the outside mapping (reka-ui's `tsconfig.check.json`). Package-name
 * `extends` specifiers, `compilerOptions.types` entries, plugin packages, and
 * `jsxImportSource` are included as ancestor targets so this can link the
 * fixture's own `@vue/tsconfig`, `vite`, `@types/node`, language plugin, or
 * `vue` JSX runtime before TypeScript climbs into Vize.
 */

const vueRuntimePackages = ["@vue/runtime-core", "@vue/runtime-dom", "vue", "vue-router"];

export function isolateUniqueLocalTypePackages(fixtureRoot, sourceConfigPath) {
  const root = resolve(fixtureRoot);
  return isolateUniqueDeclaredLocalTypePackages(
    root,
    readDeclaredPackagePaths(root, sourceConfigPath),
  );
}

export function isolateUniqueVueRuntimePackages(fixtureRoot) {
  const root = resolve(fixtureRoot);
  const declared = new Map();
  for (const name of vueRuntimePackages) {
    const ancestor = ancestorPackagePath(root, name);
    if (ancestor == null) continue;
    declared.set(name, ancestor);
  }
  return isolateUniqueDeclaredLocalTypePackages(root, declared);
}

function isolateUniqueDeclaredLocalTypePackages(root, declared) {
  if (declared.size === 0) return [];
  const reachable = collectAncestorPackageNames(root);
  const shadowed = [];
  for (const [name, target] of [...declared].sort(([left], [right]) => compare(left, right))) {
    if (!reachable.has(name)) continue;
    const link = join(root, "node_modules", name);
    if (existsSync(link) || isDanglingLink(link)) continue;
    if (isPackageDirectory(target) && isInside(root, target)) continue;
    const copies = findLocalCopies(root, name);
    const local = selectLocalCopy(root, name, copies);
    if (local == null) continue;
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

function selectLocalCopy(fixtureRoot, name, copies) {
  if (copies.length === 1) return copies[0];
  if (copies.length === 0) return null;
  const vueVersion = readFixtureVueVersion(fixtureRoot);
  if (vueVersion == null) return null;
  const matched = copies.filter((copy) => copyMatchesVue(fixtureRoot, name, copy, vueVersion));
  return matched.length === 1 ? matched[0] : null;
}

function readFixtureVueVersion(fixtureRoot) {
  const hoisted = readPackageVersion(join(fixtureRoot, "node_modules", "vue"));
  if (hoisted != null) return hoisted;
  const copies = findLocalCopies(fixtureRoot, "vue");
  if (copies.length !== 1) return null;
  return readPackageVersion(copies[0]);
}

function readPackageVersion(packageDir) {
  try {
    const pkg = JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8"));
    return typeof pkg.version === "string" && pkg.version !== "" ? pkg.version : null;
  } catch {
    return null;
  }
}

function copyMatchesVue(fixtureRoot, name, packageDir, vueVersion) {
  const store = join(fixtureRoot, "node_modules", ".pnpm");
  const relativePath = relative(store, packageDir);
  if (relativePath.startsWith("..") || relativePath.startsWith("/")) return false;
  const id = relativePath.split(/[\\/]/)[0];
  const marker = name === "vue" ? `vue@${vueVersion}` : `_vue@${vueVersion}`;
  const index = id.indexOf(marker);
  if (index === -1) return false;
  if (name === "vue" && index !== 0) return false;
  const end = index + marker.length;
  return end === id.length || id[end] === "_" || id[end] === "(";
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
