/**
 * On-disk container for the persistent pre-compile cache.
 *
 * The first format stored the whole manifest as one JSON document, which meant
 * every compiled module's `code` went to disk as a JSON-escaped string: ~87% of
 * the bytes were that one field, and a cold build paid `JSON.stringify` over all
 * of it while a warm build paid `JSON.parse` back. This container splits the
 * two concerns instead:
 *
 * ```text
 * <header JSON>\n<compressed index><compressed payload>
 * ```
 *
 * - The **header** is one line of plain JSON naming the format, the cache key,
 *   the codec, and the exact byte length of the two bodies.
 * - The **index** is a compressed JSON array of `[relativePath, sourceHash,
 *   recordLength]`. Record offsets are *not* stored: they are the running sum of
 *   the lengths, so there is no offset that can point somewhere else.
 * - The **payload** is the records back to back, each one
 *   `<meta JSON>\n<code utf8><css utf8>\n`. `code` and `css` are raw UTF-8, so
 *   the 90% of the manifest that is compiled output is never escaped, parsed, or
 *   re-quoted -- only sliced out.
 *
 * `meta` is `[codeLength, cssLength, everythingElse]`, where `everythingElse` is
 * the compiled module minus `code`/`css` **by rest destructuring**, not by a
 * hand-copied field list. A field added to `CompiledModule` later is therefore
 * carried through automatically instead of being silently dropped.
 *
 * Every structural expectation above is re-checked on read and any failure
 * returns `null`, which the caller treats as "no cache" -- a full recompile.
 * See `decodePrecompileManifest`.
 */

import path from "node:path";
import zlib from "node:zlib";

import type { CompiledModule } from "../types.ts";
import { PRECOMPILE_CACHE_FORMAT } from "./precompile-cache-key.ts";

/** File extension of the container. Not `.json`: it is a header plus two blobs. */
export const PRECOMPILE_CACHE_EXTENSION = ".vpc";

const LF = 0x0a;
const NEWLINE = Buffer.from("\n");
const EMPTY = Buffer.alloc(0);

export interface PrecompileCacheEntry {
  hash: string;
  module: CompiledModule;
}

/**
 * Whether `module` may be persisted.
 *
 * Modules assembled from `src` imports depend on sibling files that this cache
 * does not hash, so they are recompiled on every cold start instead.
 */
export function isPersistablePrecompileModule(module: CompiledModule): boolean {
  return !module.dependencies || module.dependencies.length === 0;
}

function isCompiledModule(value: unknown): value is CompiledModule {
  if (value === null || typeof value !== "object") {
    return false;
  }
  const module = value as Partial<CompiledModule>;
  if (typeof module.code !== "string" || typeof module.scopeId !== "string") {
    return false;
  }
  if (typeof module.hasScoped !== "boolean") {
    return false;
  }
  if (module.css !== undefined && typeof module.css !== "string") {
    return false;
  }
  if (module.styles !== undefined && !Array.isArray(module.styles)) {
    return false;
  }
  if (module.macroArtifacts !== undefined && !Array.isArray(module.macroArtifacts)) {
    return false;
  }
  // `src`-import modules are never written; reject them if one shows up anyway.
  return isPersistablePrecompileModule(module as CompiledModule);
}

// ---------------------------------------------------------------------------
// Codec
// ---------------------------------------------------------------------------

/**
 * `zstd` when this Node has it, `gzip` otherwise.
 *
 * The sync zstd bindings arrived in Node 22.15 / 23.8 and this package supports
 * Node >= 22, so the codec is detected rather than assumed. Both codecs carry
 * an integrity check of their own -- zstd with `checksumFlag`, gzip with its
 * trailing CRC32 -- so bit rot inside either body fails decompression instead of
 * being decoded into a plausible-looking module.
 */
const hasZstd =
  typeof zlib.zstdCompressSync === "function" && typeof zlib.zstdDecompressSync === "function";

const ZSTD_PARAMS = hasZstd ? { params: { [zlib.constants.ZSTD_c_checksumFlag]: 1 } } : undefined;

function compressBody(body: Buffer): Buffer {
  return hasZstd ? zlib.zstdCompressSync(body, ZSTD_PARAMS) : zlib.gzipSync(body, { level: 1 });
}

/** `null` for an unknown codec, or one this Node cannot read. */
function decompressBody(codec: unknown, body: Buffer): Buffer | null {
  if (codec === "zstd") {
    return hasZstd ? zlib.zstdDecompressSync(body) : null;
  }
  if (codec === "gzip") {
    return zlib.gunzipSync(body);
  }
  return null;
}

// ---------------------------------------------------------------------------
// Encode
// ---------------------------------------------------------------------------

/** `<meta JSON>\n<code><css>\n` -- the trailing LF marks the record boundary. */
function encodeRecord(module: CompiledModule): Buffer {
  const { code, css, ...rest } = module;
  const codeBytes = Buffer.from(code, "utf8");
  const cssBytes = css === undefined ? null : Buffer.from(css, "utf8");
  const meta = Buffer.from(
    JSON.stringify([codeBytes.length, cssBytes === null ? -1 : cssBytes.length, rest]),
    "utf8",
  );
  return Buffer.concat([meta, NEWLINE, codeBytes, cssBytes ?? EMPTY, NEWLINE]);
}

export interface EncodeManifestOptions {
  key: string;
  /** Vite root; paths are stored relative to it so the index stays small. */
  root: string;
  entries: ReadonlyMap<string, PrecompileCacheEntry>;
}

/** Serialize the container. Never throws for well-typed entries. */
export function encodePrecompileManifest(options: EncodeManifestOptions): Buffer {
  const { key, root, entries } = options;
  const index: [string, string, number][] = [];
  const records: Buffer[] = [];
  for (const [file, entry] of entries) {
    const record = encodeRecord(entry.module);
    index.push([path.relative(root, file), entry.hash, record.length]);
    records.push(record);
  }
  const indexBody = compressBody(Buffer.from(JSON.stringify(index), "utf8"));
  const payloadBody = compressBody(Buffer.concat(records));
  const header = Buffer.from(
    JSON.stringify({
      format: PRECOMPILE_CACHE_FORMAT,
      key,
      codec: hasZstd ? "zstd" : "gzip",
      index: indexBody.length,
      payload: payloadBody.length,
    }),
    "utf8",
  );
  return Buffer.concat([header, NEWLINE, indexBody, payloadBody]);
}

// ---------------------------------------------------------------------------
// Decode
// ---------------------------------------------------------------------------

interface ContainerHeader {
  format: unknown;
  key: unknown;
  codec: unknown;
  index: unknown;
  payload: unknown;
}

function isByteLength(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
}

function isIndexRow(value: unknown): value is [string, string, number] {
  return (
    Array.isArray(value) &&
    value.length === 3 &&
    typeof value[0] === "string" &&
    value[0].length > 0 &&
    typeof value[1] === "string" &&
    value[1].length > 0 &&
    isByteLength(value[2]) &&
    value[2] > 0
  );
}

/** Rebuild one module from its payload record, or `null` if the record is not one. */
function decodeRecord(record: Buffer): CompiledModule | null {
  if (record.at(-1) !== LF) {
    // The boundary the index described is not where a record ends.
    return null;
  }
  const metaEnd = record.indexOf(LF);
  if (metaEnd < 0 || metaEnd >= record.length - 1) {
    return null;
  }
  let meta: unknown;
  try {
    meta = JSON.parse(record.toString("utf8", 0, metaEnd));
  } catch {
    return null;
  }
  if (!Array.isArray(meta) || meta.length !== 3) {
    return null;
  }
  const [codeLength, cssLength, rest] = meta as [unknown, unknown, unknown];
  if (!isByteLength(codeLength) || typeof cssLength !== "number") {
    return null;
  }
  if (!isByteLength(cssLength) && cssLength !== -1) {
    return null;
  }
  const cssBytes = cssLength === -1 ? 0 : cssLength;
  // The lengths inside the record must agree with the length the index gave it.
  if (metaEnd + 1 + codeLength + cssBytes + 1 !== record.length) {
    return null;
  }
  if (typeof rest !== "object" || rest === null || Array.isArray(rest)) {
    return null;
  }
  // The payload bytes are the only source for `code`/`css`; a record that also
  // carries them in its meta is ambiguous about which one wins, so refuse it.
  if ("code" in rest || "css" in rest) {
    return null;
  }
  const codeStart = metaEnd + 1;
  const cssStart = codeStart + codeLength;
  const module = {
    ...(rest as Record<string, unknown>),
    code: record.toString("utf8", codeStart, cssStart),
  } as CompiledModule;
  if (cssLength !== -1) {
    module.css = record.toString("utf8", cssStart, cssStart + cssLength);
  }
  return isCompiledModule(module) ? module : null;
}

export interface DecodeManifestOptions {
  /** The key the container must claim; a mismatch rejects the whole file. */
  key: string;
  root: string;
  onReject?: (reason: string) => void;
}

/**
 * Parse the container, or return `null`.
 *
 * `null` is indistinguishable from having no cache at all, which is exactly the
 * safe outcome: recompile everything. Every gate below fails that way --
 * a missing or unparsable header, a foreign `format` or `key`, a codec this Node
 * cannot read, body lengths that do not account for the file exactly (a
 * truncated or partially written container), a body that fails its own
 * decompression checksum, an index that is not the expected shape, and record
 * lengths that do not sum to the payload. An individual entry whose record does
 * not decode into a valid `CompiledModule` is dropped on its own, as in format 1;
 * because offsets come from the index alone, one bad record cannot shift the
 * others.
 */
export function decodePrecompileManifest(
  bytes: Buffer,
  options: DecodeManifestOptions,
): Map<string, PrecompileCacheEntry> | null {
  const { key, root, onReject } = options;
  const reject = (reason: string): null => {
    onReject?.(reason);
    return null;
  };

  const headerEnd = bytes.indexOf(LF);
  if (headerEnd < 0) {
    return reject("no header");
  }
  let header: ContainerHeader;
  try {
    header = JSON.parse(bytes.toString("utf8", 0, headerEnd)) as ContainerHeader;
  } catch {
    return reject("unparsable header");
  }
  if (typeof header !== "object" || header === null || Array.isArray(header)) {
    return reject("unrecognized header");
  }
  if (header.format !== PRECOMPILE_CACHE_FORMAT || header.key !== key) {
    return reject("foreign format or key");
  }
  if (!isByteLength(header.index) || !isByteLength(header.payload)) {
    return reject("unrecognized body lengths");
  }
  const indexStart = headerEnd + 1;
  const payloadStart = indexStart + header.index;
  if (payloadStart + header.payload !== bytes.length) {
    return reject("truncated container");
  }

  let bodies: [Buffer, Buffer] | null;
  try {
    const indexText = decompressBody(header.codec, bytes.subarray(indexStart, payloadStart));
    const payloadText = decompressBody(header.codec, bytes.subarray(payloadStart));
    bodies = indexText === null || payloadText === null ? null : [indexText, payloadText];
  } catch {
    return reject("corrupt body");
  }
  if (bodies === null) {
    return reject(`unsupported codec ${JSON.stringify(header.codec)}`);
  }
  const [indexText, payload] = bodies;

  let index: unknown;
  try {
    index = JSON.parse(indexText.toString("utf8"));
  } catch {
    return reject("unparsable index");
  }
  if (!Array.isArray(index)) {
    return reject("unrecognized index");
  }

  const entries = new Map<string, PrecompileCacheEntry>();
  let offset = 0;
  for (const row of index) {
    if (!isIndexRow(row)) {
      return reject("unrecognized index row");
    }
    const [relative, hash, length] = row;
    const end = offset + length;
    if (end > payload.length) {
      return reject("index overruns payload");
    }
    const module = decodeRecord(payload.subarray(offset, end));
    offset = end;
    if (module !== null) {
      entries.set(path.resolve(root, relative), { hash, module });
    }
  }
  if (offset !== payload.length) {
    return reject("payload not fully described by the index");
  }
  return entries;
}
