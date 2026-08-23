import { existsSync, readFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";

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
 */

const packageNamePattern = /^(?:@[a-z0-9][a-z0-9-._]*\/)?[a-z0-9][a-z0-9-._]*$/u;

export function rewriteOutsidePackagePaths(fixtureRoot, sourceConfigPath, configDir) {
  const root = resolve(fixtureRoot);
  const declared = winningPaths(sourceConfigPath);
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

function retargetPathMapping(fixtureRoot, sourceDir, configDir, name, targets, markChanged) {
  const relocated = targets.map((entry) => relocatePathEntry(sourceDir, configDir, entry));
  if (!packageNamePattern.test(name)) return relocated;
  const first = targets.find((entry) => typeof entry === "string" && !entry.includes("*"));
  if (first == null) return relocated;
  const original = resolve(sourceDir, first);
  if (isInside(fixtureRoot, original)) return relocated;
  const local = join(fixtureRoot, "node_modules", ...name.split("/"));
  if (!existsSync(join(local, "package.json"))) return relocated;
  markChanged();
  return relocated.map((entry, index) =>
    index === targets.indexOf(first) ? configRelativePath(configDir, local) : entry,
  );
}

function relocatePathEntry(sourceDir, configDir, entry) {
  if (typeof entry !== "string" || entry.includes("*")) return entry;
  return configRelativePath(configDir, resolve(sourceDir, entry));
}

function winningPaths(sourceConfigPath) {
  let paths;
  let dir;
  for (const { config, dir: configDir } of [...loadExtendsChain(sourceConfigPath)].reverse()) {
    const candidate = config?.compilerOptions?.paths;
    if (candidate != null && typeof candidate === "object") {
      paths = candidate;
      dir = configDir;
    }
  }
  return paths == null ? null : { paths, dir };
}

function loadExtendsChain(sourceConfigPath) {
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
      next = resolveExtends(current, specifier);
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

function resolveExtends(fromConfig, specifier) {
  if (typeof specifier !== "string") return null;
  if (!(specifier.startsWith("./") || specifier.startsWith("../"))) return null;
  const resolved = resolve(dirname(fromConfig), specifier);
  if (existsSync(resolved)) return resolved;
  if (!resolved.endsWith(".json") && existsSync(`${resolved}.json`)) return `${resolved}.json`;
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
