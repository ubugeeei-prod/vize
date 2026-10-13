// Rewrites workspace-local dependency protocols into publishable npm versions.
// Release jobs publish prebuilt package directories without a full workspace
// install. That means protocol-based specs such as `workspace:*` and
// `catalog:*` must be resolved inside `package.json` before `vp pm publish`
// runs, otherwise pnpm aborts before it ever talks to the registry.
// This helper makes a manifest standalone by:
// 1. Finding the repository root that owns the target package.
// 2. Collecting workspace package versions from `npm/**/package.json`.
// 3. Reading catalog pins from `pnpm-workspace.yaml`.
// 4. Rewriting dependency-like sections in place.
import { println, runMain, writeStderr, writeStdout } from "../command_io.ts";
import {
  cliArgs,
  dirnamePath,
  isDir,
  joinPath,
  moonTrim,
  pathExists,
  readDir,
  readFileToString,
  replaceEvery,
  replaceFirst,
  resolvePath,
  writeStringToFile,
} from "../command_runtime.ts";
import {
  isJsonObject,
  parseJson,
  stringMember,
  stringifyJson,
  type JsonValue,
} from "../ordered_json.ts";

const DEPENDENCY_SECTIONS = [
  "dependencies",
  "peerDependencies",
  "optionalDependencies",
  "devDependencies",
] as const;

function readPackageJson(target: string): JsonValue | undefined {
  const content = readFileToString(target);
  return content === null ? undefined : parseJson(content);
}

function nonEmptyMember(value: JsonValue, key: string): string | null {
  const member = stringMember(value, key);
  return member === undefined || member === "" ? null : member;
}

function yamlScalar(value: string): string {
  return replaceEvery(moonTrim(value), '"', "");
}

function findRepoRoot(start: string): string | null {
  const currentPath = resolvePath(start);
  if (
    pathExists(joinPath(currentPath, "pnpm-workspace.yaml")) &&
    isDir(joinPath(currentPath, "npm"))
  ) {
    return currentPath;
  }
  const parent = dirnamePath(start);
  return parent === currentPath ? null : findRepoRoot(parent);
}

function collectWorkspacePackages(directory: string, versions: Map<string, string>): void {
  for (const entryName of readDir(directory)) {
    if (entryName === "node_modules" || entryName === ".git" || entryName === "_build") {
      continue;
    }
    const entryPath = joinPath(directory, entryName);
    if (isDir(entryPath)) {
      collectWorkspacePackages(entryPath, versions);
      continue;
    }
    if (entryName !== "package.json") continue;
    const json = readPackageJson(entryPath);
    if (json === undefined) continue;
    const name = nonEmptyMember(json, "name");
    const version = nonEmptyMember(json, "version");
    if (name !== null && version !== null) {
      versions.set(name, version);
    }
  }
}

/** Flattens catalog pins into a lookup keyed by `<catalog>\0<package>`. */
export function parseCatalogVersions(content: string): Map<string, string> {
  const versions = new Map<string, string>();
  let inCatalogs = false;
  let currentCatalog = "";
  for (const line of content.split("\n")) {
    const trimmed = moonTrim(line);
    if (trimmed === "" || trimmed.startsWith("#")) continue;
    if (!inCatalogs) {
      if (trimmed === "catalogs:") inCatalogs = true;
      continue;
    }
    if (!line.startsWith("  ")) break;
    if (line.startsWith("  ") && !line.startsWith("    ") && trimmed.endsWith(":")) {
      currentCatalog = replaceFirst(trimmed, ":", "");
      continue;
    }
    if (currentCatalog === "" || !line.startsWith("    ")) continue;
    const parts = trimmed.split(": ");
    if (parts.length < 2) continue;
    versions.set(`${currentCatalog}\u0000${yamlScalar(parts[0])}`, yamlScalar(parts[1]));
  }
  return versions;
}

function normalizeWorkspaceSpec(spec: string, version: string): string {
  const suffix = replaceFirst(spec, "workspace:", "");
  if (suffix === "^") return `^${version}`;
  if (suffix === "~") return `~${version}`;
  return version;
}

export function normalizeDependencySections(
  packageJson: JsonValue,
  workspaceVersions: ReadonlyMap<string, string>,
  catalogVersions: ReadonlyMap<string, string>,
  nativeBinaryVersion: string | null,
): string[] {
  const unresolved: string[] = [];
  if (!isJsonObject(packageJson)) {
    unresolved.push("package.json root is not an object");
    return unresolved;
  }
  for (const section of DEPENDENCY_SECTIONS) {
    const dependencies = packageJson.get(section);
    if (!isJsonObject(dependencies)) continue;
    for (const [dependencyName, dependencyValue] of dependencies.members) {
      if (typeof dependencyValue !== "string") continue;
      const versionSpec = dependencyValue;
      if (versionSpec.startsWith("workspace:")) {
        const version = workspaceVersions.get(dependencyName);
        if (version === undefined) {
          unresolved.push(
            `Missing workspace version for ${dependencyName} referenced from ${section}`,
          );
        } else {
          dependencies.set(dependencyName, normalizeWorkspaceSpec(versionSpec, version));
        }
        continue;
      }
      if (!versionSpec.startsWith("catalog:")) continue;
      const catalogName = replaceFirst(versionSpec, "catalog:", "");
      if (catalogName === "native-binaries" && dependencyName.startsWith("@vizejs/native-")) {
        if (nativeBinaryVersion === null) {
          unresolved.push(`Missing native package version for ${dependencyName} from ${section}`);
        } else {
          dependencies.set(dependencyName, nativeBinaryVersion);
        }
        continue;
      }
      const version = catalogVersions.get(`${catalogName}\u0000${dependencyName}`);
      if (version === undefined) {
        unresolved.push(
          `Missing catalog version for ${dependencyName} from ${catalogName} in ${section}`,
        );
      } else {
        dependencies.set(dependencyName, version);
      }
    }
  }
  return unresolved;
}

const USAGE =
  "Usage: node tools/support/release/prepare_npm_publish_manifest/main.ts <dir-or-package-json> [--print]\n";

/**
 * Main entry point for standalone publish manifest preparation.
 * The script accepts either a package directory or a direct `package.json`
 * path. `--print` is useful in CI when we need to inspect the rewritten
 * manifest that will actually be published.
 */
export async function prepareNpmPublishManifest(args: readonly string[]): Promise<number> {
  if (args.length === 0) {
    writeStderr(USAGE);
    return 1;
  }
  let shouldPrint = false;
  const positional: string[] = [];
  for (const arg of args) {
    if (arg === "--print") {
      shouldPrint = true;
    } else {
      positional.push(arg);
    }
  }
  if (positional.length === 0) {
    writeStderr("Missing package directory or package.json path\n");
    return 1;
  }

  const rawTarget = resolvePath(positional[0]);
  const packageJsonPath = isDir(rawTarget) ? joinPath(rawTarget, "package.json") : rawTarget;
  const packageJson = readPackageJson(packageJsonPath);
  if (packageJson === undefined) {
    writeStderr(`Failed to parse ${packageJsonPath}\n`);
    return 1;
  }
  const repoRoot = findRepoRoot(dirnamePath(packageJsonPath));
  if (repoRoot === null) {
    if (!isJsonObject(packageJson)) {
      writeStderr(`Failed to read object from ${packageJsonPath}\n`);
      return 1;
    }
    const unresolved: string[] = [];
    for (const section of DEPENDENCY_SECTIONS) {
      const dependencies = packageJson.get(section);
      if (!isJsonObject(dependencies)) continue;
      for (const [dependencyName, dependencyValue] of dependencies.members) {
        if (typeof dependencyValue !== "string") continue;
        if (dependencyValue.startsWith("workspace:") || dependencyValue.startsWith("catalog:")) {
          unresolved.push(
            `Cannot normalize ${dependencyName} from ${section} because the repository root could not be located`,
          );
        }
      }
    }
    if (unresolved.length === 0) {
      println(`No workspace or catalog dependencies needed rewriting for ${packageJsonPath}`);
      return 0;
    }
    for (const item of unresolved) writeStderr(`${item}\n`);
    return 1;
  }

  const workspaceVersions = new Map<string, string>();
  collectWorkspacePackages(joinPath(repoRoot, "npm"), workspaceVersions);
  const workspaceYamlPath = joinPath(repoRoot, "pnpm-workspace.yaml");
  const workspaceYaml = readFileToString(workspaceYamlPath);
  if (workspaceYaml === null) {
    writeStderr(`Failed to read ${workspaceYamlPath}\n`);
    return 1;
  }
  const catalogVersions = parseCatalogVersions(workspaceYaml);
  const nativeBinaryVersion =
    workspaceVersions.get("@vizejs/native") ?? nonEmptyMember(packageJson, "version");
  const unresolved = normalizeDependencySections(
    packageJson,
    workspaceVersions,
    catalogVersions,
    nativeBinaryVersion,
  );
  if (unresolved.length > 0) {
    for (const item of unresolved) writeStderr(`${item}\n`);
    return 1;
  }
  const updated = `${stringifyJson(packageJson, 2)}\n`;
  writeStringToFile(packageJsonPath, updated);
  println(`Prepared npm publish manifest at ${packageJsonPath}`);
  if (shouldPrint) {
    writeStdout(updated);
  }
  return 0;
}

if (import.meta.main) {
  runMain(() => prepareNpmPublishManifest(cliArgs()));
}
