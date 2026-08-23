import { existsSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";

import { expandConfigDir, resolveWithConfigDir } from "./typecheck-baseline-config-dir.mjs";
import { loadTsconfigExtendsChain } from "./typecheck-baseline-extends-chain.mjs";

/**
 * Close the vue-tsc path escape unique isolation cannot see (#4461).
 *
 * Unique isolation links a fixture-local copy so `/// <reference types />`
 * stops walking into Vize's `node_modules`. `compilerOptions.paths` is a
 * different resolver: TypeScript uses the declared mapping and never consults
 * that link. Nuxt-generated configs bake an absolute-looking relative path to
 * a package *above* the fixture, so the baseline still loads Vize's Vue beside
 * the fixture's.
 * Setting `paths` on the generated baseline replaces the inherited object, so
 * this copies every mapping, re-homes it to the generated config directory, and
 * only then retargets package names whose original target is outside and whose
 * fixture-local `node_modules/<name>` is already a real package. No rewrite
 * means the generated config leaves `paths` unset and inherits.
 * Vize's `check --tsconfig` still reads the fixture config, so a matching
 * untracked overlay is written next to the source when a rewrite exists. Git
 * porcelain uses `--untracked-files=no`, so the overlay does not dirty the
 * fixture. The matrix typechecker prefers that overlay when it is present.
 * Package-name `extends` is followed only inside the fixture. Climbing into
 * Vize's `node_modules` would load Vize's `@vue/tsconfig` and bake its
 * outside `paths` into the overlay.
 * `compilerOptions.typeRoots` and `rootDirs` are the same walk: an outside
 * `node_modules/@types` or `node_modules/<name>` is retargeted to the fixture
 * copy when that directory already exists locally. A trailing `/*` on a package
 * mapping, or a `*` mapping onto an outside `node_modules` directory, is
 * retargeted the same way. Interior `*` patterns and `#alias/*` keys are not
 * guessed. Package mappings keep `node_modules/<name>/...` subpaths (Nuxt's
 * `vue/dist/vue` JSX entry). `baseUrl` is last-wins; a rewrite pins it to `"."`.
 * `${configDir}` in `baseUrl`, `paths`, `typeRoots`, and `rootDirs` expands to
 * the declaring tsconfig directory, matching tsc, before those paths resolve.
 */

const packageNamePattern = /^(?:@[a-z0-9][a-z0-9-._]*\/)?[a-z0-9][a-z0-9-._]*$/u;

export function rewriteOutsidePackagePaths(fixtureRoot, sourceConfigPath, configDir) {
  const root = resolve(fixtureRoot);
  const declared = winningPaths(sourceConfigPath, root);
  if (declared == null) return null;
  const mapping = pathMappingRoot(sourceConfigPath, root, declared.dir);
  const rewritten = {};
  let changed = false;
  for (const [name, targets] of Object.entries(declared.paths)) {
    if (!Array.isArray(targets)) continue;
    rewritten[name] = retargetPathMapping(
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

export function pathMappingRoot(sourceConfigPath, fixtureRoot, pathsDir) {
  const base = winningBaseUrl(sourceConfigPath, fixtureRoot);
  return base == null ? pathsDir : resolve(base.dir, expandConfigDir(base.value, base.dir));
}

export function isolatedOverlayBaseUrl(sourceConfigPath, fixtureRoot) {
  return winningBaseUrl(sourceConfigPath, fixtureRoot) == null ? null : ".";
}

export function isolatedTsconfigOverlayPath(sourceConfigPath) {
  const dir = dirname(sourceConfigPath).replaceAll("\\", "/");
  const name = basename(sourceConfigPath);
  return dir === "." ? `.vize-isolated-${name}` : `${dir}/.vize-isolated-${name}`;
}

export function rewriteOutsideTypeRoots(fixtureRoot, sourceConfigPath, configDir) {
  return rewriteOutsideDirList(
    fixtureRoot,
    sourceConfigPath,
    configDir,
    "typeRoots",
    retargetTypeRoot,
  );
}

export function rewriteOutsideRootDirs(fixtureRoot, sourceConfigPath, configDir) {
  return rewriteOutsideDirList(
    fixtureRoot,
    sourceConfigPath,
    configDir,
    "rootDirs",
    retargetRootDir,
  );
}

export function writeIsolatedTsconfigOverlay(fixtureRoot, sourceConfigPath) {
  const sourcePath = resolve(sourceConfigPath);
  const configDir = dirname(sourcePath);
  const paths = rewriteOutsidePackagePaths(fixtureRoot, sourcePath, configDir);
  const typeRoots = rewriteOutsideTypeRoots(fixtureRoot, sourcePath, configDir);
  const rootDirs = rewriteOutsideRootDirs(fixtureRoot, sourcePath, configDir);
  if (paths == null && typeRoots == null && rootDirs == null) return null;
  const compilerOptions = {};
  if (paths != null) compilerOptions.paths = paths;
  if (typeRoots != null) compilerOptions.typeRoots = typeRoots;
  if (rootDirs != null) compilerOptions.rootDirs = rootDirs;
  if (paths != null && isolatedOverlayBaseUrl(sourcePath, fixtureRoot) != null) {
    compilerOptions.baseUrl = ".";
  }
  const overlayPath = isolatedTsconfigOverlayPath(sourcePath);
  writeFileSync(
    overlayPath,
    `${JSON.stringify({ extends: `./${basename(sourcePath)}`, compilerOptions }, null, 2)}\n`,
  );
  return { path: overlayPath, paths, typeRoots, rootDirs };
}

function retargetPathMapping(
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
  const packageName = packageNameFromPathMapping(name);
  const first = targets.find(
    (entry) => typeof entry === "string" && isExactOrTrailingStarPath(entry),
  );
  if (first == null) return relocated;
  const original = resolveWithConfigDir(
    sourceDir,
    tsconfigDir,
    first.endsWith("/*") ? first.slice(0, -2) : first,
  );
  if (isInside(fixtureRoot, original)) return relocated;
  let local;
  if (packageName != null) {
    const localPackage = join(fixtureRoot, "node_modules", ...packageName.split("/"));
    if (!existsSync(join(localPackage, "package.json"))) return relocated;
    const subpath = packageSubpathAfterNodeModules(original, packageName);
    local = subpath ? join(localPackage, subpath) : localPackage;
    if (subpath && !existsSync(local) && !existsSync(`${local}.d.ts`)) return relocated;
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

function packageSubpathAfterNodeModules(original, packageName) {
  const normalized = original.replaceAll("\\", "/");
  const folder = `/node_modules/${packageName}`;
  const nested = `${folder}/`;
  const index = normalized.lastIndexOf(nested);
  if (index !== -1) return normalized.slice(index + nested.length);
  return normalized.endsWith(folder) ? "" : null;
}

function packageNameFromPathMapping(name) {
  if (typeof name !== "string") return null;
  const base = name.endsWith("/*") ? name.slice(0, -2) : name;
  return packageNamePattern.test(base) ? base : null;
}

function isExactOrTrailingStarPath(entry) {
  return !entry.includes("*") || (entry.endsWith("/*") && !entry.slice(0, -2).includes("*"));
}

function relocatePathEntry(sourceDir, tsconfigDir, configDir, entry) {
  if (typeof entry !== "string" || entry.includes("*")) return entry;
  return configRelativePath(configDir, resolveWithConfigDir(sourceDir, tsconfigDir, entry));
}

function rewriteOutsideDirList(fixtureRoot, sourceConfigPath, configDir, option, retarget) {
  const root = resolve(fixtureRoot);
  const won = winningCompilerOption(sourceConfigPath, root, (config) =>
    Array.isArray(config?.compilerOptions?.[option]) ? config.compilerOptions[option] : null,
  );
  if (won == null) return null;
  let changed = false;
  const rewritten = won.value.map((entry) =>
    retarget(root, won.dir, configDir, entry, () => {
      changed = true;
    }),
  );
  return changed ? rewritten : null;
}

function retargetRootDir(fixtureRoot, sourceDir, configDir, entry, markChanged) {
  if (typeof entry !== "string") return entry;
  const relocated = relocatePathEntry(sourceDir, sourceDir, configDir, entry);
  const original = resolveWithConfigDir(sourceDir, sourceDir, entry);
  if (isInside(fixtureRoot, original)) return relocated;
  const owned = ownedNodeModulePackage(original);
  if (owned == null) return relocated;
  const local = join(fixtureRoot, "node_modules", ...owned.name.split("/"));
  if (!existsSync(join(local, "package.json"))) return relocated;
  const dir = owned.subpath === "" ? local : join(local, owned.subpath);
  if (owned.subpath !== "" && !existsSync(dir)) return relocated;
  markChanged();
  return configRelativePath(configDir, dir);
}

function ownedNodeModulePackage(target) {
  const normalized = target.replaceAll("\\", "/");
  const index = normalized.lastIndexOf("/node_modules/");
  if (index < 0) return null;
  const rest = normalized.slice(index + "/node_modules/".length);
  const name = rest.startsWith("@") ? rest.split("/").slice(0, 2).join("/") : rest.split("/")[0];
  if (!packageNamePattern.test(name)) return null;
  return { name, subpath: rest.slice(name.length).replace(/^\//, "") };
}

function retargetTypeRoot(fixtureRoot, sourceDir, configDir, entry, markChanged) {
  if (typeof entry !== "string") return entry;
  const relocated = relocatePathEntry(sourceDir, sourceDir, configDir, entry);
  const original = resolveWithConfigDir(sourceDir, sourceDir, entry);
  if (isInside(fixtureRoot, original)) return relocated;
  if (!isTypesPackageRoot(original)) return relocated;
  const local = join(fixtureRoot, "node_modules", "@types");
  if (!existsSync(local)) return relocated;
  markChanged();
  return configRelativePath(configDir, local);
}

function winningCompilerOption(sourceConfigPath, fixtureRoot, pick) {
  let value;
  let dir;
  for (const entry of [
    ...loadTsconfigExtendsChain(sourceConfigPath, (fromConfig, specifier) =>
      resolveExtends(fromConfig, specifier, fixtureRoot),
    ),
  ].reverse()) {
    const candidate = pick(entry.config);
    if (candidate != null) {
      value = candidate;
      dir = entry.dir;
    }
  }
  return value == null ? null : { value, dir };
}

function winningPaths(sourceConfigPath, fixtureRoot) {
  const won = winningCompilerOption(sourceConfigPath, fixtureRoot, (config) => {
    const value = config?.compilerOptions?.paths;
    return value != null && typeof value === "object" ? value : null;
  });
  return won == null ? null : { paths: won.value, dir: won.dir };
}

function winningBaseUrl(sourceConfigPath, fixtureRoot) {
  return winningCompilerOption(sourceConfigPath, fixtureRoot, (config) =>
    typeof config?.compilerOptions?.baseUrl === "string" ? config.compilerOptions.baseUrl : null,
  );
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

export function resolvePackageExtends(fromConfig, specifier, fixtureRoot) {
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
  return {
    name: specifier.slice(0, slash),
    subpath: specifier.slice(slash + 1),
  };
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
