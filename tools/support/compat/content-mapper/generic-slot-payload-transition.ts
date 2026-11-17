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

/** Inverse of the #13239 generic-host slot payload; mirrors payload_update/generic_slot_payload.rs. */
const recordBytes = readFileSync(
  new URL("../../../../tests/_fixtures/tooling/generic-slot-payload-13239.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "757bead9659fb95f54499946baf85ff4a129b7cab7ceb41dda195c734d30f1da",
);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.genericSlotPayloadSource.v1");
assert.equal(record.issue, 13239);
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The pre-#13239 bytes of a recorded helper; other images pass through. */
export function beforeGenericSlotPayload(file: string, current: string): string {
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

/** The shared payload declarations, as a compiled preamble holds them. */
export const previousGenericSlotPayload = record.files[0]!.hunks[0]!.before.trimEnd();
export const currentGenericSlotPayload = record.files[0]!.hunks[0]!.after.trimEnd();

/** Whether a canonical helper asset changed by exactly the recorded #13239 edit. */
export function certifyGenericSlotPayloadAsset(
  file: string,
  before: Buffer,
  after: Buffer,
): boolean {
  const transition = record.files.find((candidate) => candidate.path === file);
  if (!transition) return false;
  const previous = before.toString("utf8"),
    current = after.toString("utf8");
  if (
    identity(previous).sha256 !== transition.before.sha256 ||
    identity(current).sha256 !== transition.after.sha256
  )
    return false;
  assert.equal(beforeGenericSlotPayload(file, current), previous, `#13239 asset: ${file}`);
  return true;
}

/** Advance a preamble image across #13239; it must hold the predecessor once. */
export function applyGenericSlotPayloadIn(previous: string): string {
  assert.equal(previous.split(previousGenericSlotPayload).length, 2, "slot payload predecessor");
  assert.equal(previous.includes(currentGenericSlotPayload), false);
  return previous.replace(previousGenericSlotPayload, () => currentGenericSlotPayload);
}

/** Inverse of {@link applyGenericSlotPayloadIn}. */
export function beforeGenericSlotPayloadIn(current: string): string {
  assert.equal(current.split(currentGenericSlotPayload).length, 2, "slot payload current lines");
  return current.replace(currentGenericSlotPayload, () => previousGenericSlotPayload);
}
