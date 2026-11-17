import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

type Identity = { bytes: number; sha256: string };
type Hunk = { before: string; after: string };
type FileTransition = { path: string; before: Identity; after: Identity; hunks: Hunk[] };
type Record = {
  schema: string;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: FileTransition[];
};

/** Inverse of the multi-line value callback directives in the listener inference copy. */
const recordBytes = readFileSync(
  new URL("../../../../tests/_fixtures/tooling/multiline-value-callbacks.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "07fc40f66a0d1c49e4311b91e55ee23a1954b51a339810c298443723d6a5d08e",
);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.multilineValueCallbacksSource.v1");
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The previous bytes of a recorded producer; other images pass through. */
export function beforeMultilineValueCallbacks(file: string, current: string): string {
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
