/** Invert the owned-key hoist in `$UnownedComponentAttrs` (#11654) before older helper custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const unownedOwnedKeysRecord = "tests/_fixtures/tooling/unowned-owned-keys-11654.json";
export const unownedOwnedKeysRecordSha256 =
  "e33fe95745ca5a1ac4cdf0d3022d409eedc729ef77ee63b995de14f1688fa465";

type Identity = { bytes: number; sha256: string };
interface Transition {
  schema: string;
  issue: number;
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
    new URL(`../../../../${unownedOwnedKeysRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(
    identity(raw).sha256,
    unownedOwnedKeysRecordSha256,
    "immutable unowned owned-key transition",
  );
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.unownedOwnedKeysSource.v1");
  assert.equal(parsed.issue, 11654);
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/**
 * The exact pre-rewrite bytes of a recorded helper's current image. Older
 * images pass through; the layers below authenticate them.
 */
export function beforeUnownedOwnedKeys(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(Buffer.byteLength(current), file.after.bytes, `unowned owned-key helper: ${path}`);
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `unowned owned-key hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `unowned owned-key predecessor: ${path}`);
  return current;
}

/** The canonical helper line edit, as generated outputs embed it. */
export function unownedOwnedKeysHelperEdit(): { before: string; after: string } {
  const file = load().files.find(
    (entry) => entry.path === "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
  );
  assert.ok(file);
  assert.equal(file.hunks.length, 1);
  return file.hunks[0]!;
}

/** Advance a preamble image across the helper line edit; it must hold the line once. */
export function applyUnownedOwnedKeysIn(previous: string): string {
  const edit = unownedOwnedKeysHelperEdit();
  assert.equal(previous.split(edit.before).length, 2, "unowned owned-key predecessor line");
  return previous.replace(edit.before, () => edit.after);
}

/** Inverse of {@link applyUnownedOwnedKeysIn}. */
export function beforeUnownedOwnedKeysIn(current: string): string {
  const edit = unownedOwnedKeysHelperEdit();
  assert.equal(current.split(edit.after).length, 2, "unowned owned-key current line");
  return current.replace(edit.after, () => edit.before);
}

/** The recorded current identity of a helper, if this layer owns it. */
export function unownedOwnedKeysAfter(path: string): Identity | undefined {
  return load().files.find((entry) => entry.path === path)?.after;
}
