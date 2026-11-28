/** Restore the authenticated #13820 guard before unchanged helper history. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

type Identity = { bytes: number; sha256: string };
type Entry = {
  path: string;
  before: Identity;
  after: Identity;
  hunks: Array<{ before: string; after: string }>;
};
const raw = readFileSync(
  new URL(
    "../../../../tests/_fixtures/tooling/serialized-island-props-13820.json",
    import.meta.url,
  ),
);
const identity = (text: string | Buffer): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});
assert.equal(
  identity(raw).sha256,
  "7fdc992cd4e36af75c849cf4c4aa8f4928a6a57823a1cf264c32e31482d9c174",
);
const record = JSON.parse(raw.toString("utf8")) as {
  schema: string;
  issue: number;
  historicalReconstructionOnly: boolean;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: Entry[];
};
assert.equal(record.schema, "vize.serializedIslandPropsSource.v1");
assert.equal(record.issue, 13820);
assert.equal(record.historicalReconstructionOnly, true);
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

/** Older images pass to their owning layers, which authenticate their exact bytes. */
export function beforeSerializedIslandPropsSource(path: string, current: string): string {
  const entry = record.files.find((entry) => entry.path === path);
  if (!entry || identity(current).sha256 !== entry.after.sha256) return current;
  assert.deepEqual(identity(current), entry.after);
  for (const hunk of entry.hunks) {
    assert.ok(hunk.after);
    assert.equal(current.split(hunk.after).length, 2);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), entry.before);
  return current;
}
