// Stage the exact official Router provider without installing dependencies.
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";

export const PROVIDER_URL = "https://registry.npmjs.org/vue-router/-/vue-router-5.1.0.tgz";
export const PROVIDER_INTEGRITY =
  "HAbiLzLEHQwxPgvsbOJDAwtavszEgLwri6XfyrsPECIFez8+59xc9LofWVdc/HEaSRT822lJ8H9Ns38VVond5g==";
const SOURCE = "c0e3226dabccd7596b996ce851386997ea2d3cca";
export const MAX_BYTES = 2_000_000;
const BLOCK = 512;

/** A refused archive, package or destination. Nothing is written after one. */
export class ProviderRefusal extends Error {}

export interface ProviderFile {
  readonly path: string;
  readonly body: Buffer;
  readonly mode: number;
}

interface TarMember {
  readonly name: string;
  readonly linkname: string;
  readonly type: string;
  readonly mode: number;
  readonly size: number;
  readonly body: Buffer;
}

function field(header: Buffer, start: number, length: number): string {
  const bytes = header.subarray(start, start + length);
  const end = bytes.indexOf(0);
  return bytes.subarray(0, end < 0 ? bytes.length : end).toString("utf8");
}

function numeric(header: Buffer, start: number, length: number): number {
  const bytes = header.subarray(start, start + length);
  if ((bytes[0] ?? 0) & 0x80) {
    // GNU base-256: a big-endian integer after the marker bit.
    let value = BigInt((bytes[0] ?? 0) & 0x7f);
    for (const byte of bytes.subarray(1)) value = value * 256n + BigInt(byte);
    if (value > BigInt(Number.MAX_SAFE_INTEGER)) throw new ProviderRefusal("tar number overflow");
    return Number(value);
  }
  const text = field(header, start, length).trim();
  if (text === "") return 0;
  if (!/^[0-7]+$/u.test(text)) throw new ProviderRefusal("invalid tar number");
  return Number.parseInt(text, 8);
}

function checksumMatches(header: Buffer): boolean {
  let sum = 0;
  for (let index = 0; index < BLOCK; index += 1) {
    sum += index >= 148 && index < 156 ? 0x20 : (header[index] ?? 0);
  }
  return numeric(header, 148, 8) === sum;
}

/** `length key=value\n` records of a pax extended header. */
function paxRecords(body: Buffer): Map<string, string> {
  const records = new Map<string, string>();
  let cursor = 0;
  while (cursor < body.length) {
    const space = body.indexOf(0x20, cursor);
    if (space < 0) throw new ProviderRefusal("invalid pax record");
    const length = Number.parseInt(body.subarray(cursor, space).toString("ascii"), 10);
    if (
      !Number.isInteger(length) ||
      length <= space - cursor + 1 ||
      cursor + length > body.length
    ) {
      throw new ProviderRefusal("invalid pax record length");
    }
    const record = body.subarray(space + 1, cursor + length - 1).toString("utf8");
    const equals = record.indexOf("=");
    if (equals < 0) throw new ProviderRefusal("invalid pax record");
    records.set(record.slice(0, equals), record.slice(equals + 1));
    cursor += length;
  }
  return records;
}

function nulTerminated(body: Buffer): string {
  const end = body.indexOf(0);
  return body.subarray(0, end < 0 ? body.length : end).toString("utf8");
}

/** Every real member of an uncompressed tar, with pax and GNU long names applied. */
function tarMembers(tar: Buffer): TarMember[] {
  const members: TarMember[] = [];
  let global = new Map<string, string>();
  let pending = new Map<string, string>();
  let cursor = 0;
  while (cursor + BLOCK <= tar.length) {
    const header = tar.subarray(cursor, cursor + BLOCK);
    if (header.every((byte) => byte === 0)) break;
    if (!checksumMatches(header)) throw new ProviderRefusal("invalid tar header checksum");
    cursor += BLOCK;
    const rawSize = numeric(header, 124, 12);
    const rawType = String.fromCharCode(header[156] ?? 0);
    const available = tar.subarray(cursor, Math.min(tar.length, cursor + rawSize));
    const padded = Math.ceil(rawSize / BLOCK) * BLOCK;
    if (rawType === "x" || rawType === "g" || rawType === "L" || rawType === "K") {
      if (available.length !== rawSize) throw new ProviderRefusal("truncated tar extension");
      if (rawType === "x") pending = new Map([...pending, ...paxRecords(available)]);
      if (rawType === "g") global = new Map([...global, ...paxRecords(available)]);
      if (rawType === "L") pending.set("path", nulTerminated(available));
      if (rawType === "K") pending.set("linkpath", nulTerminated(available));
      cursor += padded;
      continue;
    }
    const extended = new Map([...global, ...pending]);
    pending = new Map();
    const prefix = field(header, 345, 155);
    const ustar = field(header, 257, 6).startsWith("ustar");
    const shortName = field(header, 0, 100);
    const size = extended.has("size") ? Number(extended.get("size")) : rawSize;
    if (!Number.isSafeInteger(size) || size < 0) throw new ProviderRefusal("invalid tar size");
    const body = tar.subarray(cursor, Math.min(tar.length, cursor + size));
    members.push({
      name: extended.get("path") ?? (ustar && prefix !== "" ? `${prefix}/${shortName}` : shortName),
      linkname: extended.get("linkpath") ?? field(header, 157, 100),
      // A NUL type flag is the pre-POSIX spelling of a regular file.
      type: rawType === "\0" ? "0" : rawType,
      mode: numeric(header, 100, 8),
      size,
      body: Buffer.from(body),
    });
    cursor += Math.ceil(size / BLOCK) * BLOCK;
  }
  return members;
}

const regularTypes = new Set(["0", "7", "S"]);

/** Validate the whole archive before any member is written. */
export function members(
  raw: Buffer,
  integrity: string,
  count: number,
  total: number,
): ProviderFile[] {
  const digest = createHash("sha512").update(raw).digest("base64");
  if (raw.length > MAX_BYTES || digest !== integrity) {
    throw new ProviderRefusal("provider archive integrity or size mismatch");
  }
  const entries = tarMembers(gunzipSync(raw));
  if (entries.length !== count || entries.reduce((sum, entry) => sum + entry.size, 0) !== total) {
    throw new ProviderRefusal("provider archive file count or unpacked size mismatch");
  }
  const files: ProviderFile[] = [];
  const names = new Set<string>();
  for (const entry of entries) {
    const parts = entry.name.split("/");
    const overlaps = [...names].some(
      (name) => entry.name.startsWith(`${name}/`) || name.startsWith(`${entry.name}/`),
    );
    if (
      !regularTypes.has(entry.type) ||
      entry.linkname !== "" ||
      parts.length < 2 ||
      parts[0] !== "package" ||
      parts.some((part) => part === "" || part === "." || part === "..") ||
      entry.name.includes("\\") ||
      entry.name.includes("\0") ||
      names.has(entry.name) ||
      overlaps
    ) {
      throw new ProviderRefusal("unsafe or duplicate provider archive member");
    }
    names.add(entry.name);
    if (entry.body.length !== entry.size) {
      throw new ProviderRefusal("provider member length mismatch");
    }
    files.push({ path: parts.slice(1).join("/"), body: entry.body, mode: entry.mode });
  }
  return files;
}

function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function exportTarget(exports: unknown, entry: string, condition: string): string {
  const conditions = isRecord(exports) ? exports[entry] : undefined;
  const target = isRecord(conditions) ? conditions[condition] : undefined;
  if (typeof target !== "string") {
    throw new ProviderRefusal("missing provider declaration, plugin or license");
  }
  return target;
}

export function validatePackage(files: readonly ProviderFile[]): Readonly<Record<string, unknown>> {
  const table = new Map(files.map((file) => [file.path, file.body]));
  const manifest = table.get("package.json");
  if (manifest === undefined) throw new ProviderRefusal("missing provider manifest");
  const parsed: unknown = JSON.parse(manifest.toString("utf8"));
  if (!isRecord(parsed) || parsed.name !== "vue-router" || parsed.version !== "5.1.0") {
    throw new ProviderRefusal("unexpected provider identity");
  }
  const required = [
    exportTarget(parsed.exports, ".", "types"),
    exportTarget(parsed.exports, "./experimental", "types"),
    exportTarget(parsed.exports, "./auto-routes", "types"),
    exportTarget(parsed.exports, "./auto", "types"),
    exportTarget(parsed.exports, "./volar/sfc-typed-router", "default"),
    exportTarget(parsed.exports, "./volar/sfc-typed-router", "types"),
    "./LICENSE",
  ];
  if (required.some((target) => !target.startsWith("./") || !table.has(target.slice(2)))) {
    throw new ProviderRefusal("missing provider declaration, plugin or license");
  }
  return parsed;
}

/** True for anything at the path, including a dangling symbolic link. */
function occupied(target: string): boolean {
  try {
    fs.lstatSync(target);
    return true;
  } catch {
    return false;
  }
}

/** Refuse before the network: the destination must be new and its parent real. */
export function assertNewDestination(destination: string): void {
  let parentIsDirectory = false;
  try {
    parentIsDirectory = fs.statSync(path.dirname(destination)).isDirectory();
  } catch {
    // A missing or unreadable parent is not an isolated directory.
  }
  if (occupied(destination) || !parentIsDirectory) {
    throw new ProviderRefusal("provider destination must be a new isolated directory");
  }
}

async function download(): Promise<Buffer> {
  const response = await fetch(PROVIDER_URL, {
    redirect: "error",
    signal: AbortSignal.timeout(30_000),
  });
  if (response.status !== 200 || response.url !== PROVIDER_URL || response.body === null) {
    throw new ProviderRefusal("unexpected official provider response");
  }
  // Read one byte past the limit at most, so an oversized body is refused, not buffered.
  const chunks: Buffer[] = [];
  let received = 0;
  for await (const chunk of response.body) {
    const bytes = Buffer.from(chunk);
    chunks.push(bytes.subarray(0, MAX_BYTES + 1 - received));
    received += bytes.length;
    if (received > MAX_BYTES) break;
  }
  return Buffer.concat(chunks);
}

const sha256 = (bytes: Buffer): string => createHash("sha256").update(bytes).digest("hex");

/** Two-space JSON with non-ASCII escaped, as the published receipts are written. */
function receiptText(value: unknown, indent?: number): string {
  return JSON.stringify(value, null, indent).replace(
    /[\u0080-￿]/gu,
    (character) => `\\u${character.charCodeAt(0).toString(16).padStart(4, "0")}`,
  );
}

export async function stage(destination: string): Promise<void> {
  assertNewDestination(destination);
  const raw = await download();
  const files = members(raw, PROVIDER_INTEGRITY, 70, 1_167_017);
  const manifest = validatePackage(files);
  fs.mkdirSync(destination);
  const packageRoot = path.join(destination, "package");
  const rows = [];
  for (const file of files) {
    const target = path.join(packageRoot, file.path);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, file.body, { flag: "wx" });
    fs.chmodSync(target, file.mode & 0o777);
    rows.push({
      path: file.path,
      bytes: file.body.length,
      sha256: sha256(file.body),
      mode: file.mode,
    });
  }
  fs.writeFileSync(path.join(destination, "vue-router-5.1.0.tgz"), raw);
  const receipt = {
    schema: "vize.router-page-provider",
    url: PROVIDER_URL,
    integrity: `sha512-${PROVIDER_INTEGRITY}`,
    archiveBytes: raw.length,
    archiveSha256: sha256(raw),
    publishedSource: SOURCE,
    pageAndMapSource: "feed382f2fbfe38b3892ea780f5aea3d5459986a",
    typeofReferenceSource: "071f1969be1348e797a55d0d8afb72b8068154dc",
    files: rows,
    exports: manifest.exports,
  };
  fs.writeFileSync(path.join(destination, "receipt.json"), `${receiptText(receipt, 2)}\n`);
  console.log(receiptText(receipt));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const destination = process.argv[2];
  if (destination === undefined || process.argv.length !== 3) {
    console.error("usage: provider-archive.ts NEW_ISOLATED_DESTINATION");
    process.exit(1);
  }
  await stage(path.resolve(destination));
}
