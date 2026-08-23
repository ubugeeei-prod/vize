import { existsSync, lstatSync, mkdirSync, readdirSync, readFileSync, symlinkSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";

/**
 * Keep the fixture's type environment inside the fixture (run 31979524200).
 *
 * A fixture lives at `tests/_fixtures/_git/<id>`, so every `node_modules`
 * directory Vize installs for itself — the repository root's and `tests/`' — is
 * on the fixture's own module resolution path. That is harmless for `import`,
 * because the fixture's config maps what it needs through `compilerOptions.paths`.
 * It is not harmless for `/// <reference types="..." />`: TypeScript resolves a
 * type reference directive by walking `node_modules` upward from the containing
 * file and never consults `paths`, so a package the fixture depends on but its
 * package manager does not link at `<fixture>/node_modules/<name>` escapes the
 * fixture and is satisfied by Vize's copy instead.
 *
 * On run 31979524200 that was elk's `.nuxt/nuxt.d.ts` asking for `vue-router`.
 * pnpm keeps transitive dependencies out of the top level, so the walk reached
 * Vize's `vue-router@4.5.1`, which pulled in Vize's `vue@3.6.0-beta.10` beside
 * elk's own `vue@3.5.30`. With two identities in one program elk's own
 * augmentations stopped reaching its components, and its instance type lost
 * every member Nuxt and vue-i18n contribute. The baseline reported 902
 * diagnostics; with the escape closed it reports 9.
 *
 * The repair is to give the resolver a fixture-local answer before it can leave,
 * and the fixture's own config already says which answer: Nuxt writes
 * `"vue-router": ["../node_modules/.pnpm/vue-router@5.1.0.../node_modules/vue-router"]`
 * into `paths` for exactly this package. So this materializes those declarations
 * as links, rather than choosing a version itself.
 *
 * It is deliberately conservative. A name is linked only when the fixture's
 * config declares it, an ancestor `node_modules` could otherwise answer it, the
 * declared target is a real package directory, and that directory is inside the
 * fixture. Everything else is left alone, and the ambient-environment gate in
 * `typecheck-baseline-ambient.mjs` is what proves the outcome — this is the
 * repair, not the check.
 */

/** `paths` also carries `#imports`-style aliases and `foo/*` patterns, which are not packages. */
const packageNamePattern = /^(?:@[a-z0-9][a-z0-9-._]*\/)?[a-z0-9][a-z0-9-._]*$/u;

export function isolateFixtureTypePackages(fixtureRoot, sourceConfigPath) {
  const root = resolve(fixtureRoot);
  const declared = readDeclaredPackagePaths(sourceConfigPath);
  if (declared.size === 0) return [];
  const reachable = collectAncestorPackageNames(root);
  const shadowed = [];
  for (const [name, target] of [...declared].sort(([left], [right]) => compare(left, right))) {
    if (!reachable.has(name)) continue;
    const link = join(root, "node_modules", name);
    if (existsSync(link) || isDanglingLink(link)) continue;
    if (!isPackageDirectory(target) || !isInside(root, target)) continue;
    mkdirSync(dirname(link), { recursive: true });
    symlinkSync(relative(dirname(link), target), link);
    shadowed.push({ name, target: relative(root, target).replaceAll("\\", "/") });
  }
  return shadowed;
}

function readDeclaredPackagePaths(sourceConfigPath) {
  const declared = new Map();
  let config;
  try {
    config = JSON.parse(stripJsonc(readFileSync(sourceConfigPath, "utf8")));
  } catch {
    // Missing or invalid JSON still declares nothing. JSONC comments and
    // trailing commas are stripped first so configs like reka-ui's
    // `tsconfig.check.json` keep their `paths` (#4461).
    return declared;
  }
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

/**
 * Only names an ancestor could actually answer are linked, so a fixture that is
 * already self-contained is left byte-identical and the repair cannot invent a
 * resolution the real project never had.
 */
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

/** `existsSync` follows symlinks, so a broken link would otherwise be overwritten. */
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

/**
 * Strip JSONC comments and trailing commas, string-aware. A regex pass would
 * treat `"src/**\/*"` as a block comment and rewrite the config.
 */
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
