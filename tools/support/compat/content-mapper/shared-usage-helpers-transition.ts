import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeClosedEmitFactory } from "./closed-emit-factory-transition.ts";

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

/** Inverse of the shared per-usage component helpers; mirrors payload_update/usage_helpers.rs. */
const recordBytes = readFileSync(
  new URL("../../../../tests/_fixtures/tooling/shared-usage-helpers.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "4b1c772e73ad30a9d3da814660b846df190f689e771a1e5cb371c0257bcec34e",
);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.sharedUsageHelpersSource.v1");
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The pre-helper bytes of a recorded helper asset; other images pass through. */
export function beforeSharedUsageHelpersSource(file: string, current: string): string {
  const transition = record.files.find((candidate) => candidate.path === file);
  if (!transition) return current;
  // The closed-SFC helpers sit on top of the same declaration and SDK assets.
  current = beforeClosedEmitFactory(file, current);
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

/** Advance a composed shared preamble across the usage-helper layer. */
export function applySharedUsageHelpersIn(previous: string): string {
  let current = previous;
  for (const hunk of record.compiledPreamble.hunks) {
    assert.equal(current.split(hunk.before).length, 2, "shared usage helper predecessor line");
    assert.ok(!current.includes("$UsageProps<"), "shared usage helpers are not yet declared");
    current = current.replace(hunk.before, () => hunk.after);
  }
  return current;
}

/** Inverse of {@link applySharedUsageHelpersIn}. */
export function beforeSharedUsageHelpersIn(current: string): string {
  let previous = current;
  for (const hunk of record.compiledPreamble.hunks) {
    assert.equal(previous.split(hunk.after).length, 2, "shared usage helper current lines");
    previous = previous.replace(hunk.after, () => hunk.before);
  }
  return previous;
}
