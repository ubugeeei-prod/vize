// Cargo decides freshness from source mtimes, and a fresh checkout stamps every
// file with the checkout time. A warm target directory then rebuilds every
// workspace crate. This emulates the persistent working tree that Cargo's
// model assumes: a trusted run records each tracked file's content and the mtime
// it was given on the target disk; later runs give a file its recorded mtime
// only when the content is identical, and any other file a time newer than
// every artifact on the disk. Cargo then rebuilds exactly what a developer's
// persistent checkout would rebuild.
import { execFileSync } from "node:child_process";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  renameSync,
  rmSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import type { BigIntStats, Dirent } from "node:fs";
import { dirname, join, resolve } from "node:path";

export const manifestName = ".vize-source-mtimes.json";
const schema = "vize.rust-source-mtimes";
const version = 1;
const blob = /^[0-9a-f]{40}(?:[0-9a-f]{24})?$/;
// Cargo leaves these large directories under each profile; freshness evidence
// lives only in `.fingerprint`.
const skippedTargetDirectories = new Set(["build", "deps", "examples", "incremental", "doc"]);

/** A tracked file's content (Git blob ID) and the whole-second mtime it was given. */
type FileRecord = [blob: string, mtime: number];

export interface SourceMtimesManifest {
  schema: typeof schema;
  version: typeof version;
  root: string;
  state: "pending" | "complete";
  updatedAt: number;
  files: Record<string, FileRecord>;
}

export type RestoreResult =
  | { applied: false; reason: string }
  | {
      applied: true;
      reason: string;
      root: string;
      floor: number;
      restored: number;
      changed: number;
      invalidated: number;
      invalidation: string;
      manifest: string;
    };

export type RecordResult =
  | { recorded: false; reason: string }
  | { recorded: true; rewritten: number; files: number };

const git = (root: string, args: string[], input?: string): string =>
  execFileSync("git", ["-C", root, ...args], {
    encoding: "utf8",
    input,
    maxBuffer: 256 * 1024 * 1024,
    stdio: ["pipe", "pipe", "pipe"],
  });

export function workTree(path: string): string | null {
  let directory = resolve(path);
  while (!existsSync(directory)) directory = dirname(directory);
  try {
    return git(directory, ["rev-parse", "--show-toplevel"]).trim();
  } catch {
    return null;
  }
}

const exactSeconds = (stat: BigIntStats, value: number) =>
  stat.mtimeNs === BigInt(value) * 1_000_000_000n;
const ceilSeconds = (stat: BigIntStats) => Number((stat.mtimeNs + 999_999_999n) / 1_000_000_000n);

function visitFingerprintDirectories(target: string, visitor: (path: string) => void): void {
  const visit = (directory: string, depth: number) => {
    let entries: Dirent[];
    try {
      entries = readdirSync(directory, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      if (!entry.isDirectory()) continue;
      const path = join(directory, entry.name);
      if (entry.name === ".fingerprint") {
        visitor(path);
      } else if (depth < 4 && !skippedTargetDirectories.has(entry.name)) {
        visit(path, depth + 1);
      }
    }
  };
  visit(target, 0);
}

/** The newest fingerprint written into a target directory, in whole seconds. */
export function newestFingerprint(target: string): number {
  let newest = 0;
  visitFingerprintDirectories(target, (path) => {
    for (const unit of readdirSync(path)) {
      const unitPath = join(path, unit);
      const unitStat = lstatSync(unitPath, { bigint: true });
      newest = Math.max(newest, ceilSeconds(unitStat));
      // Cargo writes freshness evidence after creating the unit directory.
      // The directory mtime therefore does not bound the mtimes of its
      // files, especially when a build crosses a whole-second boundary.
      if (!unitStat.isDirectory()) continue;
      for (const evidence of readdirSync(unitPath)) {
        newest = Math.max(
          newest,
          ceilSeconds(lstatSync(join(unitPath, evidence), { bigint: true })),
        );
      }
    }
  });
  return newest;
}

/** Keep dependency artifacts, but never trust workspace outputs without custody. */
function invalidateUnownedWorkspaceArtifacts(
  root: string,
  target: string,
): { removed: number; reason: string } {
  if (newestFingerprint(target) === 0) return { removed: 0, reason: "no fingerprints" };
  let owners: Set<string>;
  try {
    const metadata = JSON.parse(
      execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
        cwd: root,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      }),
    ) as {
      packages: Array<{ id: string; name: string; targets: Array<{ name: string }> }>;
      workspace_members: string[];
    };
    const members = new Set(metadata.workspace_members);
    const names = metadata.packages
      .filter(({ id }) => members.has(id))
      .flatMap(({ name, targets }) => [name, ...targets.map((target) => target.name)]);
    owners = new Set(names.flatMap((name) => [name, name.replaceAll("-", "_")]));
  } catch (error) {
    // Source mtimes remain the conservative fallback outside a Cargo workspace.
    const detail = error instanceof Error ? error.message.split("\n", 1)[0] : String(error);
    return { removed: 0, reason: `metadata unavailable: ${detail}` };
  }
  let removed = 0;
  const seen: string[] = [];
  visitFingerprintDirectories(target, (path) => {
    for (const unit of readdirSync(path)) {
      if (seen.length < 5) seen.push(unit);
      const match = /^(.*)-[0-9a-f]{16}$/u.exec(unit);
      if (match?.[1] && owners.has(match[1])) {
        rmSync(join(path, unit), { recursive: true });
        removed++;
      }
    }
  });
  const reason =
    removed > 0
      ? `removed ${removed} workspace fingerprint(s)`
      : `no owner match; owners=${[...owners].slice(0, 5).join(",")}; units=${seen.join(",")}`;
  return { removed, reason };
}

function isObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function readManifest(
  file: string,
  root: string,
): { manifest: SourceMtimesManifest | null; reason: string } {
  let value: unknown;
  try {
    value = JSON.parse(readFileSync(file, "utf8"));
  } catch (error) {
    const code = isObject(error) ? error.code : undefined;
    return { manifest: null, reason: code === "ENOENT" ? "no manifest" : "unreadable" };
  }
  if (
    !isObject(value) ||
    value.schema !== schema ||
    value.version !== version ||
    value.root !== root ||
    !Number.isSafeInteger(value.updatedAt) ||
    !isObject(value.files)
  )
    return { manifest: null, reason: "incompatible manifest" };
  // A run that never finished may have changed files after recording them.
  if (value.state !== "complete") return { manifest: null, reason: "unfinished manifest" };
  return { manifest: value as unknown as SourceMtimesManifest, reason: "complete manifest" };
}

function writeManifest(file: string, manifest: SourceMtimesManifest) {
  mkdirSync(dirname(file), { recursive: true });
  const temporary = `${file}.${process.pid}.tmp`;
  writeFileSync(temporary, JSON.stringify(manifest));
  renameSync(temporary, file);
}

/** Tracked regular files at stage 0 with their index blob. */
function trackedFiles(root: string): Array<[path: string, blob: string]> {
  const files: Array<[string, string]> = [];
  for (const record of git(root, ["ls-files", "-s", "-z"]).split("\0")) {
    if (!record) continue;
    const tab = record.indexOf("\t");
    const [mode, object, stage] = record.slice(0, tab).split(" ");
    if ((mode !== "100644" && mode !== "100755") || stage !== "0" || !blob.test(object ?? ""))
      continue;
    files.push([record.slice(tab + 1), object as string]);
  }
  return files;
}

function hashObjects(root: string, paths: string[]): string[] {
  if (paths.length === 0) return [];
  const hashes = git(root, ["hash-object", "--stdin-paths"], `${paths.join("\n")}\n`)
    .trim()
    .split("\n");
  if (hashes.length !== paths.length || !hashes.every((hash) => blob.test(hash)))
    throw new Error("git hash-object returned an unexpected result");
  return hashes;
}

/**
 * Restores recorded mtimes before Cargo runs. When a manifest cannot authorize
 * reuse, every tracked source moves past the newest reused fingerprint.
 */
export function restoreSourceMtimes({
  target,
  secondary = "",
  record,
  now = Date.now(),
}: {
  target: string;
  secondary?: string;
  record: boolean;
  now?: number;
}): RestoreResult {
  const targetPath = resolve(target);
  const file = join(targetPath, manifestName);
  const root = workTree(dirname(targetPath));
  const skip = (reason: string): RestoreResult => {
    // A committed disk must never keep records that this run's builds invalidate.
    if (record) rmSync(file, { force: true });
    return { applied: false, reason };
  };
  if (!root) return skip("target is outside a Git work tree");
  const secondaryPath = secondary ? resolve(secondary) : "";
  const sharedSecondary = secondaryPath && workTree(dirname(secondaryPath)) === root;
  const { manifest, reason } = sharedSecondary
    ? { manifest: null, reason: "another target directory shares this work tree" }
    : readManifest(file, root);
  let invalidated = 0;
  let invalidation = "authorized manifest";
  if (!manifest) {
    const outcomes = [invalidateUnownedWorkspaceArtifacts(root, targetPath)];
    if (sharedSecondary) outcomes.push(invalidateUnownedWorkspaceArtifacts(root, secondaryPath));
    invalidated = outcomes.reduce((total, outcome) => total + outcome.removed, 0);
    invalidation = outcomes.map((outcome) => outcome.reason).join("; ");
  }
  const recorded = manifest?.files ?? {};
  // Whole seconds keep restored values exact; the fingerprint and manifest
  // bounds, not the clock, keep a changed file newer than every artifact.
  const floor = Math.max(
    Math.floor(now / 1000),
    (manifest?.updatedAt ?? 0) + 1,
    newestFingerprint(targetPath) + 1,
    sharedSecondary ? newestFingerprint(secondaryPath) + 1 : 0,
  );
  // Content edited before this step does not match its index blob.
  const modified = new Set(git(root, ["ls-files", "-m", "-z"]).split("\0").filter(Boolean));
  const modifiedPaths = [...modified].filter((path) => existsSync(join(root, path)));
  const modifiedBlobs = new Map(
    hashObjects(root, modifiedPaths).map((hash, index) => [modifiedPaths[index], hash]),
  );
  const next: Record<string, FileRecord> = {};
  let restored = 0;
  let changed = 0;
  for (const [path, indexBlob] of trackedFiles(root)) {
    const absolute = join(root, path);
    try {
      if (!lstatSync(absolute).isFile()) continue;
    } catch {
      continue;
    }
    const content = modifiedBlobs.get(path) ?? (modified.has(path) ? null : indexBlob);
    const previous: unknown = recorded[path];
    const kept =
      content !== null &&
      Array.isArray(previous) &&
      previous[0] === content &&
      Number.isSafeInteger(previous[1]) &&
      (previous[1] as number) < floor
        ? (previous[1] as number)
        : null;
    const mtime = kept ?? floor;
    utimesSync(absolute, mtime, mtime);
    if (kept === null) changed++;
    else restored++;
    if (content !== null) next[path] = [content, mtime];
  }
  // Refresh the index stat cache once so later Git commands do not rehash.
  try {
    git(root, ["update-index", "-q", "--refresh"]);
  } catch {
    // Unmerged or modified entries are expected to remain dirty.
  }
  if (record)
    writeManifest(file, { schema, version, root, state: "pending", updatedAt: floor, files: next });
  return {
    applied: true,
    reason,
    root,
    floor,
    restored,
    changed,
    invalidated,
    invalidation,
    manifest: file,
  };
}

/**
 * Records the final working-tree state after a trusted run, before the sticky
 * disk is committed. Files whose mtime moved were rewritten during the run, so
 * their current content and mtime replace the earlier record.
 */
export function recordSourceMtimes({
  target,
  now = Date.now(),
}: {
  target: string;
  now?: number;
}): RecordResult {
  const file = join(resolve(target), manifestName);
  let value: unknown;
  try {
    value = JSON.parse(readFileSync(file, "utf8"));
  } catch {
    return { recorded: false, reason: "no manifest" };
  }
  if (
    !isObject(value) ||
    value.schema !== schema ||
    value.version !== version ||
    value.state !== "pending" ||
    typeof value.root !== "string" ||
    !Number.isSafeInteger(value.updatedAt) ||
    !isObject(value.files)
  ) {
    rmSync(file, { force: true });
    return { recorded: false, reason: "unexpected manifest" };
  }
  const manifest = value as unknown as SourceMtimesManifest;
  const { root, files } = manifest;
  const rewritten: Array<[string, BigIntStats]> = [];
  for (const [path, [, mtime]] of Object.entries(files)) {
    let stat: BigIntStats;
    try {
      stat = lstatSync(join(root, path), { bigint: true });
    } catch {
      delete files[path];
      continue;
    }
    if (!stat.isFile()) delete files[path];
    else if (!exactSeconds(stat, mtime)) rewritten.push([path, stat]);
  }
  const hashes = hashObjects(
    root,
    rewritten.map(([path]) => path),
  );
  let updatedAt = Math.max(Math.ceil(now / 1000), manifest.updatedAt);
  rewritten.forEach(([path, stat], index) => {
    const mtime = ceilSeconds(stat);
    files[path] = [hashes[index] as string, mtime];
    updatedAt = Math.max(updatedAt, mtime);
  });
  writeManifest(file, { ...manifest, state: "complete", updatedAt, files });
  return { recorded: true, rewritten: rewritten.length, files: Object.keys(files).length };
}
