/** Authenticate the member-inferred emit factory change before older SDK history. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type Identity = { bytes: number; sha256: string };
type Edit = { before: string; after: string };
type Entry = { path: string; before: Identity; after: Identity; edits: Edit[] };
interface Record {
  version: number;
  kind: string;
  nativeAcceptanceCredit: boolean;
  canonicalAssetsChangedByThisFix: boolean;
  files: Entry[];
  preamble: Entry;
}

export const emitMemberInferRecordPath =
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/emit-member-infer-update-20261108.json";
const recordSha256 = "4b155209430258da65b41f472c84e3583d1ae5f4e551d01640f624fe20473424";
const root = new URL("../../../", import.meta.url);
export const readRepository = (path: string) => fs.readFileSync(new URL(path, root), "utf8");
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
function authenticate(text: string, identity: Identity) {
  assert.deepEqual(
    { bytes: Buffer.byteLength(text), sha256: hash(text) },
    identity,
    "selected emit gate producer: member-inferred factory image",
  );
}
function record(read: (path: string) => string): Record {
  const raw = read(emitMemberInferRecordPath);
  assert.equal(hash(raw), recordSha256, "immutable member-inferred factory source inverse");
  const value = JSON.parse(raw) as Record;
  assert.equal(value.version, 1);
  assert.equal(value.kind, "exact-emit-member-infer-source-inverse");
  assert.equal(value.nativeAcceptanceCredit, false);
  assert.equal(value.canonicalAssetsChangedByThisFix, true);
  assert.equal(value.files.length, 4);
  return value;
}
/** Text without any successor span (older history or a restored image) passes through. */
function editsBefore(current: string, edits: Edit[]): string {
  const counts = edits.map((edit) => current.split(edit.after).length - 1);
  if (counts.every((count) => count === 0)) return current;
  assert.ok(counts.every((count) => count === 1));
  for (const edit of edits) {
    assert.ok(edit.after);
    current = current.replace(edit.after, edit.before);
  }
  return current;
}
/** Lenient chain step: files outside the record pass through unchanged. */
export function emitMemberInferEditsBefore(
  read: (path: string) => string,
  path: string,
  current: string,
): string {
  const entry = record(read).files.find((entry) => entry.path === path);
  return entry ? editsBefore(current, entry.edits) : current;
}
/** Authenticated restoration of one recorded file. */
export function beforeEmitMemberInfer(
  read: (path: string) => string,
  path: string,
  current: string,
): string {
  const entry = record(read).files.find((entry) => entry.path === path);
  assert.ok(entry, `recorded member-inferred factory file ${path}`);
  authenticate(current, entry.after);
  const previous = editsBefore(current, entry.edits);
  authenticate(previous, entry.before);
  return previous;
}
/** Source-composition prediction; compiled preamble acceptance is a native gate. */
export function emitMemberInferPreambleBefore(
  read: (path: string) => string,
  current: string,
): string {
  return editsBefore(current, record(read).preamble.edits);
}
