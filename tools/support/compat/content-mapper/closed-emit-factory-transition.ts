/** Invert the closed-SFC emit factory of the listener gate before older emitter custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const closedEmitFactoryRecord = "tests/_fixtures/tooling/closed-emit-factory.json";
const recordSha256 = "80d2fc39b47e8c85920d26100b7d5d2c9c19147dd7c6fd3ab4572c1aa8200905";

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
    new URL(`../../../../${closedEmitFactoryRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable closed emit factory transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.closedEmitFactorySource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The pre-closed-factory bytes of a recorded producer's current image; older images pass through. */
export function beforeClosedEmitFactory(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file) return current;
  const now = identity(current);
  if (now.bytes !== file.after.bytes || now.sha256 !== file.after.sha256) return current;
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `closed emit factory hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `closed emit factory predecessor: ${path}`);
  return current;
}
