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

/** Inverse of the #13179 `undefined` withDefaults default; mirrors payload_update/with_defaults_undefined.rs. */
const recordBytes = readFileSync(
  new URL(
    "../../../../tests/_fixtures/tooling/with-defaults-undefined-13179.json",
    import.meta.url,
  ),
);
assert.equal(
  createHash("sha256").update(recordBytes).digest("hex"),
  "d42e8c0f6f8c3f9a41701ba5c23870d593c1fb9acee609e9024b4219794f94a8",
);
const record = JSON.parse(recordBytes.toString("utf8")) as Record;
assert.equal(record.schema, "vize.withDefaultsUndefinedSource.v1");
assert.equal(record.issue, 13179);
assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

/** The pre-#13179 bytes of a recorded helper; other images pass through. */
export function beforeWithDefaultsUndefined(file: string, current: string): string {
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

/** The one edited declaration, as packet relocation certifies it. */
export const previousDefaultNativeType = record.files[0]!.hunks[0]!.before.trimEnd();
export const currentDefaultNativeType = record.files[0]!.hunks[0]!.after.trimEnd();

/** Whether a canonical helper asset changed by exactly the recorded #13179 edit. */
export function certifyWithDefaultsUndefinedAsset(
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
  assert.equal(beforeWithDefaultsUndefined(file, current), previous, `#13179 asset: ${file}`);
  return true;
}

/** Advance a preamble image across #13179; it must hold the predecessor line once. */
export function applyWithDefaultsUndefinedIn(previous: string): string {
  assert.equal(previous.split(previousDefaultNativeType).length, 2, "default native predecessor");
  assert.equal(previous.includes(currentDefaultNativeType), false);
  return previous.replace(previousDefaultNativeType, () => currentDefaultNativeType);
}

/** Inverse of {@link applyWithDefaultsUndefinedIn}. */
export function beforeWithDefaultsUndefinedIn(current: string): string {
  assert.equal(current.split(currentDefaultNativeType).length, 2, "default native current line");
  return current.replace(currentDefaultNativeType, () => previousDefaultNativeType);
}
