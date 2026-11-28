/** Invert the bound-props instance of unmarked constructors before older emitter custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const genericConstructorBoundInstanceRecord = "tests/_fixtures/tooling/generic-constructor-bound-instance.json";
const recordSha256 = "2911bc1eb6c1390f6a7e97f736b231da1684abac5fcbb1f8a91fb93878e548d6";

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
    new URL(`../../../../${genericConstructorBoundInstanceRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable generic constructor bound instance transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.genericConstructorBoundInstanceSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The exact pre-bound-instance bytes of a recorded producer's current image; older images pass through. */
export function beforeGenericConstructorBoundInstance(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(
    Buffer.byteLength(current),
    file.after.bytes,
    `generic constructor bound instance producer: ${path}`,
  );
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `generic constructor bound instance hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `generic constructor bound instance predecessor: ${path}`);
  return current;
}
