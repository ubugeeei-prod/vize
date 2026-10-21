/** Deterministic identity and installed-size receipt for a packed fresh project (#3956). */

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

export interface PackedInstallReceipt {
  schema: "vize.packedInstallReceipt";
  version: 1;
  projectShape: string;
  packageManager: string;
  packages: Array<{ name: string; version: string; installedBytes: number }>;
  nativeBinary: {
    package: string;
    path: string;
    bytes: number;
    sha256: string;
  };
}

interface NativeBinarySelection {
  packageName: string;
  packageRoot: string;
  binaryPath: string;
}

const byCodeUnit = (left: string, right: string) => (left < right ? -1 : left > right ? 1 : 0);

function packagePath(nodeModules: string, name: string): string {
  return path.join(nodeModules, ...name.split("/"));
}

function packageManifest(packageRoot: string): { name: string; version: string } {
  const filename = path.join(packageRoot, "package.json");
  const manifest = JSON.parse(fs.readFileSync(filename, "utf8")) as {
    name?: unknown;
    version?: unknown;
  };
  assert.equal(typeof manifest.name, "string", `${filename} is missing name`);
  assert.equal(typeof manifest.version, "string", `${filename} is missing version`);
  return manifest as { name: string; version: string };
}

function directPackageRoot(projectRoot: string, name: string): string {
  const installed = packagePath(path.join(projectRoot, "node_modules"), name);
  assert.ok(fs.existsSync(installed), `${name} is missing from the fresh project`);
  const packageRoot = fs.realpathSync(installed);
  assert.equal(packageManifest(packageRoot).name, name);
  return packageRoot;
}

function containingPackageRoot(resolved: string, name: string): string {
  let directory = fs.statSync(resolved).isDirectory() ? resolved : path.dirname(resolved);
  for (;;) {
    const manifest = path.join(directory, "package.json");
    if (fs.existsSync(manifest) && packageManifest(directory).name === name) return directory;
    const parent = path.dirname(directory);
    assert.notEqual(parent, directory, `could not find installed ${name} above ${resolved}`);
    directory = parent;
  }
}

function resolvedPackageRoot(requireFrom: NodeJS.Require, name: string): string {
  try {
    return containingPackageRoot(requireFrom.resolve(`${name}/package.json`), name);
  } catch {
    return containingPackageRoot(requireFrom.resolve(name), name);
  }
}

function installedBytes(packageRoot: string): number {
  let bytes = 0;
  const visit = (directory: string) => {
    const entries = fs
      .readdirSync(directory, { withFileTypes: true })
      .sort((left, right) => byCodeUnit(left.name, right.name));
    for (const entry of entries) {
      const filename = path.join(directory, entry.name);
      if (entry.isDirectory()) visit(filename);
      else if (entry.isFile()) bytes += fs.statSync(filename).size;
      // Package-manager links are installation topology, not package payload.
    }
  };
  visit(packageRoot);
  return bytes;
}

function selectedNativeBinary(nativeRoot: string): NativeBinarySelection {
  const nativeRequire = createRequire(path.join(nativeRoot, "package.json"));
  const targetsModule = nativeRequire("./native-targets.js") as {
    nativeTargets(errors: unknown[]): string[];
  };
  const targets = targetsModule.nativeTargets([]);
  assert.ok(targets.length > 0, "installed @vizejs/native selected no host target");

  for (const target of targets) {
    const localBinary = path.join(nativeRoot, `vize-vitrine.${target}.node`);
    if (fs.existsSync(localBinary)) {
      return { packageName: "@vizejs/native", packageRoot: nativeRoot, binaryPath: localBinary };
    }

    const packageName = `@vizejs/native-${target}`;
    try {
      const binaryPath = fs.realpathSync(nativeRequire.resolve(packageName));
      assert.equal(path.extname(binaryPath), ".node", `${packageName} did not resolve to a binary`);
      return {
        packageName,
        packageRoot: containingPackageRoot(binaryPath, packageName),
        binaryPath,
      };
    } catch (error) {
      if (error instanceof assert.AssertionError) throw error;
    }
  }
  throw new Error(`installed @vizejs/native has no binary for ${targets.join(", ")}`);
}

export function createPackedInstallReceipt(
  projectRoot: string,
  packageNames: readonly string[],
  cell: { projectShape: string; packageManager: string },
): PackedInstallReceipt {
  const packageRoots = new Map<string, string>();
  for (const name of packageNames) packageRoots.set(name, directPackageRoot(projectRoot, name));

  const vizeRoot = packageRoots.get("vize");
  assert.equal(typeof vizeRoot, "string", "packed install receipt requires vize");
  const vizeRequire = createRequire(path.join(vizeRoot, "package.json"));
  const nativeRoot = resolvedPackageRoot(vizeRequire, "@vizejs/native");
  packageRoots.set("@vizejs/native", nativeRoot);
  const nativeBinary = selectedNativeBinary(nativeRoot);
  packageRoots.set(nativeBinary.packageName, nativeBinary.packageRoot);

  const packages = [...packageRoots]
    .sort(([left], [right]) => byCodeUnit(left, right))
    .map(([name, packageRoot]) => {
      const manifest = packageManifest(packageRoot);
      assert.equal(manifest.name, name);
      return { name, version: manifest.version, installedBytes: installedBytes(packageRoot) };
    });
  const relativeNativePath = path.relative(nativeBinary.packageRoot, nativeBinary.binaryPath);
  assert.ok(
    relativeNativePath !== "" &&
      !path.isAbsolute(relativeNativePath) &&
      !relativeNativePath.startsWith(".."),
    `selected native binary is outside ${nativeBinary.packageRoot}`,
  );
  const binary = fs.readFileSync(nativeBinary.binaryPath);

  return {
    schema: "vize.packedInstallReceipt",
    version: 1,
    projectShape: cell.projectShape,
    packageManager: cell.packageManager,
    packages,
    nativeBinary: {
      package: nativeBinary.packageName,
      path: relativeNativePath.split(path.sep).join("/"),
      bytes: binary.length,
      sha256: createHash("sha256").update(binary).digest("hex"),
    },
  };
}

export function recordPackedInstallReceipt(
  projectRoot: string,
  packageNames: readonly string[],
  cell: { projectShape: string; packageManager: string },
): PackedInstallReceipt {
  const receipt = createPackedInstallReceipt(projectRoot, packageNames, cell);
  console.log(`runtime: packed install receipt ${JSON.stringify(receipt)}`);
  return receipt;
}
