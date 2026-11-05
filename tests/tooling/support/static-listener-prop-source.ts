/** Authenticate the static marker interpretation before reading frozen gate history. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
export const staticListenerSource =
  "crates/vize_canon/src/virtual_ts/scope/component_events/generic_inference.rs";
export const staticListenerRecord =
  "tests/_fixtures/tooling/static-listener-prop-update-11815.json";
const recordSha256 = "9944123afc7253e4e154c209ea6f420d87d67db0326623546da355254c6781c6";
type Identity = { bytes: number; sha256: string };
interface Transition {
  version: number;
  kind: string;
  nativeAcceptanceCredit: boolean;
  file: {
    path: string;
    before: Identity;
    after: Identity;
    edits: { before: string; after: string }[];
  };
}
const hash = (value: string) => createHash("sha256").update(value).digest("hex");
export function beforeStaticListenerSource(
  read: (path: string) => string,
  path: string,
  current: string,
): string {
  if (path !== staticListenerSource) return current;
  const raw = read(staticListenerRecord);
  assert.equal(
    hash(raw),
    recordSha256,
    "selected emit gate producer: immutable static listener transition",
  );
  const record = JSON.parse(raw) as Transition;
  assert.equal(record.version, 1);
  assert.equal(record.kind, "exact-static-listener-prop-source-inverse");
  assert.equal(record.nativeAcceptanceCredit, false);
  assert.equal(record.file.path, path);
  const identity = (value: string) => ({ bytes: Buffer.byteLength(value), sha256: hash(value) });
  assert.deepEqual(
    identity(current),
    record.file.after,
    "selected emit gate producer: current static listener source",
  );
  for (const edit of record.file.edits) {
    assert.ok(edit.after);
    assert.equal(current.split(edit.after).length, 2);
    current = current.replace(edit.after, edit.before);
  }
  assert.deepEqual(
    identity(current),
    record.file.before,
    "selected emit gate producer: historical static listener source",
  );
  return current;
}
