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

/** Inverse of the per-module inferred emit args alias; mirrors payload_update/inferred_emit_args_alias.rs. */
const recordBytes = readFileSync(
  new URL("../../../../tests/_fixtures/tooling/inferred-emit-args-alias.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "60a3da4ec31f15c961032660268545b751cefcec44e7c524d694c231b6e4eae6",
);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.inferredEmitArgsAliasSource.v1");
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The bytes of a recorded producer before the inferred-args alias; other images pass through. */
export function beforeInferredEmitArgsAlias(file: string, current: string): string {
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
