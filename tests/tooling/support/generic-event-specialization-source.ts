import assert from "node:assert/strict";
import { createHash } from "node:crypto";

type Identity = { bytes: number; sha256: string };
type Transition = {
  before: Identity;
  after: Identity;
  edits: { before: string; after: string }[];
};
interface Record {
  version: number;
  kind: string;
  nativeAcceptanceCredit: boolean;
  files: (Transition & { path: string })[];
  preamble: Transition;
}
export const genericEventRecordPath =
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/generic-event-specialization-update-11815.json";
const recordSha256 = "d9eee3c8eb982eba5e2144cca457ecf0b15e1edbb5948b019dd404e6c10ab845";
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
function authenticate(text: string, identity: Identity) {
  assert.deepEqual(
    { bytes: Buffer.byteLength(text), sha256: hash(text) },
    identity,
    "selected emit gate producer: generic specialization image",
  );
}
function inverse(current: string, transition: Transition): string {
  authenticate(current, transition.after);
  for (const edit of transition.edits) {
    assert.ok(edit.after);
    assert.equal(current.split(edit.after).length, 2);
    current = current.replace(edit.after, edit.before);
  }
  authenticate(current, transition.before);
  return current;
}
function record(read: (path: string) => string): Record {
  const raw = read(genericEventRecordPath);
  assert.equal(hash(raw), recordSha256, "immutable generic specialization source inverse");
  const value = JSON.parse(raw) as Record;
  assert.equal(value.version, 1);
  assert.equal(value.kind, "exact-generic-event-specialization-source-inverse");
  assert.equal(value.nativeAcceptanceCredit, false);
  assert.equal(value.files.length, 5);
  return value;
}
/** Restore only the authenticated successor; historical inputs remain unchanged. */
export function beforeGenericEventSpecialization(
  read: (path: string) => string,
  path: string,
  current: string,
): string {
  const entry = record(read).files.find((entry) => entry.path === path);
  return entry ? inverse(current, entry) : current;
}
/** Source-composition prediction; compiled preamble acceptance is a native gate. */
export function beforeGenericEventPreamble(
  read: (path: string) => string,
  current: string,
): string {
  return inverse(current, record(read).preamble);
}
