import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  assertInstalledPackage,
  assertNoWorkspaceProtocols,
  assertPublishEntrypointsExist,
  isCompatibleWithCurrentRunner,
  isPlatformSpecific,
  readPackageJson,
  type PackageInfo,
} from "./manifest.ts";
import { isFile, runCommand, writeJsonPretty } from "./process.ts";
import { preparePublishManifest } from "./prepare.ts";
import {
  RUNTIME_PEER_DEPENDENCIES,
  runInstalledContentMapperChecks,
  runRuntimeChecks,
} from "./runtime.ts";

interface Options {
  contentMapperChecks: boolean;
  keepTemp: boolean;
  packageDirs: string[];
  prepareManifests: boolean;
  runtimeChecks: boolean;
}
function parseArgs(argv: readonly string[]): Options {
  const options: Options = {
    contentMapperChecks: false,
    keepTemp: false,
    packageDirs: [],
    prepareManifests: false,
    runtimeChecks: false,
  };
  for (const arg of argv) {
    switch (arg) {
      case "--content-mapper-checks":
        options.contentMapperChecks = true;
        break;
      case "--keep-temp":
        options.keepTemp = true;
        break;
      case "--prepare-manifests":
        options.prepareManifests = true;
        break;
      case "--runtime-checks":
        options.runtimeChecks = true;
        break;
      default:
        if (arg.startsWith("--")) throw new Error(`Unknown argument: ${arg}`);
        try {
          options.packageDirs.push(fs.realpathSync.native(arg));
        } catch {
          options.packageDirs.push(arg);
        }
    }
  }
  if (!options.packageDirs.length)
    throw new Error(
      "Usage: rust-script tools/commands/release/npm/smoke-release-install.rs [--prepare-manifests] [--runtime-checks] [--content-mapper-checks] [--keep-temp] <package-dir>...",
    );
  return options;
}
function packPackage(directory: string, packs: string): string {
  const before = new Set(fs.readdirSync(packs));
  runCommand(
    process.env.NPM_BIN ?? "npm",
    ["pack", "--ignore-scripts", "--pack-destination", packs],
    directory,
  );
  const created = fs
    .readdirSync(packs)
    .sort()
    .filter((entry) => entry.endsWith(".tgz") && !before.has(entry));
  if (created.length !== 1) throw new Error(`expected exactly one tarball from ${directory}`);
  return fs.realpathSync.native(path.join(packs, created[0]));
}
function installPackedPackages(
  temporary: string,
  packages: readonly PackageInfo[],
  runtimePeers: boolean,
): string {
  const install = path.join(temporary, "install"),
    dependencies: Record<string, string> = {};
  fs.mkdirSync(install, { recursive: true });
  for (const pkg of packages) dependencies[pkg.name] = `file:${pkg.tarball}`;
  if (runtimePeers) Object.assign(dependencies, RUNTIME_PEER_DEPENDENCIES);
  writeJsonPretty(path.join(install, "package.json"), {
    name: "vize-release-install-smoke",
    private: true,
    dependencies,
  });
  runCommand(
    process.env.NPM_BIN ?? "npm",
    [
      "install",
      "--ignore-scripts",
      "--package-lock=false",
      "--no-audit",
      "--fund=false",
      "--legacy-peer-deps",
      "--include=optional",
    ],
    install,
  );
  for (const pkg of packages) assertInstalledPackage(path.join(install, "node_modules"), pkg);
  return install;
}
export function smokeReleaseInstall(argv: readonly string[]): number {
  const options = parseArgs(argv),
    temporary = fs.realpathSync.native(
      fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-")),
    );
  const packs = path.join(temporary, "packs");
  fs.mkdirSync(packs, { recursive: true });
  try {
    const packages: PackageInfo[] = [];
    for (const directory of options.packageDirs) {
      if (!isFile(path.join(directory, "package.json")))
        throw new Error(`${directory} does not contain package.json`);
      if (options.prepareManifests) preparePublishManifest(directory);
      const manifest = readPackageJson(directory);
      assertNoWorkspaceProtocols(directory, manifest);
      const compatible = isCompatibleWithCurrentRunner(manifest);
      if (!isPlatformSpecific(manifest)) assertPublishEntrypointsExist(directory, manifest);
      const tarball = packPackage(directory, packs);
      packages.push({ compatible, name: manifest.name, version: manifest.version, tarball });
      console.log(`${compatible ? "install" : "pack-only"}: ${manifest.name}@${manifest.version}`);
    }
    const installable = packages.filter((pkg) => pkg.compatible);
    if (!installable.length) throw new Error("no package tarballs are compatible with this runner");
    const install = installPackedPackages(
      temporary,
      installable,
      options.runtimeChecks || options.contentMapperChecks,
    );
    if (options.runtimeChecks) runRuntimeChecks(install, installable, packages, temporary);
    if (options.contentMapperChecks) runInstalledContentMapperChecks(install);
    console.log(`smoked ${installable.length}/${packages.length} package tarballs`);
    return 0;
  } finally {
    if (options.keepTemp) console.log(`kept ${temporary}`);
    else
      try {
        fs.rmSync(temporary, { recursive: true, force: true });
      } catch {
        /* Rust's cleanup is best effort. */
      }
  }
}
