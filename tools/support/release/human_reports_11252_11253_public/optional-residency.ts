import assert from "node:assert/strict";
import { lstatSync, readFileSync, readlinkSync, realpathSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { lockPackageIntegrity } from "./lock-integrity.ts";
import { array, bytes, object, sha256, text, wholeJson } from "./value.ts";
import type { PublicMetadata } from "./network.ts";
import { sriSha512 } from "../public_install/registry.ts";

export interface HostFacts {
  os: string;
  cpu: string;
  libc: "glibc" | "UNKNOWN" | "NOT_APPLICABLE";
  libcVersion: string | null;
  libcSource: string;
}
export function hostFacts(): HostFacts {
  const header = object(object(process.report.getReport()).header);
  const version =
    typeof header.glibcVersionRuntime === "string" ? header.glibcVersionRuntime : null;
  return {
    os: process.platform,
    cpu: process.arch,
    libc: process.platform !== "linux" ? "NOT_APPLICABLE" : version ? "glibc" : "UNKNOWN",
    libcVersion: version,
    libcSource: "Node process.report.header.glibcVersionRuntime; absence never implies musl",
  };
}
function stat(path: string) {
  try {
    return lstatSync(path);
  } catch (error) {
    if (typeof error === "object" && error !== null && "code" in error && error.code === "ENOENT")
      return null;
    throw error;
  }
}
const under = (root: string, path: string) => path.startsWith(root + "/") && resolve(path) === path;
/** Record real link/missing/partial facts before the strict classifier can fail. */
export function physicalPathWitness(root: string, path: string, parent: string, name: string) {
  assert(under(root, path) && (parent === root || under(root, parent)));
  assert.match(name, /^(?:@[a-z0-9._-]+\/)?[a-z0-9._-]+$/u);
  const marker = parent.lastIndexOf("/node_modules/");
  const aliasRoot =
    marker < 0 ? join(root, "node_modules") : parent.slice(0, marker + "/node_modules".length);
  const alias = join(aliasRoot, name);
  assert(under(root, alias));
  const paths = new Set<string>([root]);
  for (const target of [path, alias]) {
    let cursor = root;
    for (const part of relative(root, target).split("/")) {
      cursor = join(cursor, part);
      paths.add(cursor);
    }
  }
  return [...paths].map((file) => {
    try {
      const value = lstatSync(file);
      return {
        path: file,
        state: "PRESENT",
        kind: value.isSymbolicLink()
          ? "symlink"
          : value.isDirectory()
            ? "directory"
            : value.isFile()
              ? "file"
              : "special",
        link: value.isSymbolicLink() ? readlinkSync(file) : null,
      };
    } catch (error) {
      return {
        path: file,
        state: "UNAVAILABLE",
        code: typeof error === "object" && error !== null && "code" in error ? error.code : null,
        error: String(error),
      };
    }
  });
}
export interface PhysicalAbsence {
  state: "ABSENT";
  path: string;
  storeRoot: string;
  alias: string;
  checkedDirectories: string[];
}
export function absentPackage(
  root: string,
  path: string,
  parent: string,
  name: string,
  version: string,
): PhysicalAbsence {
  assert(under(root, path) && (parent === root || under(root, parent)));
  const storeRoot = join(root, "node_modules", ".pnpm", name.replaceAll("/", "+") + "@" + version);
  assert.equal(
    path,
    join(storeRoot, "node_modules", name),
    "only an exact declared pnpm store path may be absent",
  );
  const checkedDirectories = [root, join(root, "node_modules"), join(root, "node_modules/.pnpm")];
  for (const directory of checkedDirectories) {
    const value = stat(directory);
    assert(
      value?.isDirectory() && !value.isSymbolicLink(),
      `unsafe or missing store ancestor: ${directory}`,
    );
  }
  assert.equal(stat(storeRoot), null, "present/broken/partial package store cannot be waived");
  const marker = parent.lastIndexOf("/node_modules/");
  const aliasRoot =
    marker < 0 ? join(root, "node_modules") : parent.slice(0, marker + "/node_modules".length);
  const alias = join(aliasRoot, name);
  assert(under(root, alias));
  let cursor = root;
  for (const part of relative(root, alias).split("/")) {
    cursor = join(cursor, part);
    const value = stat(cursor);
    if (!value) break;
    assert(
      value.isDirectory() && !value.isSymbolicLink(),
      "existing or broken alias refuses absence",
    );
    assert.notEqual(cursor, alias, "present dependency alias cannot be recorded absent");
  }
  assert.equal(stat(alias), null, "present/broken dependency alias refuses absence");
  return { state: "ABSENT", path, storeRoot, alias, checkedDirectories };
}
export function presentPackage(root: string, path: string): string | null {
  assert(under(root, path));
  const value = stat(path);
  if (!value) return null;
  assert(value.isDirectory() || value.isSymbolicLink(), "package path must be a directory");
  const actual = realpathSync(path); // A present dangling link must throw, not become absence.
  assert(under(root, actual));
  assert(lstatSync(actual).isDirectory());
  return actual;
}
function allowed(selectors: unknown, actual: string | null): boolean | null {
  if (selectors === undefined) return true;
  const values = array(selectors).map(text);
  assert(values.length > 0 && new Set(values).size === values.length);
  assert(
    values.every((value) => /^!?[a-z0-9_-]+$/u.test(value)),
    "unsupported platform selector",
  );
  if (actual === null) return null;
  if (values.includes("!" + actual)) return false;
  const positive = values.filter((value) => !value.startsWith("!"));
  return positive.length === 0 || positive.includes("any") || positive.includes(actual);
}
export function platformExclusion(metadata: Record<string, unknown>, host: HostFacts): string[] {
  const proofs: string[] = [];
  for (const [key, actual] of [
    ["os", host.os],
    ["cpu", host.cpu],
    ["libc", host.libc === "glibc" ? "glibc" : null],
  ] as const) {
    const result = allowed(metadata[key], actual);
    if (result === false) proofs.push(key);
  }
  assert(proofs.length > 0, "no published constraint proves incompatibility with this actual host");
  return proofs;
}
export interface OptionalAbsence {
  state: "DECLARED_OPTIONAL_NOT_INSTALLED";
  name: string;
  version: string;
  parentDirectory: string;
  parentName: string;
  parentVersion: string;
  parentManifestRawBase64: string;
  parentManifestSha256: string;
  physical: PhysicalAbsence;
  metadata: PublicMetadata;
  lock: ReturnType<typeof lockPackageIntegrity>;
  host: HostFacts;
  excludedBy: string[];
  nativeLoadCredit: false;
}
export function validateOptionalAbsence(
  row: OptionalAbsence,
  root: string,
  lock: Buffer,
  host: HostFacts,
  packages: readonly {
    name: string;
    version: string;
    directory: string;
    files: readonly { path: string; sha256: string }[];
  }[],
): void {
  assert.equal(row.state, "DECLARED_OPTIONAL_NOT_INSTALLED");
  assert.equal(row.nativeLoadCredit, false);
  assert.deepEqual(row.host, host);
  const parents = packages.filter(
    (pkg) =>
      pkg.directory === row.parentDirectory &&
      pkg.name === row.parentName &&
      pkg.version === row.parentVersion,
  );
  assert.equal(parents.length, 1, "actual authenticated installed parent required");
  const parent = bytes(row.parentManifestRawBase64);
  assert.equal(sha256(parent), row.parentManifestSha256);
  assert(
    parents[0]!.files.some(
      (file) =>
        file.path === join(row.parentDirectory, "package.json") && file.sha256 === sha256(parent),
    ),
  );
  const manifest = object(wholeJson(parent));
  assert.equal(manifest.name, row.parentName);
  assert.equal(manifest.version, row.parentVersion);
  assert.equal(
    object(manifest.optionalDependencies)[row.name],
    row.version,
    "an exact packed-parent optional version is required",
  );
  const metadata = object(wholeJson(bytes(row.metadata.rawBase64)));
  assert.equal(sha256(bytes(row.metadata.rawBase64)), row.metadata.rawSha256);
  assert.deepEqual(metadata, row.metadata.metadata);
  assert.equal(metadata.name, row.name);
  assert.equal(metadata.version, row.version);
  assert.equal(row.metadata.name, row.name);
  assert.equal(row.metadata.version, row.version);
  assert.equal(
    row.metadata.url,
    `https://registry.npmjs.org/${encodeURIComponent(row.name)}/${row.version}`,
  );
  assert.equal(row.metadata.status, 200);
  assert.equal(object(metadata.dist).integrity, row.metadata.integrity);
  assert.equal(object(metadata.dist).tarball, row.metadata.resolved);
  sriSha512(row.metadata.integrity);
  assert.deepEqual(lockPackageIntegrity(lock, row.name, row.version), row.lock);
  assert.equal(row.lock.integrity, row.metadata.integrity);
  assert.deepEqual(platformExclusion(metadata, host), row.excludedBy);
  const expectedStore = join(
    root,
    "node_modules/.pnpm",
    row.name.replaceAll("/", "+") + "@" + row.version,
  );
  assert.equal(row.physical.state, "ABSENT");
  assert.equal(row.physical.storeRoot, expectedStore);
  assert.equal(row.physical.path, join(expectedStore, "node_modules", row.name));
  const marker = row.parentDirectory.lastIndexOf("/node_modules/");
  const aliasRoot =
    marker < 0
      ? join(root, "node_modules")
      : row.parentDirectory.slice(0, marker + "/node_modules".length);
  assert.equal(row.physical.alias, join(aliasRoot, row.name));
  assert.deepEqual(row.physical.checkedDirectories, [
    root,
    join(root, "node_modules"),
    join(root, "node_modules/.pnpm"),
  ]);
  assert(
    !packages.some((pkg) => pkg.directory === row.physical.path),
    "absent declarations are not installed packages",
  );
}
export function checkHostFacts(host: HostFacts, os: string, cpu: string): void {
  assert.equal(host.os, os);
  assert.equal(host.cpu, cpu);
  assert.equal(
    host.libcSource,
    "Node process.report.header.glibcVersionRuntime; absence never implies musl",
  );
  if (os !== "linux") {
    assert.equal(host.libc, "NOT_APPLICABLE");
    assert.equal(host.libcVersion, null);
  } else if (host.libc === "glibc") assert.match(text(host.libcVersion), /^\d+\.\d+(?:\.\d+)?$/u);
  else {
    assert.equal(host.libc, "UNKNOWN");
    assert.equal(host.libcVersion, null);
  }
}
export interface ManagerFile {
  path: string;
  state: "PRESENT" | "ABSENT";
  rawBase64: string | null;
  sha256: string | null;
}
export function managerFiles(root: string): ManagerFile[] {
  return ["package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml", ".npmrc"].map((path) => {
    const target = join(root, path),
      value = stat(target);
    if (!value) return { path, state: "ABSENT", rawBase64: null, sha256: null };
    assert(value.isFile() && !value.isSymbolicLink());
    assert(value.size <= 4 * 1024 * 1024);
    const raw = readFileSync(target);
    return { path, state: "PRESENT", rawBase64: raw.toString("base64"), sha256: sha256(raw) };
  });
}
export function checkManagerFiles(files: ManagerFile[], lock: Buffer, rootManifest: Buffer): void {
  assert.deepEqual(
    files.map((file) => file.path),
    ["package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml", ".npmrc"],
  );
  for (const file of files) {
    if (file.state === "ABSENT") {
      assert.equal(file.rawBase64, null);
      assert.equal(file.sha256, null);
      continue;
    }
    assert.equal(file.state, "PRESENT");
    assert.equal(sha256(bytes(text(file.rawBase64))), file.sha256);
  }
  assert(bytes(text(files[0]!.rawBase64)).equals(rootManifest));
  assert(bytes(text(files[1]!.rawBase64)).equals(lock));
}
