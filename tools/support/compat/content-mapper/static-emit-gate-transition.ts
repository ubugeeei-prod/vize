/** Invert the static-tag emit-props gate before older emitter custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const staticEmitGateRecord = "tests/_fixtures/tooling/static-emit-gate-source.json";
const recordSha256 = "3dcff6d84c6ce53b58c546e6df6db87f7b282963f0097d2d0dc0af79b01257b0";

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
  const raw = readFileSync(new URL(`../../../../${staticEmitGateRecord}`, import.meta.url), "utf8");
  assert.equal(identity(raw).sha256, recordSha256, "immutable static emit gate transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.staticEmitGateSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The exact pre-gate bytes of a recorded producer's current image; older images pass through. */
export function beforeStaticEmitGate(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(Buffer.byteLength(current), file.after.bytes, `static emit gate producer: ${path}`);
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `static emit gate hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `static emit gate predecessor: ${path}`);
  return current;
}
