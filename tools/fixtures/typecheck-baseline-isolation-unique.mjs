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

/**
 * Close the remaining type-reference escape when Nuxt (or another generator)
 * bakes `compilerOptions.paths` to a package *outside* the fixture (#4461).
 *
 * `typecheck-baseline-isolation.mjs` will not materialize an outside target —
 * that would import the contamination. If the fixture's own pnpm store holds
 * exactly one copy of that name, this links that copy instead of walking out to
 * Vize's. Multiple copies are different peer suffixes; this does not guess
 * which Vue they were built against.
 */

const packageNamePattern = /^(?:@[a-z0-9][a-z0-9-._]*\/)?[a-z0-9][a-z0-9-._]*$/u;

export function isolateUniqueLocalTypePackages(fixtureRoot, sourceConfigPath) {
  const root = resolve(fixtureRoot);
  const declared = readOwnPackagePaths(sourceConfigPath);
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

function readOwnPackagePaths(sourceConfigPath) {
  const declared = new Map();
  const config = parseTsconfig(sourceConfigPath);
  const paths = config?.compilerOptions?.paths;
  if (paths == null || typeof paths !== "object") return declared;
  const configDir = dirname(resolve(sourceConfigPath));
  for (const [name, targets] of Object.entries(paths)) {
    if (!packageNamePattern.test(name) || !Array.isArray(targets)) continue;
    const first = targets.find((entry) => typeof entry === "string" && !entry.includes("*"));
    if (first == null) continue;
    declared.set(name, resolve(configDir, first));
  }
  return declared;
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

function parseTsconfig(configPath) {
  try {
    return JSON.parse(stripJsonc(readFileSync(configPath, "utf8")));
  } catch {
    return null;
  }
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

function stripJsonc(text) {
  let out = "";
  let inString = false;
  for (let i = 0; i < text.length; i += 1) {
    const ch = text[i];
    if (inString) {
      out += ch;
      if (ch === "\\") {
        out += text[i + 1] ?? "";
        i += 1;
      } else if (ch === '"') {
        inString = false;
      }
      continue;
    }
    if (ch === '"') {
      inString = true;
      out += ch;
      continue;
    }
    if (ch === "/" && text[i + 1] === "/") {
      while (i < text.length && text[i] !== "\n") i += 1;
      out += "\n";
      continue;
    }
    if (ch === "/" && text[i + 1] === "*") {
      i += 2;
      while (i + 1 < text.length && !(text[i] === "*" && text[i + 1] === "/")) i += 1;
      i += 1;
      continue;
    }
    out += ch;
  }
  return out.replace(/,(\s*[}\]])/g, "$1");
}
