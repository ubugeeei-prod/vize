import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { isEntrypoint, runEntrypoint } from "../../../support/command-runtime.ts";

export const renamedPackages = [
  [
    "vize_l1",
    "vize_s1",
    "0.428.1",
    "f28076ad1b5a7adbe4439a57e158a99584dcd4e0d980e0f8b70c8650ce5e243b",
  ],
  [
    "vize_l2",
    "vize_s2",
    "0.428.1",
    "5fc5c0a5d6f0118ab070c76963a8be9e17e14ff5895f727d8fe0b9c428e3c18a",
  ],
  [
    "vize_l1_to_l2",
    "vize_s1_to_s2",
    "0.428.1",
    "92e2f8378bb827ae67f6471e08a6ac8aef40e6dfacdab5044449b1637031d241",
  ],
  [
    "vize_l2_to_l3",
    "vize_s2_to_s3",
    "0.428.1",
    "02aa69e3f234592083045ad076cf33f681c84505ccc571da452f6bcaf4227f45",
  ],
  [
    "vize_l3",
    "vize_impeto",
    "0.429.0",
    "d51929bec2e0ece251a468179854df74184b46126d764fb6a606a3579934badf",
  ],
] as const;

interface Version {
  readonly major: number;
  readonly minor: number;
  readonly patch: number;
  readonly prerelease: readonly string[];
}

function parseVersion(value: string): Version {
  const match = /^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/u.exec(value);
  if (match === null) throw new Error(`invalid semantic version: ${value}`);
  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
    prerelease: match[4]?.split(".") ?? [],
  };
}

function compareIdentifiers(left: string, right: string): number {
  const leftNumber = /^\d+$/u.test(left) ? Number(left) : undefined;
  const rightNumber = /^\d+$/u.test(right) ? Number(right) : undefined;
  if (leftNumber !== undefined && rightNumber !== undefined) return leftNumber - rightNumber;
  if (leftNumber !== undefined) return -1;
  if (rightNumber !== undefined) return 1;
  return left < right ? -1 : left > right ? 1 : 0;
}

export function compareVersions(leftValue: string, rightValue: string): number {
  const left = parseVersion(leftValue);
  const right = parseVersion(rightValue);
  for (const field of ["major", "minor", "patch"] as const) {
    if (left[field] !== right[field]) return left[field] - right[field];
  }
  if (left.prerelease.length === 0 || right.prerelease.length === 0) {
    return right.prerelease.length - left.prerelease.length;
  }
  const length = Math.max(left.prerelease.length, right.prerelease.length);
  for (let index = 0; index < length; index += 1) {
    const leftPart = left.prerelease[index];
    const rightPart = right.prerelease[index];
    if (leftPart === undefined) return -1;
    if (rightPart === undefined) return 1;
    const difference = compareIdentifiers(leftPart, rightPart);
    if (difference !== 0) return difference;
  }
  return 0;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function section(source: string, name: string): { body: string; start: number; end: number } {
  const header = new RegExp(`^\\[${escapeRegExp(name)}\\]\\s*$`, "mu").exec(source);
  if (header === null) throw new Error(`TOML section [${name}] is missing`);
  const start = header.index + header[0].length;
  const next = /^\[[^\]]+\]\s*$/gmu;
  next.lastIndex = start;
  const following = next.exec(source);
  const end = following?.index ?? source.length;
  return { body: source.slice(start, end), start, end };
}

function stringField(source: string, sectionName: string, field: string): string | undefined {
  const { body } = section(source, sectionName);
  const match = new RegExp(`^\\s*${escapeRegExp(field)}\\s*=\\s*"([^"]+)"\\s*(?:#.*)?$`, "mu").exec(
    body,
  );
  return match?.[1];
}

function replaceStringField(
  source: string,
  sectionName: string,
  field: string,
  expected: string,
  replacement: string,
): string {
  const target = section(source, sectionName);
  const pattern = new RegExp(
    `^(\\s*${escapeRegExp(field)}\\s*=\\s*)"${escapeRegExp(expected)}"`,
    "mu",
  );
  const matches = [...target.body.matchAll(new RegExp(pattern.source, "gmu"))];
  if (matches.length !== 1) throw new Error(`${sectionName}.${field} identity mismatch`);
  const body = target.body.replace(pattern, `$1"${replacement}"`);
  return source.slice(0, target.start) + body + source.slice(target.end);
}

function packageVersion(manifest: string, workspace: string): string {
  const direct = stringField(manifest, "package", "version");
  if (direct !== undefined) return direct;
  if (!/^\s*version\.workspace\s*=\s*true\s*(?:#.*)?$/mu.test(section(manifest, "package").body)) {
    throw new Error("current package version is missing");
  }
  const inherited = stringField(workspace, "workspace.package", "version");
  if (inherited === undefined) throw new Error("workspace package version is missing");
  return inherited;
}

export function adaptPublishedManifest(
  source: string,
  oldName: string,
  newName: string,
  oldVersion: string,
): string {
  if (
    stringField(source, "package", "name") !== oldName ||
    stringField(source, "package", "version") !== oldVersion ||
    stringField(source, "lib", "name") !== oldName
  ) {
    throw new Error("immutable baseline package/lib/version identity mismatch");
  }
  return replaceStringField(
    replaceStringField(source, "package", "name", oldName, newName),
    "lib",
    "name",
    oldName,
    newName,
  );
}

function adaptGitManifest(source: string, oldName: string, newName: string): string {
  if (stringField(source, "package", "name") !== oldName) {
    throw new Error("actual base old-package identity mismatch");
  }
  let adapted = replaceStringField(source, "package", "name", oldName, newName);
  try {
    const libraryName = stringField(adapted, "lib", "name");
    if (libraryName !== oldName) throw new Error("actual base library identity mismatch");
    adapted = replaceStringField(adapted, "lib", "name", oldName, newName);
  } catch (error) {
    if (!(error instanceof Error) || error.message !== "TOML section [lib] is missing") throw error;
  }
  return adapted;
}

export function adaptWorkspaceDependency(source: string, oldName: string, newName: string): string {
  const dependencies = section(source, "workspace.dependencies");
  const pattern = new RegExp(
    `^(\\s*${escapeRegExp(oldName)}\\s*=\\s*\\{)([^\\n]*)(\\}\\s*(?:#.*)?$)`,
    "mu",
  );
  const match = pattern.exec(dependencies.body);
  if (match === null) throw new Error("actual base workspace dependency lookup is missing");
  const details = match[2];
  const dependencyPath = /\bpath\s*=\s*"([^"]+)"/u.exec(details)?.[1];
  if (![`crates/${oldName}`, `davinci/${oldName}`].includes(dependencyPath ?? "")) {
    throw new Error("actual base workspace dependency path mismatch");
  }
  const packageName = /\bpackage\s*=\s*"([^"]+)"/u.exec(details)?.[1];
  if (packageName !== undefined && packageName !== oldName) {
    throw new Error("actual base workspace package identity mismatch");
  }
  const withoutPackage = details.replace(/\s*,?\s*package\s*=\s*"[^"]+"\s*/u, "").trimEnd();
  const separator = withoutPackage.trimEnd().endsWith(",") ? " " : ", ";
  const replacement = `${match[1]}${withoutPackage}${separator}package = "${newName}" ${match[3]}`;
  const body = dependencies.body.replace(pattern, replacement);
  return source.slice(0, dependencies.start) + body + source.slice(dependencies.end);
}

export function hasRegistryBaseline(index: string, packageName: string, current: string): boolean {
  let rows = 0;
  let found = false;
  for (const line of index.split(/\r?\n/u).filter(Boolean)) {
    let row: unknown;
    try {
      row = JSON.parse(line);
    } catch (error) {
      throw new Error(`invalid registry index: ${String(error)}`);
    }
    if (typeof row !== "object" || row === null || Array.isArray(row)) {
      throw new Error("registry index package identity/schema mismatch");
    }
    const record = row as Record<string, unknown>;
    if (record.name !== packageName || typeof record.yanked !== "boolean") {
      throw new Error("registry index package identity/schema mismatch");
    }
    if (typeof record.vers !== "string") throw new Error("registry version is missing");
    found ||= compareVersions(record.vers, current) <= 0;
    rows += 1;
  }
  if (rows === 0) throw new Error("registry index is empty");
  return found;
}

export function verifyChecksum(bytes: Uint8Array, expected: string): void {
  if (createHash("sha256").update(bytes).digest("hex") !== expected) {
    throw new Error("immutable published archive checksum mismatch");
  }
}

function capture(
  command: string,
  args: readonly string[],
): { status: number; stdout: string; stderr: string } {
  const result = spawnSync(command, [...args], { encoding: "utf8" });
  if (result.error !== undefined) throw result.error;
  return { status: result.status ?? 1, stdout: result.stdout, stderr: result.stderr };
}

function requireSuccess(command: string, args: readonly string[], message: string): string {
  const result = capture(command, args);
  if (result.status !== 0) throw new Error(`${message}: ${result.stderr}`);
  return result.stdout;
}

function download(url: string, destination: string): number {
  const result = capture("curl", [
    "--silent",
    "--show-error",
    "--location",
    "--fail-with-body",
    "--connect-timeout",
    "5",
    "--max-time",
    "15",
    "--retry",
    "2",
    "--retry-delay",
    "1",
    "--output",
    destination,
    "--write-out",
    "%{http_code}",
    url,
  ]);
  const status = Number(result.stdout);
  if (!Number.isInteger(status)) throw new Error("invalid registry HTTP status");
  if (result.status !== 0 && !(result.status === 22 && status === 404)) {
    throw new Error(`registry request failed (HTTP ${status}): ${result.stderr}`);
  }
  return status;
}

function prepareGitBaseline(
  packageName: string,
  oldName: string,
  destination: string,
  base: string,
): string | undefined {
  const verified = capture("git", ["rev-parse", "--verify", `${base}^{commit}`]);
  if (verified.status !== 0) throw new Error("actual Git base is unavailable");
  const sha = verified.stdout.trim();
  for (const directory of ["crates", "davinci"]) {
    const current = capture("git", ["show", `${sha}:${directory}/${packageName}/Cargo.toml`]);
    if (current.status === 0) {
      if (stringField(current.stdout, "package", "name") !== packageName) {
        throw new Error("actual base new-package manifest identity mismatch");
      }
      return undefined;
    }
  }

  const archive = path.join(destination, "actual-base.tar");
  requireSuccess(
    "git",
    ["archive", "--format=tar", "--output", archive, sha],
    "cannot archive actual base",
  );
  requireSuccess("tar", ["-xf", archive, "-C", destination], "cannot extract actual Git base");
  const root = ["crates", "davinci"]
    .map((directory) => path.join(destination, directory, oldName))
    .find((candidate) => fs.existsSync(path.join(candidate, "Cargo.toml")));
  if (root === undefined) throw new Error("actual base package manifest is missing");
  const manifestPath = path.join(root, "Cargo.toml");
  const workspacePath = path.join(destination, "Cargo.toml");
  fs.writeFileSync(
    manifestPath,
    adaptGitManifest(fs.readFileSync(manifestPath, "utf8"), oldName, packageName),
  );
  fs.writeFileSync(
    workspacePath,
    adaptWorkspaceDependency(fs.readFileSync(workspacePath, "utf8"), oldName, packageName),
  );
  process.stderr.write(
    `SemVer baseline: exact Git parent ${sha}, ${oldName} -> ${packageName}; package/lib and workspace package lookup identity only, Rust API/dependency keys/versions unchanged\n`,
  );
  return root;
}

function run(args: readonly string[]): void {
  const [packageName, destinationInput, ...base] = args;
  if (packageName === undefined || destinationInput === undefined) {
    throw new Error(
      "Usage: node tools/commands/ci/github/semver-baseline.ts <package> <empty-directory> [actual-base-revision]",
    );
  }
  if (base.length > 1) throw new Error("expected at most one actual base revision");
  const rename = renamedPackages.find(([name]) => name === packageName);
  if (rename === undefined) return;
  const [, oldName, oldVersion, checksum] = rename;
  const manifestPath = ["crates", "davinci"]
    .map((directory) => path.join(directory, packageName, "Cargo.toml"))
    .find((candidate) => fs.existsSync(candidate));
  if (manifestPath === undefined) throw new Error("current package manifest is missing");
  const currentManifest = fs.readFileSync(manifestPath, "utf8");
  if (stringField(currentManifest, "package", "name") !== packageName) {
    throw new Error("current package identity mismatch");
  }
  const currentVersion = packageVersion(currentManifest, fs.readFileSync("Cargo.toml", "utf8"));
  const destination = path.resolve(destinationInput);
  if (
    !fs.statSync(destination, { throwIfNoEntry: false })?.isDirectory() ||
    fs.readdirSync(destination).length !== 0
  ) {
    throw new Error("baseline destination must be an empty directory");
  }
  if (base[0] !== undefined) {
    const root = prepareGitBaseline(packageName, oldName, destination, base[0]);
    if (root !== undefined) process.stdout.write(`${root}\n`);
    return;
  }

  const indexPath = path.join(destination, "index.jsonl");
  const indexStatus = download(`https://index.crates.io/vi/ze/${packageName}`, indexPath);
  if (
    indexStatus === 200 &&
    hasRegistryBaseline(fs.readFileSync(indexPath, "utf8"), packageName, currentVersion)
  )
    return;
  if (indexStatus !== 200 && indexStatus !== 404) {
    throw new Error(`registry index returned HTTP ${indexStatus}`);
  }
  if (compareVersions(currentVersion, oldVersion) < 0) {
    throw new Error(`current version predates the immutable ${oldVersion} baseline`);
  }
  const archive = path.join(destination, "baseline.crate");
  const archiveStatus = download(
    `https://static.crates.io/crates/${oldName}/${oldName}-${oldVersion}.crate`,
    archive,
  );
  if (archiveStatus !== 200)
    throw new Error(`old published archive returned HTTP ${archiveStatus}`);
  verifyChecksum(fs.readFileSync(archive), checksum);
  requireSuccess(
    "tar",
    ["-xzf", archive, "-C", destination],
    "cannot extract immutable published archive",
  );
  const root = path.join(destination, `${oldName}-${oldVersion}`);
  const baselineManifest = path.join(root, "Cargo.toml");
  fs.writeFileSync(
    baselineManifest,
    adaptPublishedManifest(
      fs.readFileSync(baselineManifest, "utf8"),
      oldName,
      packageName,
      oldVersion,
    ),
  );
  process.stderr.write(
    `SemVer baseline: ${oldName}@${oldVersion}, SHA256 ${checksum}; package/lib name -> ${packageName}, source and version unchanged\n`,
  );
  process.stdout.write(`${root}\n`);
}

function main(): number {
  try {
    run(process.argv.slice(2));
    return 0;
  } catch (error) {
    process.stderr.write(
      `SemVer baseline: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    return 1;
  }
}

if (isEntrypoint(import.meta.url)) runEntrypoint(main);
