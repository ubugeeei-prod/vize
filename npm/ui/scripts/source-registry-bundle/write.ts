import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";

import { buildLibRegistry } from "./bundle.ts";
import {
  LIB_REGISTRY_FILES_DIRECTORY,
  LIB_REGISTRY_OUTPUT_DIRECTORY,
  type LibRegistryBuildInput,
  type LibRegistryBundle,
  type LibRegistryItemKind,
  type LibRegistrySeedItem,
} from "./types.ts";

/** Serialize a manifest exactly as it is published (2-space JSON + newline). */
export function serializeLibRegistryManifest(bundle: LibRegistryBundle): string {
  return `${JSON.stringify(bundle.manifest, null, 2)}\n`;
}

/**
 * Replace `<packageRoot>/registry` with `registry.json` and `files/**`.
 *
 * The directory is removed first so files dropped from the catalog can never
 * linger in a later tarball.
 */
export function writeLibRegistry(packageRoot: string, bundle: LibRegistryBundle): string {
  const outputDirectory = path.join(packageRoot, LIB_REGISTRY_OUTPUT_DIRECTORY);
  rmSync(outputDirectory, { recursive: true, force: true });
  for (const [filePath, bytes] of bundle.files) {
    const target = path.join(outputDirectory, LIB_REGISTRY_FILES_DIRECTORY, ...filePath.split("/"));
    mkdirSync(path.dirname(target), { recursive: true });
    writeFileSync(target, bytes);
  }
  const manifestPath = path.join(outputDirectory, "registry.json");
  writeFileSync(manifestPath, serializeLibRegistryManifest(bundle));
  return manifestPath;
}

interface PackageManifestFields {
  readonly name: string;
  readonly version: string;
  readonly peerDependencies?: Readonly<Record<string, string>>;
  readonly dependencies?: Readonly<Record<string, string>>;
}

function isRecordOfStrings(value: unknown): value is Readonly<Record<string, string>> {
  return (
    typeof value === "object" &&
    value !== null &&
    Object.values(value).every((entry) => typeof entry === "string")
  );
}

function readPackageManifest(packageRoot: string): PackageManifestFields {
  const parsed: unknown = JSON.parse(readFileSync(path.join(packageRoot, "package.json"), "utf8"));
  if (typeof parsed !== "object" || parsed === null)
    throw new Error("package.json is not an object");
  const name: unknown = Reflect.get(parsed, "name");
  const version: unknown = Reflect.get(parsed, "version");
  const peerDependencies: unknown = Reflect.get(parsed, "peerDependencies") ?? {};
  const dependencies: unknown = Reflect.get(parsed, "dependencies") ?? {};
  if (typeof name !== "string" || typeof version !== "string") {
    throw new Error("package.json must declare string name and version");
  }
  if (!isRecordOfStrings(peerDependencies) || !isRecordOfStrings(dependencies)) {
    throw new Error("package.json dependency maps must map names to ranges");
  }
  return { name, version, peerDependencies, dependencies };
}

/** Options for {@link createPackageLibRegistryInput}. */
export interface PackageLibRegistryOptions {
  readonly packageRoot: string;
  readonly kind: LibRegistryItemKind;
  readonly defaultTargetDirectory: string;
  readonly items: readonly LibRegistrySeedItem[];
}

/** Build the registry input from a package directory and its catalog seeds. */
export function createPackageLibRegistryInput(
  options: PackageLibRegistryOptions,
): LibRegistryBuildInput {
  const manifest = readPackageManifest(options.packageRoot);
  return {
    packageName: manifest.name,
    packageVersion: manifest.version,
    kind: options.kind,
    defaultTargetDirectory: options.defaultTargetDirectory,
    sourceRoot: path.join(options.packageRoot, "src"),
    items: options.items,
    peerDependencies: manifest.peerDependencies ?? {},
    dependencies: manifest.dependencies ?? {},
  };
}

/** Build and write a package registry; returns the manifest path. */
export function emitPackageLibRegistry(options: PackageLibRegistryOptions): string {
  return writeLibRegistry(
    options.packageRoot,
    buildLibRegistry(createPackageLibRegistryInput(options)),
  );
}
