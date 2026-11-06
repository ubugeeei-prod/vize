import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type Identity = { bytes: number; sha256: string };
type Entry = {
  path: string;
  before: Identity;
  after: Identity;
  edits: { before: string; after: string }[];
};
type Record = {
  version: number;
  kind: string;
  issue: number;
  nativeAcceptanceCredit: boolean;
  files: Entry[];
};

const recordPath =
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/constructor-emit-export-update-11982.json";
const recordSha256 = "a245df16bcb622ea50b6c3a9db839d986ec5413c3b9a63dc0a45d5e894284187";
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
function authenticate(text: string, identity: Identity) {
  assert.deepEqual({ bytes: Buffer.byteLength(text), sha256: hash(text) }, identity);
}
function record(read: (path: string) => string): Record {
  const raw = read(recordPath);
  assert.equal(hash(raw), recordSha256);
  const value = JSON.parse(raw) as Record;
  assert.equal(value.version, 1);
  assert.equal(value.kind, "exact-constructor-emit-export-source-inverse");
  assert.equal(value.issue, 11982);
  assert.equal(value.nativeAcceptanceCredit, false);
  assert.equal(value.files.length, 4);
  return value;
}
export function beforeConstructorEmitExport(
  read: (path: string) => string,
  path: string,
  current: string,
): string {
  const entry = record(read).files.find((entry) => entry.path === path);
  if (!entry) return current;
  authenticate(current, entry.after);
  current = constructorEmitExportEditsBefore(read, path, current);
  authenticate(current, entry.before);
  return current;
}
export function constructorEmitExportEditsBefore(
  read: (path: string) => string,
  path: string,
  current: string,
): string {
  const entry = record(read).files.find((entry) => entry.path === path);
  if (!entry) return current;
  const counts = entry.edits.map((edit) => current.split(edit.after).length - 1);
  if (counts.every((count) => count === 0)) return current;
  assert.ok(counts.every((count) => count === 1));
  for (const edit of entry.edits) {
    assert.ok(edit.after);
    current = current.replace(edit.after, edit.before);
  }
  return current;
}
