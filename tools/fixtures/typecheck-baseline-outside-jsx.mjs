import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";

import { resolveWithConfigDir } from "./typecheck-baseline-config-dir.mjs";
import { loadTsconfigExtendsChain } from "./typecheck-baseline-extends-chain.mjs";
import {
  isolatedTsconfigOverlayPath,
  resolvePackageExtends,
} from "./typecheck-baseline-outside-paths.mjs";

/**
 * Close the jsx-runtime walk unique isolation cannot retarget (#4461).
 *
 * Unique-link answers `require("vue/jsx-runtime")`. Generated configs also
 * write `jsxImportSource: "../../node_modules/vue"`, which TypeScript resolves
 * from the tsconfig directory and loads Vize's Vue beside the fixture.
 * Overlay `paths` cannot rewrite `compilerOptions.jsxImportSource`. When the
 * fixture already has that package, retarget path specifiers. Package-name
 * sources stay with unique-link. Child replaces parent, matching tsc.
 */

export function rewriteOutsideJsxImportSource(fixtureRoot, sourceConfigPath, configDir) {
  const root = resolve(fixtureRoot);
  const declared = winningJsxImportSource(sourceConfigPath, root);
  if (declared == null) return null;
  if (!isPathSpecifier(declared.specifier)) return null;
  return retargetJsxImportSourcePath(root, declared.dir, configDir, declared.specifier);
}

export function applyIsolatedJsxOverlay(fixtureRoot, sourceConfigPath, overlay) {
  const sourcePath = resolve(sourceConfigPath);
  const rewritten = rewriteOutsideJsxImportSource(fixtureRoot, sourcePath, dirname(sourcePath));
  if (rewritten == null) return overlay ?? null;
  const overlayPath = overlay?.path ?? isolatedTsconfigOverlayPath(sourcePath);
  const document = existsSync(overlayPath)
    ? JSON.parse(readFileSync(overlayPath, "utf8"))
    : { extends: `./${basename(sourcePath)}` };
  document.compilerOptions = { ...(document.compilerOptions ?? {}), jsxImportSource: rewritten };
  writeFileSync(overlayPath, `${JSON.stringify(document, null, 2)}\n`);
  return { ...(overlay ?? {}), path: overlayPath, jsxImportSource: rewritten };
}

export function applyIsolatedJsxBaseline(fixtureRoot, sourceConfigPath, baselinePath) {
  const rewritten = rewriteOutsideJsxImportSource(
    fixtureRoot,
    resolve(sourceConfigPath),
    dirname(resolve(baselinePath)),
  );
  if (rewritten == null) return null;
  const document = JSON.parse(readFileSync(baselinePath, "utf8"));
  document.compilerOptions = { ...(document.compilerOptions ?? {}), jsxImportSource: rewritten };
  writeFileSync(baselinePath, `${JSON.stringify(document, null, 2)}\n`);
  return rewritten;
}

function retargetJsxImportSourcePath(fixtureRoot, sourceDir, configDir, entry) {
  const original = resolveWithConfigDir(sourceDir, sourceDir, entry);
  if (isInside(fixtureRoot, original)) return null;
  const owned = owningNodeModulePackage(original);
  if (owned == null) return null;
  return localPackageTarget(fixtureRoot, configDir, owned.name, relative(owned.root, original));
}

function localPackageTarget(fixtureRoot, configDir, name, subpath) {
  const local = join(fixtureRoot, "node_modules", ...name.split("/"));
  if (!existsSync(join(local, "package.json"))) return null;
  const nested = typeof subpath === "string" ? subpath.replaceAll("\\", "/") : "";
  if (nested.startsWith("..") || nested.startsWith("/")) return null;
  const target = nested === "" ? local : join(local, nested);
  if (nested !== "" && !existsSync(target)) return null;
  return configRelativePath(configDir, target);
}

function isPathSpecifier(entry) {
  return (
    entry.startsWith("./") ||
    entry.startsWith("../") ||
    entry.startsWith("/") ||
    entry.includes("/node_modules/") ||
    entry.includes("${configDir}")
  );
}

function winningJsxImportSource(sourceConfigPath, fixtureRoot) {
  let specifier = null;
  let dir = null;
  for (const { config, dir: configDir } of [
    ...loadTsconfigExtendsChain(
      sourceConfigPath,
      (fromConfig, specifier) =>
        resolveRelativeExtends(fromConfig, specifier, fixtureRoot) ??
        resolvePackageExtends(fromConfig, specifier, fixtureRoot),
    ),
  ].reverse()) {
    const candidate = config?.compilerOptions?.jsxImportSource;
    if (typeof candidate !== "string") continue;
    specifier = candidate;
    dir = configDir;
  }
  return specifier == null ? null : { specifier, dir };
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

function isInside(root, target) {
  const path = relative(root, target);
  return path !== "" && !path.startsWith("..") && !path.startsWith("/");
}

function configRelativePath(from, to) {
  const path = relative(from, to).replaceAll("\\", "/");
  if (path.startsWith("/") || /^[A-Za-z]:\//u.test(path)) return path;
  return path.startsWith(".") ? path : `./${path}`;
}
