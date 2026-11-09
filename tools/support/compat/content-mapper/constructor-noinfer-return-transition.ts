/** Invert the `NoInfer` constructor return (#12431) before older emitter custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const constructorNoInferReturnRecord =
  "tests/_fixtures/tooling/constructor-noinfer-return.json";
const recordSha256 = "22b856cdbe8804b9584ef85ec020d698a60d424ec61de8f2531c5dc2159a6102";

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
    new URL(`../../../../${constructorNoInferReturnRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable constructor NoInfer transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.constructorNoInferReturnSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The pre-`NoInfer` bytes of a recorded producer's current image; older images pass through. */
export function beforeConstructorNoInferReturn(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file) return current;
  const now = identity(current);
  if (now.bytes !== file.after.bytes || now.sha256 !== file.after.sha256) return current;
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `constructor NoInfer hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `constructor NoInfer predecessor: ${path}`);
  return current;
}
