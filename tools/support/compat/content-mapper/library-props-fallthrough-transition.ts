import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

type Identity = { bytes: number; sha256: string };
type Hunk = { before: string; after: string };
type FileTransition = { path: string; before: Identity; after: Identity; hunks: Hunk[] };
type Record = {
  schema: string;
  issue: number;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: FileTransition[];
};

/** Inverse of the #13021 library fallthrough tail; mirrors payload_update/library_props_fallthrough.rs. */
const recordBytes = readFileSync(
  new URL(
    "../../../../tests/_fixtures/tooling/library-props-fallthrough-13021.json",
    import.meta.url,
  ),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "60a46c81160f70f3956f0ac032753898b2001c95baf079be64f72a242f1b8677",
);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.libraryPropsFallthroughSource.v1");
assert.equal(record.issue, 13021);
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The pre-#13021 bytes of a recorded asset; other images pass through. */
export function beforeLibraryPropsFallthrough(file: string, current: string): string {
  const transition = record.files.find((candidate) => candidate.path === file);
  if (!transition) return current;
  const now = identity(current);
  if (now.bytes !== transition.after.bytes || now.sha256 !== transition.after.sha256)
    return current;
  let previous = current;
  for (const hunk of transition.hunks) {
    assert.equal(previous.split(hunk.after).length - 1, 1);
    previous = previous.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(previous), transition.before);
  return previous;
}
