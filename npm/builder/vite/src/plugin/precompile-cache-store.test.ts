/**
 * On-disk container tests for the persistent pre-compile cache.
 *
 * The container trades one self-describing JSON document for a header, a
 * compressed index and a compressed payload, so there are more ways for the
 * bytes on disk to be wrong. Every one has to end in "no cache" -- a full
 * recompile -- never in a module decoded out of bytes that do not describe it.
 * Each case below damages the container somewhere different.
 */

import assert from "node:assert/strict";
import zlib from "node:zlib";

import {
  PRECOMPILE_CACHE_EXTENSION,
  decodePrecompileManifest,
  encodePrecompileManifest,
} from "./precompile-cache-store.ts";
import { PRECOMPILE_CACHE_FORMAT } from "./precompile-cache-key.ts";
import type { CompiledModule } from "../types.ts";

assert.equal(PRECOMPILE_CACHE_EXTENSION, ".vpc", "the container is not a JSON document");

const ROOT = "/proj";
const KEY = "0123456789abcdef0123456789abcdef";

const plain: CompiledModule = {
  code: "export default {}\n",
  scopeId: "data-v-a",
  hasScoped: false,
};
const rich: CompiledModule = {
  code: 'const a = "x";\nexport default { render() {} }\n',
  css: ".a[data-v-b] { color: red }",
  scopeId: "data-v-b",
  hasScoped: true,
  templateHash: "t1",
  styleHash: "s1",
  scriptHash: "c1",
  styles: [{ content: ".a{}", lang: "css", scoped: true, module: false, index: 0 }],
  macroArtifacts: [{ kind: "k", name: "n", source: "s", content: "c", start: 0, end: 1 }],
};
const emptyCss: CompiledModule = { ...plain, css: "", scopeId: "data-v-c" };
// A field this build does not know about: rest destructuring must carry it.
const future = { ...plain, scopeId: "data-v-d", futureField: { nested: [1, 2] } } as CompiledModule;

function build(entries: Record<string, CompiledModule>, root = ROOT, key = KEY): Buffer {
  const map = new Map(
    Object.entries(entries).map(([file, module], i) => [file, { hash: `h${i}`, module }]),
  );
  return encodePrecompileManifest({ key, root, entries: map });
}

function decode(bytes: Buffer, root = ROOT, key = KEY) {
  return decodePrecompileManifest(bytes, { key, root });
}

const files = {
  a: `${ROOT}/src/A.vue`,
  b: `${ROOT}/src/deep/B.vue`,
  c: `${ROOT}/src/C.vue`,
  d: `${ROOT}/src/D.vue`,
};
const full = build({ [files.a]: plain, [files.b]: rich, [files.c]: emptyCss, [files.d]: future });

// ---------------------------------------------------------------------------
// Round-trip. Every field shape has to come back exactly, including the ones
// this build has no name for.
// ---------------------------------------------------------------------------
{
  const entries = decode(full);
  assert.ok(entries, "a container this process just wrote must decode");
  assert.deepEqual([...entries.keys()].sort(), Object.values(files).sort());
  assert.deepEqual(entries.get(files.a)!.module, plain);
  assert.deepEqual(entries.get(files.b)!.module, rich);
  assert.deepEqual(
    entries.get(files.c)!.module,
    emptyCss,
    "an empty css block is not an absent one",
  );
  assert.equal(entries.get(files.c)!.module.css, "", '`css: ""` must not decode as undefined');
  assert.equal("css" in entries.get(files.a)!.module, false, "an absent css must stay absent");
  assert.deepEqual(entries.get(files.d)!.module, future, "unknown fields must survive the trip");
  assert.deepEqual(
    [...entries.values()].map((entry) => entry.hash),
    ["h0", "h1", "h2", "h3"],
    "source hashes must survive the trip",
  );
}

// ---------------------------------------------------------------------------
// Paths are stored relative to the root, so the index stays small -- and a
// container read under a different root resolves elsewhere and simply misses.
// ---------------------------------------------------------------------------
{
  assert.equal(
    full.includes(Buffer.from(`${ROOT}/src`)),
    false,
    "absolute paths must not reach the container",
  );
  const moved = decode(full, "/other");
  assert.ok(moved);
  assert.deepEqual([...moved.keys()].sort(), [
    "/other/src/A.vue",
    "/other/src/C.vue",
    "/other/src/D.vue",
    "/other/src/deep/B.vue",
  ]);
  assert.equal(
    moved.get(files.a),
    undefined,
    "the original path must not be found under a new root",
  );
}

// ---------------------------------------------------------------------------
// Container surgery helpers: decompress the two bodies, damage one, put it back.
// ---------------------------------------------------------------------------
interface Parts {
  header: Record<string, unknown>;
  index: Buffer;
  payload: Buffer;
}

function split(bytes: Buffer): Parts {
  const end = bytes.indexOf(0x0a);
  const header = JSON.parse(bytes.toString("utf8", 0, end)) as Record<string, unknown>;
  const indexStart = end + 1;
  const payloadStart = indexStart + (header.index as number);
  const inflate = (body: Buffer) =>
    header.codec === "zstd" ? zlib.zstdDecompressSync(body) : zlib.gunzipSync(body);
  return {
    header,
    index: inflate(bytes.subarray(indexStart, payloadStart)),
    payload: inflate(bytes.subarray(payloadStart)),
  };
}

function join(parts: Parts): Buffer {
  const deflate = (body: Buffer) =>
    parts.header.codec === "zstd"
      ? zlib.zstdCompressSync(body, { params: { [zlib.constants.ZSTD_c_checksumFlag]: 1 } })
      : zlib.gzipSync(body, { level: 1 });
  const index = deflate(parts.index);
  const payload = deflate(parts.payload);
  const header = Buffer.from(
    JSON.stringify({ ...parts.header, index: index.length, payload: payload.length }),
    "utf8",
  );
  return Buffer.concat([header, Buffer.from("\n"), index, payload]);
}

/** Replace the header without touching either body. */
function patchHeader(bytes: Buffer, patch: Record<string, unknown>): Buffer {
  const end = bytes.indexOf(0x0a);
  const header = JSON.parse(bytes.toString("utf8", 0, end)) as Record<string, unknown>;
  return Buffer.concat([
    Buffer.from(JSON.stringify({ ...header, ...patch }), "utf8"),
    bytes.subarray(end),
  ]);
}

// Sanity: the helpers reproduce a container the decoder still accepts.
assert.ok(decode(join(split(full))), "the surgery helpers must round-trip a clean container");

// ---------------------------------------------------------------------------
// Header gates. None of these may yield entries.
// ---------------------------------------------------------------------------
for (const [label, bytes] of [
  ["empty", Buffer.alloc(0)],
  ["no newline", Buffer.from('{"format":2}')],
  ["header not json", Buffer.concat([Buffer.from("<html>\n"), full])],
  ["header is an array", patchArray(full, "[]")],
  ["header is null", patchArray(full, "null")],
  ["header is a number", patchArray(full, "7")],
  ["format bumped", patchHeader(full, { format: PRECOMPILE_CACHE_FORMAT + 1 })],
  ["format missing", patchHeader(full, { format: undefined })],
  ["key mismatch", patchHeader(full, { key: "not-the-real-key" })],
  ["codec unknown", patchHeader(full, { codec: "brotli" })],
  ["codec missing", patchHeader(full, { codec: undefined })],
  ["index length not a number", patchHeader(full, { index: "12" })],
  ["index length negative", patchHeader(full, { index: -1 })],
  ["payload length fractional", patchHeader(full, { payload: 1.5 })],
  [
    "index length short by one",
    patchHeader(full, { index: (split(full).header.index as number) - 1 }),
  ],
  ["trailing junk appended", Buffer.concat([full, Buffer.from("x")])],
  ["last byte truncated", full.subarray(0, full.length - 1)],
  ["truncated in half", full.subarray(0, Math.floor(full.length / 2))],
] as const) {
  assert.equal(decode(bytes as Buffer), null, `${label} must be refused`);
}

function patchArray(bytes: Buffer, header: string): Buffer {
  return Buffer.concat([Buffer.from(header, "utf8"), bytes.subarray(bytes.indexOf(0x0a))]);
}

// ---------------------------------------------------------------------------
// Body gates. A body that fails its own integrity check, or an index that does
// not describe the payload it ships with, must take the whole container down.
// ---------------------------------------------------------------------------
{
  const end = full.indexOf(0x0a);
  const indexStart = end + 1;
  const payloadStart = indexStart + (split(full).header.index as number);

  const flip = (at: number): Buffer => {
    const copy = Buffer.from(full);
    copy[at] ^= 0xff;
    return copy;
  };
  assert.equal(decode(flip(payloadStart + 8)), null, "a corrupt payload body must be refused");
  assert.equal(decode(flip(indexStart + 8)), null, "a corrupt index body must be refused");

  const parts = split(full);
  const rows = JSON.parse(parts.index.toString("utf8")) as [string, string, number][];
  const withIndex = (index: string) => join({ ...parts, index: Buffer.from(index) });
  for (const [label, bytes] of [
    ["an index that is not an array", withIndex('{"entries":{}}')],
    ["an index row of the wrong arity", withIndex('[["src/A.vue","h0"]]')],
    ["a non-numeric record length", withIndex('[["src/A.vue","h0","12"]]')],
    ["a zero record length", withIndex('[["src/A.vue","h0",0]]')],
    ["an empty relative path", withIndex('[["","h0",12]]')],
    ["an empty source hash", withIndex('[["src/A.vue","",12]]')],
    [
      "record lengths that overrun the payload",
      withIndex(JSON.stringify(rows.map((r, i) => (i === 0 ? [r[0], r[1], r[2] + 1] : r)))),
    ],
    ["an index that leaves payload bytes undescribed", withIndex(JSON.stringify(rows.slice(0, 2)))],
    [
      "a payload shorter than the index describes",
      join({ ...parts, payload: parts.payload.subarray(0, parts.payload.length - 1) }),
    ],
  ] as const) {
    assert.equal(decode(bytes), null, `${label} must be refused`);
  }
}

// ---------------------------------------------------------------------------
// Record gates. A single bad record is dropped; because offsets come from the
// index alone it cannot shift the records after it, which is asserted here.
// ---------------------------------------------------------------------------
{
  const parts = split(full);
  const rows = JSON.parse(parts.index.toString("utf8")) as [string, string, number][];
  const firstEnd = rows[0]![2];

  const noBoundary = Buffer.from(parts.payload);
  noBoundary[firstEnd - 1] = 0x20; // the record's terminating newline
  const survived = decode(join({ ...parts, payload: noBoundary }));
  assert.ok(survived, "one unterminated record must not condemn the container");
  assert.equal(survived.has(files.a), false, "the unterminated record must be dropped");
  assert.deepEqual(survived.get(files.b)!.module, rich, "later records must still decode");
  assert.equal(survived.size, 3);

  // A record whose own declared lengths disagree with the length the index gave it.
  const metaEnd = parts.payload.indexOf(0x0a);
  const meta = JSON.parse(parts.payload.toString("utf8", 0, metaEnd)) as [number, number, object];
  const shrunk = Buffer.concat([
    Buffer.from(JSON.stringify([meta[0] - 1, meta[1], meta[2]]), "utf8"),
    parts.payload.subarray(metaEnd),
  ]);
  const relengthened = rows.map((row, i) =>
    i === 0 ? [row[0], row[1], row[2] + (shrunk.length - parts.payload.length)] : row,
  );
  const mismatched = decode(
    join({
      ...parts,
      index: Buffer.from(JSON.stringify(relengthened)),
      payload: shrunk,
    }),
  );
  assert.ok(mismatched, "a length disagreement is a dropped entry, not a dead container");
  assert.equal(mismatched.has(files.a), false, "the disagreeing record must be dropped");
  assert.deepEqual(mismatched.get(files.b)!.module, rich);

  // A record whose meta is not a record header at all.
  const notMeta = Buffer.concat([Buffer.from("nope"), parts.payload.subarray(4)]);
  const broken = decode(join({ ...parts, payload: notMeta }));
  assert.ok(broken);
  assert.equal(broken.has(files.a), false, "an unparsable record meta must be dropped");
}

// ---------------------------------------------------------------------------
// Shape gate: a record that slices cleanly but is not a `CompiledModule`.
//
// `code` and `css` come out of the payload bytes, so those two can only ever be
// strings here -- the reachable violations are in the record meta, which a
// foreign or broken writer could have produced. Each one is spliced in as the
// first record of an otherwise sound container.
// ---------------------------------------------------------------------------
{
  const parts = split(full);
  const rows = JSON.parse(parts.index.toString("utf8")) as [string, string, number][];
  const tail = parts.payload.subarray(rows[0]![2]);
  const code = Buffer.from("export default {}\n");

  for (const [label, meta] of [
    ["scopeId missing", { hasScoped: false }],
    ["scopeId not a string", { scopeId: 1, hasScoped: false }],
    ["hasScoped not a boolean", { scopeId: "x", hasScoped: "yes" }],
    ["styles not an array", { scopeId: "x", hasScoped: false, styles: {} }],
    ["macroArtifacts not an array", { scopeId: "x", hasScoped: false, macroArtifacts: "x" }],
    ["src dependencies present", { scopeId: "x", hasScoped: false, dependencies: ["/d.js"] }],
    ["code shadowed in the meta", { scopeId: "x", hasScoped: false, code: "other" }],
    ["css shadowed in the meta", { scopeId: "x", hasScoped: false, css: "other" }],
    ["meta is an array", [1, 2, 3]],
    ["meta is null", null],
  ] as const) {
    const head = Buffer.from(JSON.stringify([code.length, -1, meta]), "utf8");
    const record = Buffer.concat([head, Buffer.from("\n"), code, Buffer.from("\n")]);
    const entries = decode(
      join({
        ...parts,
        index: Buffer.from(
          JSON.stringify(rows.map((row, i) => (i === 0 ? [row[0], row[1], record.length] : row))),
        ),
        payload: Buffer.concat([record, tail]),
      }),
    );
    assert.ok(entries, `${label}: the container must still decode`);
    assert.equal(entries.has(files.a), false, `${label}: the entry must be dropped`);
    assert.deepEqual(entries.get(files.b)!.module, rich, `${label}: siblings must survive`);
  }
}

// ---------------------------------------------------------------------------
// The gzip fallback, which is what a Node without sync zstd bindings writes.
// ---------------------------------------------------------------------------
{
  const parts = split(full);
  const gzipped = join({ ...parts, header: { ...parts.header, codec: "gzip" } });
  const entries = decode(gzipped);
  assert.ok(entries, "a gzip container must be readable wherever zstd is");
  assert.deepEqual(entries.get(files.b)!.module, rich);
}

// ---------------------------------------------------------------------------
// Degenerate but legal: no entries at all.
// ---------------------------------------------------------------------------
{
  const empty = decode(build({}));
  assert.ok(empty, "an empty container is valid");
  assert.equal(empty.size, 0);
}

console.log("✅ vite-plugin-vize precompile cache store tests passed!");
