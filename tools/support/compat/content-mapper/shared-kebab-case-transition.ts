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
  compiledPreamble: { before: Identity; after: Identity; hunks: Hunk[] };
  compiledStrictPreamble: { before: Identity; after: Identity; hunks: Hunk[] };
};

const recordBytes = readFileSync(
  new URL("../../../../tests/_fixtures/tooling/shared-kebab-case.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "8cca9098b24f14c0f1ac1407c55e0e8f8ea9910966d58714b5609923c7204869",
);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.sharedKebabCaseSource.v1");
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The exact predecessor of a recorded shared `KebabCase` source image. */
export function beforeSharedKebabCaseSource(file: string, current: string): string {
  const transition = record.files.find((candidate) => candidate.path === file);
  if (!transition) return current;
  const now = identity(current);
  if (now.bytes !== transition.after.bytes || now.sha256 !== transition.after.sha256)
    return current;
  let previous = current;
  for (const hunk of transition.hunks) {
    assert.equal(previous.split(hunk.after).length - 1, 1, `shared kebab-case hunk: ${file}`);
    previous = previous.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(previous), transition.before, `shared kebab-case predecessor: ${file}`);
  return previous;
}

/** Advance a composed shared preamble through the recorded kebab-case fast path. */
export function applySharedKebabCaseIn(previous: string): string {
  let current = previous;
  for (const hunk of record.compiledPreamble.hunks) {
    assert.equal(current.split(hunk.before).length, 2, "shared kebab-case predecessor line");
    current = current.replace(hunk.before, () => hunk.after);
  }
  return current;
}

/** Inverse of {@link applySharedKebabCaseIn}. */
export function beforeSharedKebabCaseIn(current: string): string {
  let previous = current;
  for (const hunk of record.compiledPreamble.hunks) {
    assert.equal(previous.split(hunk.after).length, 2, "shared kebab-case current line");
    previous = previous.replace(hunk.after, () => hunk.before);
  }
  return previous;
}
