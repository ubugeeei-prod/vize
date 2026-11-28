/** Invert the optional `$AuthoredKeyWitness` keys before older helper custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const optionalJsxWitnessRecord = "tests/_fixtures/tooling/optional-jsx-witness.json";
export const optionalJsxWitnessRecordSha256 =
  "0df69550cd8659a19587d40e979bd5480ae9ff8813e30a70436e9f9348964ea9";

type Identity = { bytes: number; sha256: string };
interface Transition {
  schema: string;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: Array<{
    path: string;
    before: Identity;
    after: Identity;
    hunks: Array<{ before: string; after: string }>;
  }>;
}

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

let record: Transition | undefined;
function load(): Transition {
  if (record) return record;
  const raw = readFileSync(
    new URL(`../../../../${optionalJsxWitnessRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(
    identity(raw).sha256,
    optionalJsxWitnessRecordSha256,
    "immutable optional JSX witness transition",
  );
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.optionalJsxWitnessSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/**
 * The exact pre-rewrite bytes of a recorded helper's current image. Older
 * images pass through; the layers below authenticate them.
 */
export function beforeOptionalJsxWitness(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(
    Buffer.byteLength(current),
    file.after.bytes,
    `optional JSX witness helper: ${path}`,
  );
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `optional JSX witness hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `optional JSX witness predecessor: ${path}`);
  return current;
}

/** The canonical helper line edit, as generated outputs embed it. */
export function optionalJsxWitnessHelperEdit(): { before: string; after: string } {
  const file = load().files.find(
    (entry) => entry.path === "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
  );
  assert.ok(file);
  assert.equal(file.hunks.length, 1);
  return file.hunks[0]!;
}

/** Advance a preamble image across the helper line edit; it must hold the line once. */
export function applyOptionalJsxWitnessIn(previous: string): string {
  const edit = optionalJsxWitnessHelperEdit();
  assert.equal(previous.split(edit.before).length, 2, "optional JSX witness predecessor line");
  return previous.replace(edit.before, () => edit.after);
}

/** Inverse of {@link applyOptionalJsxWitnessIn}. */
export function beforeOptionalJsxWitnessIn(current: string): string {
  const edit = optionalJsxWitnessHelperEdit();
  assert.equal(current.split(edit.after).length, 2, "optional JSX witness current line");
  return current.replace(edit.after, () => edit.before);
}

/** The recorded current identity of a helper, if this layer owns it. */
export function optionalJsxWitnessAfter(path: string): Identity | undefined {
  return load().files.find((entry) => entry.path === path)?.after;
}
