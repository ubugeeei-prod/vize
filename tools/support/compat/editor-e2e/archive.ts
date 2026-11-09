import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { repositoryRoot } from "./real-vue-workspace.ts";

export interface TarPackageSpecification {
  readonly defaultArchive: string;
  readonly label: string;
  readonly root: string;
  readonly minimumBytes: number;
  readonly maximumBytes: number;
  readonly required: readonly string[];
  readonly allowed: readonly string[];
  readonly forbidden: readonly string[];
  readonly expectations: Readonly<Record<string, readonly string[]>>;
}

export interface TarPackage {
  readonly archive: string;
  readonly entries: readonly string[];
  read(entry: string): string;
}

function runTar(args: readonly string[], failure: string): string {
  const result = spawnSync("tar", [...args], {
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error !== undefined) throw new Error(`failed to run tar: ${result.error}`);
  if (result.status !== 0) throw new Error(`${failure}:\n${result.stdout}${result.stderr}`);
  return result.stdout;
}

function assertSafeEntries(entries: readonly string[], label: string, root: string): void {
  const prefix = `${root}/`;
  for (const entry of entries) {
    if (entry.includes("\\")) throw new Error(`${label} entry must use POSIX separators: ${entry}`);
    if (entry.includes("\0")) throw new Error(`${label} entry contains a NUL byte: ${entry}`);
    if (entry.startsWith("/")) throw new Error(`${label} entry must be relative: ${entry}`);
    if (entry.split("/").includes(".."))
      throw new Error(`${label} entry must not traverse: ${entry}`);
    if (entry !== prefix && !entry.startsWith(prefix)) throw new Error(`unexpected root: ${entry}`);
  }
}

function assertArchiveSize(archive: string, specification: TarPackageSpecification): void {
  let size: number;
  try {
    size = fs.statSync(archive).size;
  } catch (error) {
    throw new Error(`${specification.label} does not exist: ${archive} (${String(error)})`);
  }
  if (size <= specification.minimumBytes)
    throw new Error(`${specification.label} is suspiciously small: ${size} bytes`);
  if (size >= specification.maximumBytes)
    throw new Error(`${specification.label} is unexpectedly large: ${size} bytes`);
}

export function assertTarPackage(
  specification: TarPackageSpecification,
  argument = process.argv[2],
): TarPackage {
  const archive = path.resolve(argument ?? path.join(repositoryRoot, specification.defaultArchive));
  assertArchiveSize(archive, specification);
  const entries = runTar(["-tzf", archive], `tar -tzf failed for ${archive}`)
    .split(/\r?\n/u)
    .map((entry) => entry.trim())
    .filter(Boolean)
    .sort();
  if (new Set(entries).size !== entries.length)
    throw new Error(`${specification.label} has duplicates`);
  assertSafeEntries(entries, specification.label, specification.root);

  const entrySet = new Set(entries);
  const read = (entry: string): string =>
    runTar(["-xOzf", archive, entry], `missing tar entry: ${entry}`);
  for (const entry of specification.required) {
    if (!entrySet.has(entry))
      throw new Error(`${specification.label} is missing required file: ${entry}`);
    if (read(entry).trim().length === 0)
      throw new Error(`${specification.label} file is empty: ${entry}`);
  }
  const allowed = new Set(specification.allowed);
  for (const entry of entries) {
    if (!allowed.has(entry))
      throw new Error(`${specification.label} ships an unexpected file: ${entry}`);
    if (specification.forbidden.some((pattern) => entry.includes(pattern)))
      throw new Error(`${specification.label} must not ship ${entry}`);
  }
  for (const [entry, needles] of Object.entries(specification.expectations)) {
    const source = read(entry);
    for (const needle of needles) {
      if (!source.includes(needle))
        throw new Error(`${specification.label.replace(" archive", "")} package missing ${needle}`);
    }
  }
  return { archive, entries, read };
}

export function workspaceVersion(): string {
  const source = fs.readFileSync(path.join(repositoryRoot, "Cargo.toml"), "utf8");
  const version = /^version = "([^"]+)"$/mu.exec(source)?.[1];
  if (version === undefined) throw new Error("workspace version is missing from Cargo.toml");
  return version;
}

export function relativeArchivePath(archive: string): string {
  return path.relative(repositoryRoot, archive) || path.basename(archive);
}
