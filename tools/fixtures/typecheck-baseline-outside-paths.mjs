import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";

/**
 * Close the vue-tsc path escape unique isolation cannot see (#4461).
 *
 * Unique isolation links a fixture-local copy so `/// <reference types />`
 * stops walking into Vize's `node_modules`. `compilerOptions.paths` is a
 * different resolver: TypeScript uses the declared mapping and never consults
 * that link. Nuxt-generated configs bake an absolute-looking relative path to
 * a package *above* the fixture, so the baseline still loads Vize's Vue beside
 * the fixture's.
 *
 * Setting `paths` on the generated baseline replaces the inherited object, so
 * this copies every mapping, re-homes it to the generated config directory, and
 * only then retargets package names whose original target is outside and whose
 * fixture-local `node_modules/<name>` is already a real package. No rewrite
 * means the generated config leaves `paths` unset and inherits.
 *
 * Vize's `check --tsconfig` still reads the fixture config, so a matching
 * untracked overlay is written next to the source when a rewrite exists. Git
 * porcelain uses `--untracked-files=no`, so the overlay does not dirty the
 * fixture. The matrix typechecker prefers that overlay when it is present.
 *
 * Package-name `extends` is followed only inside the fixture. Climbing into
 * Vize's `node_modules` would load Vize's `@vue/tsconfig` and bake its
 * outside `paths` into the overlay.
 *
 * `compilerOptions.typeRoots` is the same class of hole: TypeScript searches
 * those directories instead of the fixture's `node_modules/@types`. An outside
 * `node_modules/@types` is retargeted to the fixture-local copy when it exists.
 *
 * A trailing `/*` on a package mapping is the same walk: TypeScript still
 * loads that outside tree. A mapping named `*` whose target is an outside
 * `node_modules` directory is retargeted to the fixture copy. Interior `*`
 * patterns and `#alias/*` keys are not guessed.
 */

const packageNamePattern = /^(?:@[a-z0-9][a-z0-9-._]*\/)?[a-z0-9][a-z0-9-._]*$/u;

export function rewriteOutsidePackagePaths(fixtureRoot, sourceConfigPath, configDir) {
  const root = resolve(fixtureRoot);
  const declared = winningPaths(sourceConfigPath, root);
  if (declared == null) return null;
  const rewritten = {};
  let changed = false;
  for (const [name, targets] of Object.entries(declared.paths)) {
    if (!Array.isArray(targets)) continue;
    rewritten[name] = retargetPathMapping(root, declared.dir, configDir, name, targets, () => {
      changed = true;
    });
  }
  return changed ? rewritten : null;
}

export function isolatedTsconfigOverlayPath(sourceConfigPath) {
  const dir = dirname(sourceConfigPath).replaceAll("\\", "/");
  const name = basename(sourceConfigPath);
  return dir === "." ? `.vize-isolated-${name}` : `${dir}/.vize-isolated-${name}`;
}

export function rewriteOutsideTypeRoots(fixtureRoot, sourceConfigPath, configDir) {
  const root = resolve(fixtureRoot);
  const declared = winningTypeRoots(sourceConfigPath, root);
  if (declared == null) return null;
  const rewritten = [];
  let changed = false;
  for (const entry of declared.typeRoots) {
    rewritten.push(
      retargetTypeRoot(root, declared.dir, configDir, entry, () => {
        changed = true;
      }),
    );
  }
  return changed ? rewritten : null;
}

export function writeIsolatedTsconfigOverlay(fixtureRoot, sourceConfigPath) {
  const sourcePath = resolve(sourceConfigPath);
  const configDir = dirname(sourcePath);
  const paths = rewriteOutsidePackagePaths(fixtureRoot, sourcePath, configDir);
  const typeRoots = rewriteOutsideTypeRoots(fixtureRoot, sourcePath, configDir);
  if (paths == null && typeRoots == null) return null;
  const compilerOptions = {};
  if (paths != null) compilerOptions.paths = paths;
  if (typeRoots != null) compilerOptions.typeRoots = typeRoots;
  const overlayPath = isolatedTsconfigOverlayPath(sourcePath);
  writeFileSync(
    overlayPath,
    `${JSON.stringify({ extends: `./${basename(sourcePath)}`, compilerOptions }, null, 2)}\n`,
  );
  return { path: overlayPath, paths, typeRoots };
}

function retargetPathMapping(fixtureRoot, sourceDir, configDir, name, targets, markChanged) {
  const relocated = targets.map((entry) => relocatePathEntry(sourceDir, configDir, entry));
  const packageName = packageNameFromPathMapping(name);
  const first = targets.find(
    (entry) => typeof entry === "string" && isExactOrTrailingStarPath(entry),
  );
  if (first == null) return relocated;
  const original = resolve(sourceDir, first.endsWith("/*") ? first.slice(0, -2) : first);
  if (isInside(fixtureRoot, original)) return relocated;
  let local;
  if (packageName != null) {
    local = join(fixtureRoot, "node_modules", ...packageName.split("/"));
    if (!existsSync(join(local, "package.json"))) return relocated;
  } else if (name === "*" && first.endsWith("/*") && basename(original) === "node_modules") {
    local = join(fixtureRoot, "node_modules");
    if (!existsSync(local)) return relocated;
  } else {
    return relocated;
  }
  markChanged();
  const rewritten = first.endsWith("/*")
    ? `${configRelativePath(configDir, local)}/*`
    : configRelativePath(configDir, local);
  return relocated.map((entry, index) => (index === targets.indexOf(first) ? rewritten : entry));
}

function packageNameFromPathMapping(name) {
  if (typeof name !== "string") return null;
  const base = name.endsWith("/*") ? name.slice(0, -2) : name;
  return packageNamePattern.test(base) ? base : null;
}

function isExactOrTrailingStarPath(entry) {
  return !entry.includes("*") || (entry.endsWith("/*") && !entry.slice(0, -2).includes("*"));
}

function relocatePathEntry(sourceDir, configDir, entry) {
  if (typeof entry !== "string" || entry.includes("*")) return entry;
  return configRelativePath(configDir, resolve(sourceDir, entry));
}

function retargetTypeRoot(fixtureRoot, sourceDir, configDir, entry, markChanged) {
  if (typeof entry !== "string") return entry;
  const relocated = relocatePathEntry(sourceDir, configDir, entry);
  const original = resolve(sourceDir, entry);
  if (isInside(fixtureRoot, original)) return relocated;
  if (!isTypesPackageRoot(original)) return relocated;
  const local = join(fixtureRoot, "node_modules", "@types");
  if (!existsSync(local)) return relocated;
  markChanged();
  return configRelativePath(configDir, local);
}

function winningPaths(sourceConfigPath, fixtureRoot) {
  let paths;
  let dir;
  for (const { config, dir: configDir } of [
    ...loadExtendsChain(sourceConfigPath, fixtureRoot),
  ].reverse()) {
    const candidate = config?.compilerOptions?.paths;
    if (candidate != null && typeof candidate === "object") {
      paths = candidate;
      dir = configDir;
    }
  }
  return paths == null ? null : { paths, dir };
}

function winningTypeRoots(sourceConfigPath, fixtureRoot) {
  let typeRoots;
  let dir;
  for (const { config, dir: configDir } of [
    ...loadExtendsChain(sourceConfigPath, fixtureRoot),
  ].reverse()) {
    const candidate = config?.compilerOptions?.typeRoots;
    if (Array.isArray(candidate)) {
      typeRoots = candidate;
      dir = configDir;
    }
  }
  return typeRoots == null ? null : { typeRoots, dir };
}

function loadExtendsChain(sourceConfigPath, fixtureRoot) {
  const chain = [];
  const seen = new Set();
  let current = resolve(sourceConfigPath);
  while (!seen.has(current)) {
    seen.add(current);
    const config = parseTsconfig(current);
    if (config == null) break;
    chain.push({ config, dir: dirname(current) });
    const specifiers = extendsSpecifiers(config.extends);
    let next = null;
    for (const specifier of specifiers) {
      next = resolveExtends(current, specifier, fixtureRoot);
      if (next != null) break;
    }
    if (next == null) break;
    current = next;
  }
  return chain;
}

function parseTsconfig(configPath) {
  try {
    return JSON.parse(stripJsonc(readFileSync(configPath, "utf8")));
  } catch {
    return null;
  }
}

function extendsSpecifiers(value) {
  if (typeof value === "string") return [value];
  return Array.isArray(value) ? value.filter((entry) => typeof entry === "string") : [];
}

function resolveExtends(fromConfig, specifier, fixtureRoot) {
  if (typeof specifier !== "string") return null;
  if (specifier.startsWith("./") || specifier.startsWith("../")) {
    const resolved = resolve(dirname(fromConfig), specifier);
    if (existsSync(resolved)) return resolved;
    if (!resolved.endsWith(".json") && existsSync(`${resolved}.json`)) return `${resolved}.json`;
    return null;
  }
  return resolvePackageExtends(fromConfig, specifier, fixtureRoot);
}

function resolvePackageExtends(fromConfig, specifier, fixtureRoot) {
  const parsed = splitPackageExtends(specifier);
  if (parsed == null) return null;
  const root = resolve(fixtureRoot);
  let directory = dirname(resolve(fromConfig));
  let previous = null;
  while (directory !== previous) {
    if (!(directory === root || isInside(root, directory))) break;
    const pkg = join(directory, "node_modules", ...parsed.name.split("/"));
    const file = packageExtendsFile(pkg, parsed.subpath);
    if (file != null) return file;
    if (directory === root) break;
    previous = directory;
    directory = dirname(directory);
  }
  return null;
}

function splitPackageExtends(specifier) {
  if (specifier.startsWith("@")) {
    const slash = specifier.indexOf("/");
    if (slash < 0) return null;
    const rest = specifier.slice(slash + 1);
    const second = rest.indexOf("/");
    if (second < 0) return { name: specifier, subpath: null };
    return {
      name: `${specifier.slice(0, slash + 1)}${rest.slice(0, second)}`,
      subpath: rest.slice(second + 1),
    };
  }
  if (specifier.includes(":")) return null;
  const slash = specifier.indexOf("/");
  if (slash < 0) return { name: specifier, subpath: null };
  return { name: specifier.slice(0, slash), subpath: specifier.slice(slash + 1) };
}

function packageExtendsFile(pkg, subpath) {
  if (!existsSync(join(pkg, "package.json"))) return null;
  if (subpath == null) {
    const main = join(pkg, "tsconfig.json");
    return existsSync(main) ? main : null;
  }
  const file = join(pkg, subpath);
  if (existsSync(file)) return file;
  if (!file.endsWith(".json") && existsSync(`${file}.json`)) return `${file}.json`;
  return null;
}

function isInside(root, target) {
  const path = relative(root, target);
  return path !== "" && !path.startsWith("..") && !path.startsWith("/");
}

function isTypesPackageRoot(directory) {
  const normalized = directory.replaceAll("\\", "/");
  return normalized.endsWith("/node_modules/@types");
}

function configRelativePath(from, to) {
  const path = relative(from, to).replaceAll("\\", "/");
  if (path.startsWith("/") || /^[A-Za-z]:\//u.test(path)) return path;
  return path.startsWith(".") ? path : `./${path}`;
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
