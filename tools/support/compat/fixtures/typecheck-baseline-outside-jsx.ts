import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";

import { resolveWithConfigDir } from "./typecheck-baseline-config-dir.ts";
import { loadTsconfigExtendsChain } from "./typecheck-baseline-extends-chain.ts";
import {
  isolatedTsconfigOverlayPath,
  resolvePackageExtends,
  type IsolatedOverlay,
} from "./typecheck-baseline-outside-paths.ts";

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

export function rewriteOutsideJsxImportSource(
  fixtureRoot: string,
  sourceConfigPath: string,
  configDir: string,
): string | null {
  const root = resolve(fixtureRoot);
  const declared = winningJsxImportSource(sourceConfigPath, root);
  if (declared == null) return null;
  if (!isPathSpecifier(declared.specifier)) return null;
  return retargetJsxImportSourcePath(root, declared.dir, configDir, declared.specifier);
}

export function applyIsolatedJsxOverlay(
  fixtureRoot: string,
  sourceConfigPath: string,
  overlay: IsolatedOverlay | null | undefined,
): IsolatedOverlay | null {
  const sourcePath = resolve(sourceConfigPath);
  const rewritten = rewriteOutsideJsxImportSource(fixtureRoot, sourcePath, dirname(sourcePath));
  if (rewritten == null) return overlay ?? null;
  const overlayPath = overlay?.path ?? isolatedTsconfigOverlayPath(sourcePath);
  const document: { extends?: unknown; compilerOptions?: object } = existsSync(overlayPath)
    ? (JSON.parse(readFileSync(overlayPath, "utf8")) as { compilerOptions?: object })
    : { extends: `./${basename(sourcePath)}` };
  document.compilerOptions = { ...document.compilerOptions, jsxImportSource: rewritten };
  writeFileSync(overlayPath, `${JSON.stringify(document, null, 2)}\n`);
  return { ...overlay, path: overlayPath, jsxImportSource: rewritten };
}

export function applyIsolatedJsxBaseline(
  fixtureRoot: string,
  sourceConfigPath: string,
  baselinePath: string,
): string | null {
  const rewritten = rewriteOutsideJsxImportSource(
    fixtureRoot,
    resolve(sourceConfigPath),
    dirname(resolve(baselinePath)),
  );
  if (rewritten == null) return null;
  const document = JSON.parse(readFileSync(baselinePath, "utf8")) as { compilerOptions?: object };
  document.compilerOptions = { ...document.compilerOptions, jsxImportSource: rewritten };
  writeFileSync(baselinePath, `${JSON.stringify(document, null, 2)}\n`);
  return rewritten;
}

function retargetJsxImportSourcePath(
  fixtureRoot: string,
  sourceDir: string,
  configDir: string,
  entry: string,
): string | null {
  const original = resolveWithConfigDir(sourceDir, sourceDir, entry);
  if (isInside(fixtureRoot, original)) return null;
  const owned = owningNodeModulePackage(original);
  if (owned == null) return null;
  return localPackageTarget(fixtureRoot, configDir, owned.name, relative(owned.root, original));
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

function winningJsxImportSource(
  sourceConfigPath: string,
  fixtureRoot: string,
): { specifier: string; dir: string } | null {
  let specifier: string | null = null;
  let dir: string | null = null;
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
  return specifier == null ? null : { specifier, dir: dir! };
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
