/** Invert authored listener props boxes before older source custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const listenerAuthoredPropsRecord =
  "tests/_fixtures/tooling/listener-authored-props-12431.json";
const recordSha256 = "163e4510f8525a29b4c4af7e3267f4e9cda9b1a71489553ec60ff70831ddd5f2";

type Identity = { bytes: number; sha256: string };
type Hunk = { before: string; after: string };
type Transition = { before: Identity; after: Identity; hunks: Hunk[] };
type Record = {
  schema: string;
  issue: number;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: Array<Transition & { path: string }>;
  compiledPreamble: Transition;
  compiledStrictPreamble: Transition;
};

const recordBytes = readFileSync(
  new URL(`../../../../${listenerAuthoredPropsRecord}`, import.meta.url),
);
assert.equal(createHash("sha256").update(recordBytes).digest("hex"), recordSha256);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.listenerAuthoredPropsSource.v1");
assert.equal(record.issue, 12431);
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

function restore(current: string, transition: Transition): string {
  assert.deepEqual(identity(current), transition.after);
  for (const hunk of transition.hunks) {
    assert.equal(current.split(hunk.after).length, 2);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), transition.before);
  return current;
}

/** Restore one recorded source to the image before raw authored listener props. */
export function beforeListenerAuthoredPropsSource(path: string, current: string): string {
  const transition = record.files.find((file) => file.path === path);
  return transition && identity(current).sha256 === transition.after.sha256
    ? restore(current, transition)
    : current;
}

/** Advance a composed helper preamble across the authored-props layer. */
export function applyListenerAuthoredPropsIn(previous: string): string {
  let current = previous;
  for (const hunk of record.compiledPreamble.hunks) {
    assert.equal(current.split(hunk.before).length, 2);
    current = current.replace(hunk.before, () => hunk.after);
  }
  return current;
}

/** Restore a compiled helper preamble before the authored-props layer. */
export function beforeListenerAuthoredPropsIn(current: string): string {
  const transition = [record.compiledPreamble, record.compiledStrictPreamble].find(
    (candidate) => identity(current).sha256 === candidate.after.sha256,
  );
  return transition ? restore(current, transition) : current;
}
