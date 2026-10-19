// Whole-body identity and independent-file restoration for the opt-in ZIP trial.
import { createHash } from "node:crypto";
import { closeSync, createReadStream, lstatSync, readdirSync, writeSync } from "node:fs";
import { join, posix, relative, sep } from "node:path";
import type { ZipEntry } from "./artifact-payload-zip.ts";

export const SCHEMA = "vize.artifact-payload.v1";

const S_IFMT = 0o170000;
const S_IFDIR = 0o040000;
const S_IFREG = 0o100000;
const S_IFLNK = 0o120000;

export type Json =
  | null
  | boolean
  | number
  | string
  | readonly Json[]
  | { readonly [key: string]: Json };
export type JsonObject = { readonly [key: string]: Json };

export interface ManifestRow {
  readonly path: string;
  readonly kind: "file" | "directory";
  readonly mode: number;
  readonly externalAttr: number;
  readonly createSystem: number;
  readonly sha256?: string;
  readonly bytes?: number;
}

export interface Manifest {
  readonly schema: string;
  readonly sourceSha: string;
  readonly driverSha: string;
  readonly artifactId: number;
  readonly originalZipSha256: string;
  readonly entries: readonly ManifestRow[];
}

export class PayloadError extends Error {}

export function ensure(condition: unknown, message: string): asserts condition {
  if (!condition) throw new PayloadError(message);
}

export function isObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function field(value: unknown, key: string): Json {
  ensure(isObject(value) && key in value, `missing field ${key}`);
  return value[key] ?? null;
}

export function text(value: unknown, key: string): string {
  const result = field(value, key);
  ensure(typeof result === "string", `field ${key} must be a string`);
  return result;
}

export function integer(value: unknown, key: string): number {
  const result = field(value, key);
  ensure(typeof result === "number" && Number.isInteger(result), `field ${key} must be an integer`);
  return result;
}

/** Sha256 hex and byte count of a chunk stream, optionally teed into a file descriptor. */
export async function digestStream(
  stream: AsyncIterable<Uint8Array>,
  output?: number,
): Promise<{ sha: string; size: number }> {
  const digest = createHash("sha256");
  let size = 0;
  for await (const chunk of stream) {
    digest.update(chunk);
    size += chunk.length;
    if (output !== undefined) {
      for (let done = 0; done < chunk.length;) done += writeSync(output, chunk, done);
    }
  }
  return { sha: digest.digest("hex"), size };
}

export async function digestFile(path: string): Promise<string> {
  return (await digestStream(createReadStream(path, { highWaterMark: 1024 * 1024 }))).sha;
}

export async function digestToFile(
  stream: AsyncIterable<Uint8Array>,
  fd: number,
): Promise<{ sha: string; size: number }> {
  try {
    return await digestStream(stream, fd);
  } finally {
    closeSync(fd);
  }
}

function sorted(value: Json): Json {
  if (Array.isArray(value)) return value.map(sorted);
  if (!isObject(value)) return value;
  return Object.fromEntries(
    Object.keys(value)
      .sort(byCodePoint)
      .map((key) => [key, sorted(value[key] ?? null)]),
  );
}

export function byCodePoint(left: string, right: string): number {
  return Buffer.compare(Buffer.from(left), Buffer.from(right));
}

export function canonicalJson(value: Json): string {
  return JSON.stringify(sorted(value));
}

export function jsonBytes(value: Json): Buffer {
  return Buffer.from(`${canonicalJson(value)}\n`);
}

export function safeName(name: unknown, directory = false): string {
  ensure(typeof name === "string" && name !== "" && !name.includes("\0"), "invalid path");
  const raw = directory && name.endsWith("/") ? name.slice(0, -1) : name;
  const parts = raw.split("/");
  ensure(
    !raw.startsWith("/") &&
      !parts.some((part) => part === "" || part === "." || part === "..") &&
      !raw.includes("\\"),
    "unsafe artifact path",
  );
  return raw;
}

export function zipRow(info: ZipEntry): ManifestRow {
  const directory = info.name.endsWith("/");
  const name = safeName(info.name, directory);
  const mode = info.externalAttr >>> 16;
  const type = mode & S_IFMT;
  ensure(
    type === 0 || type === (directory ? S_IFDIR : S_IFREG),
    "unsupported original ZIP entry type",
  );
  ensure(!directory || info.size === 0, "directory contains file bytes");
  return {
    path: name,
    kind: directory ? "directory" : "file",
    mode: mode ? mode & 0o7777 : directory ? 0o755 : 0o644,
    // Retain the entire original permission/type metadata, even where ZIP has none.
    externalAttr: info.externalAttr,
    createSystem: info.createSystem,
  };
}

function parents(name: string): string[] {
  const result: string[] = [];
  for (let parent = posix.dirname(name); parent !== "."; parent = posix.dirname(parent)) {
    result.push(parent);
  }
  return result;
}

export function parseManifest(value: unknown): Manifest {
  const entries = field(value, "entries");
  ensure(Array.isArray(entries), "manifest entries must be a list");
  return {
    schema: text(value, "schema"),
    sourceSha: text(value, "sourceSha"),
    driverSha: text(value, "driverSha"),
    artifactId: integer(value, "artifactId"),
    originalZipSha256: text(value, "originalZipSha256"),
    entries: entries.map(parseRow),
  };
}

function parseRow(row: Json): ManifestRow {
  const kind = text(row, "kind");
  ensure(kind === "file" || kind === "directory", "invalid entry kind");
  const mode = field(row, "mode");
  ensure(
    typeof mode === "number" && Number.isInteger(mode) && mode >= 0 && mode <= 0o7777,
    "invalid mode",
  );
  const base = {
    path: safeName(field(row, "path")),
    kind,
    mode,
    externalAttr: isObject(row) && "externalAttr" in row ? integer(row, "externalAttr") : 0,
    createSystem: isObject(row) && "createSystem" in row ? integer(row, "createSystem") : 0,
  } as const;
  if (kind === "directory") return base;
  const sha = field(row, "sha256");
  ensure(typeof sha === "string" && /^[0-9a-f]{64}$/.test(sha), "invalid digest");
  const bytes = field(row, "bytes");
  ensure(typeof bytes === "number" && Number.isInteger(bytes) && bytes >= 0, "invalid byte count");
  return { ...base, sha256: sha, bytes };
}

export function validateManifest(manifest: Manifest, receipt: unknown): Map<string, number> {
  ensure(
    manifest.schema === field(receipt, "schema") &&
      manifest.sourceSha === field(receipt, "sourceSha") &&
      manifest.driverSha === field(receipt, "driverSha") &&
      manifest.artifactId === field(receipt, "artifactId"),
    "manifest provenance differs",
  );
  ensure(manifest.schema === SCHEMA, "unsupported payload schema");
  const entries = manifest.entries;
  ensure(entries.length === field(receipt, "entryCount"), "manifest entry count differs");
  const names = new Set<string>();
  const objects = new Map<string, number>();
  for (const row of entries) {
    ensure(!names.has(row.path), "duplicate artifact path");
    names.add(row.path);
    if (row.kind === "file" && row.sha256 !== undefined && row.bytes !== undefined) {
      const known = objects.get(row.sha256);
      ensure(known === undefined || known === row.bytes, "object size differs");
      objects.set(row.sha256, row.bytes);
    }
  }
  const fileNames = new Set(entries.filter((row) => row.kind === "file").map((row) => row.path));
  for (const name of names) {
    ensure(!parents(name).some((parent) => fileNames.has(parent)), "file shadows parent directory");
  }
  const rawBytes = entries.reduce((total, row) => total + (row.bytes ?? 0), 0);
  ensure(rawBytes === field(receipt, "rawBytes"), "raw byte count differs");
  ensure(objects.size === field(receipt, "objectCount"), "object count differs");
  return objects;
}

export async function verifyTree(
  root: string,
  manifest: { readonly entries: readonly ManifestRow[] },
): Promise<{ entries: number; independentFiles: number }> {
  ensure(
    (lstatSync(root).mode & S_IFMT) === S_IFDIR,
    "restored root must be an independent directory",
  );
  const expected = new Map(manifest.entries.map((row) => [row.path, row]));
  const implicit = new Set<string>();
  for (const name of expected.keys()) for (const parent of parents(name)) implicit.add(parent);
  const actual = new Set<string>();
  const inodes = new Set<string>();
  const pending = [root];
  for (let directory = pending.shift(); directory !== undefined; directory = pending.shift()) {
    let names: string[];
    try {
      names = readdirSync(directory);
    } catch (error) {
      throw new PayloadError("cannot inspect complete restored inventory", { cause: error });
    }
    for (const name of names) {
      const path = join(directory, name);
      const logical = relative(root, path).split(sep).join("/");
      const metadata = lstatSync(path, { bigint: true });
      const type = Number(metadata.mode) & S_IFMT;
      ensure(type !== S_IFLNK, "restored symlink is forbidden");
      if (type === S_IFDIR) pending.push(path);
      const row = expected.get(logical);
      ensure(row !== undefined || implicit.has(logical), "unexpected restored path");
      if (row === undefined) {
        ensure(type === S_IFDIR, "implicit parent is not a directory");
        continue;
      }
      actual.add(logical);
      ensure((Number(metadata.mode) & 0o7777) === row.mode, "restored mode differs");
      if (row.kind === "directory") {
        ensure(type === S_IFDIR, "restored directory differs");
      } else {
        ensure(type === S_IFREG, "restored file differs");
        ensure(metadata.nlink === 1n, "restored file has mutable aliases");
        const identity = `${metadata.dev}:${metadata.ino}`;
        ensure(!inodes.has(identity), "restored files share an inode");
        inodes.add(identity);
        ensure(
          metadata.size === BigInt(row.bytes ?? -1) && (await digestFile(path)) === row.sha256,
          "restored file body differs",
        );
      }
    }
  }
  ensure(actual.size === expected.size, "missing restored path");
  return { entries: actual.size, independentFiles: inodes.size };
}
