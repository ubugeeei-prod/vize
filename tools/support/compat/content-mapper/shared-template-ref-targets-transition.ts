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

/** Inverse of the shared template-ref registry targets; mirrors payload_update/template_ref_targets.rs. */
const recordBytes = readFileSync(
  new URL("../../../../tests/_fixtures/tooling/shared-template-ref-targets.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "d1270351281979a5e0995b5969a6fe8e0d182bf7af7ed5dd554d51f1dbb9c370",
);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.sharedTemplateRefTargetsSource.v1");
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The pre-target bytes of a recorded helper asset; other images pass through. */
export function beforeSharedTemplateRefTargetsSource(file: string, current: string): string {
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

/** Advance a composed shared preamble across the template-ref target layer. */
export function applySharedTemplateRefTargetsIn(previous: string): string {
  let current = previous;
  for (const hunk of record.compiledPreamble.hunks) {
    assert.equal(current.split(hunk.before).length, 2, "template-ref target predecessor line");
    assert.ok(!current.includes("$DomElement<"), "template-ref targets are not yet declared");
    current = current.replace(hunk.before, () => hunk.after);
  }
  return current;
}

/** Inverse of {@link applySharedTemplateRefTargetsIn}. */
export function beforeSharedTemplateRefTargetsIn(current: string): string {
  let previous = current;
  for (const hunk of record.compiledPreamble.hunks) {
    assert.equal(previous.split(hunk.after).length, 2, "template-ref target current lines");
    previous = previous.replace(hunk.after, () => hunk.before);
  }
  return previous;
}
