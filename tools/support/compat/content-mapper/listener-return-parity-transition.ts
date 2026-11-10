/** Invert synthesized listener return parity before older source custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const listenerReturnParityRecord =
  "tests/_fixtures/tooling/listener-return-parity-12431.json";
const recordSha256 = "8a63622f0facb4c1fba727f64b0a59bf9e2ccb2c5634e12f3f538f79ca4cdced";

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
    new URL(`../../../../${listenerReturnParityRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable listener return parity transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.listenerReturnParitySource.v1");
  assert.equal(parsed.issue, 12431);
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** Restore the exact source image before synthesized listener returns became void. */
export function beforeListenerReturnParity(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.deepEqual(identity(current), file.after, `listener return parity source: ${path}`);
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `listener return parity hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `listener return parity predecessor: ${path}`);
  return current;
}
