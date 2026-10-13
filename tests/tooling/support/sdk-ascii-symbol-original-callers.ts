import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type Identity = { file: string; mode: string; blob: string; bytes: number; sha256: string };
type Capsule = Identity & { rawBase64: string };
type Frame = { kind: string; source: string; tree: string; files: Identity[] };
export type AsciiCaller = { file: string; mode: string; blob: string; body: Buffer };
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
const blob = (body: Buffer) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
function load(name: string, digest: string) {
  const raw = fs.readFileSync(new URL(name, import.meta.url));
  assert.equal(sha(raw), digest);
  return JSON.parse(raw.toString("utf8"));
}
const original = load(
  "./ascii-symbol-original-callers.json",
  "0dc0852abe918f17f378a708e70d61b42f0630fe94865db96f8c7201d43ffbd3",
) as {
  version: number;
  source: string;
  tree: string;
  original: Capsule[];
  scope: string;
};
const frames = load(
  "./ascii-symbol-caller-frames.json",
  "ae9cbab52a1ba62f3e31b8e4f92f80e3c973de3510d219ee940a6b89cfc716d6",
) as {
  version: number;
  frames: Frame[];
  changedBodies: (Capsule & { kind: string })[];
};
assert.equal(frames.version, 1);
const composed = load(
  "./ascii-symbol-composed-caller-frame.json",
  "22b6a814fc44d072037fdf690761597ea68339a2a7a9e2d6537925c94ceac7a6",
) as { version: number; frame: Frame; changedBodies: (Capsule & { kind: string })[] };
assert.equal(composed.version, 1);
assert.equal(composed.frame.kind, "owned-sdk-reader");
assert.equal(composed.frame.source, "fbdaa063d15302b8f2ff8693f57336cba007d056");
assert.equal(composed.frame.tree, "160337c9ffa80e18ae3d7b29b3d22a253cf3f290");
const retained = {
  ...original,
  frames: [...frames.frames, composed.frame],
  changedBodies: [...frames.changedBodies, ...composed.changedBodies],
};
assert.equal(retained.version, 1);
assert.equal(retained.source, "5c54b1195fa8c0bf151e0a2862bf4238a3597b4e");
assert.equal(retained.original.length, 10);
assert.equal(new Set(retained.original.map(({ file }) => file)).size, 10);
const originals = new Map<string, Buffer>();
for (const entry of retained.original) {
  const bytes = Buffer.from(entry.rawBase64, "base64");
  assert.equal(bytes.toString("base64"), entry.rawBase64);
  assert.equal(entry.mode, "100644");
  assert.equal(bytes.length, entry.bytes);
  assert.equal(sha(bytes), entry.sha256);
  assert.equal(blob(bytes), entry.blob);
  originals.set(entry.file, bytes);
}
assert.equal(retained.frames.length, 4);
for (const frame of retained.frames) {
  assert.equal(frame.files.length, 10);
  assert.deepEqual(
    frame.files.map(({ file }) => file),
    [...originals.keys()],
  );
  for (const identity of frame.files) assert.equal(identity.mode, "100644");
}

const changed = new Map<string, Buffer>();
assert.equal(retained.changedBodies.length, 7);
for (const entry of retained.changedBodies) {
  const frame = retained.frames.find(({ kind }) => kind === entry.kind);
  assert.ok(frame);
  const expected = frame.files.find(({ file }) => file === entry.file);
  assert.ok(expected);
  const { rawBase64, kind, ...identity } = entry;
  assert.deepEqual(identity, expected);
  const bytes = Buffer.from(rawBase64, "base64");
  assert.equal(bytes.toString("base64"), rawBase64);
  assert.equal(bytes.length, expected.bytes);
  assert.equal(sha(bytes), expected.sha256);
  assert.equal(blob(bytes), expected.blob);
  const key = `${kind}:${entry.file}`;
  assert.equal(changed.has(key), false);
  changed.set(key, bytes);
}

/** Sealed whole source frames support real hybrid controls without historical Git objects. */
export function retainedAsciiCallerFrame(kind: string): AsciiCaller[] {
  const frame = retained.frames.find((entry) => entry.kind === kind);
  assert.ok(frame, "unknown source frame is refused");
  return frame.files.map((entry) => {
    const bytes = changed.get(`${kind}:${entry.file}`) ?? originals.get(entry.file);
    assert.ok(bytes);
    assert.equal(bytes.length, entry.bytes);
    assert.equal(sha(bytes), entry.sha256);
    assert.equal(blob(bytes), entry.blob);
    return { file: entry.file, mode: entry.mode, blob: entry.blob, body: Buffer.from(bytes) };
  });
}

/** Complete source frames only; this never grants native, workload or performance credit. */
export function qualifyAsciiOriginalCallers(entries: AsciiCaller[]) {
  assert.equal(entries.length, 10);
  assert.equal(new Set(entries.map(({ file }) => file)).size, 10);
  const observed = new Map(
    entries.map((entry) => {
      assert.deepEqual(
        Object.keys(entry).toSorted((a, b) => a.localeCompare(b)),
        ["blob", "body", "file", "mode"],
      );
      assert.equal(entry.mode, "100644");
      assert.ok(Buffer.isBuffer(entry.body));
      assert.equal(blob(entry.body), entry.blob);
      return [entry.file, { ...entry, bytes: entry.body.length, sha256: sha(entry.body) }] as const;
    }),
  );
  const frame = retained.frames.find(({ files }) =>
    files.every((identity) => {
      const actual = observed.get(identity.file);
      return (
        actual &&
        actual.mode === identity.mode &&
        actual.blob === identity.blob &&
        actual.bytes === identity.bytes &&
        actual.sha256 === identity.sha256
      );
    }),
  );
  assert.ok(
    frame,
    "one complete original, reviewed SDK, reviewed reader or owned SDK/reader source frame is required",
  );
  return {
    authority: "complete-ascii-original-caller-source-frame",
    kind: frame.kind,
    source: frame.source,
    tree: frame.tree,
    originalSource: retained.source,
    wholeCallerBodies: 10,
    runtimeCredit: false,
    performanceCredit: false,
  };
}

/** The original positive reads its whole old body only after complete current-frame authentication. */
export function originalAsciiCaller(source: string, file: string, entries: AsciiCaller[]) {
  assert.equal(source, retained.source);
  qualifyAsciiOriginalCallers(entries);
  const body = originals.get(file);
  assert.ok(body, "unknown original source body is refused");
  return Buffer.from(body);
}
