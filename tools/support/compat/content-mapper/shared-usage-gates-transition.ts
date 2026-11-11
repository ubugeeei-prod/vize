/**
 * Invert the fallthrough source gate, the key-set `$AttrValue` and the shared
 * emit gate (producers, helper assets, SDK entries and the compiled preamble)
 * before older emitter custody.
 */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const sharedUsageGatesRecord = "tests/_fixtures/tooling/shared-usage-gates.json";
const recordSha256 = "06099f9bf5eee765b161d63ae32e8d59f8cc37e20a70dfbb3625d4e2f629578d";

type Identity = { bytes: number; sha256: string };
type Entry = {
  path: string;
  before: Identity;
  after: Identity;
  hunks: Array<{ before: string; after: string }>;
};
interface Transition {
  schema: string;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: Entry[];
  compiledPreamble: Entry;
  compiledStrictPreamble: Entry;
}

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

let record: Transition | undefined;
function load(): Transition {
  if (record) return record;
  const raw = readFileSync(
    new URL(`../../../../${sharedUsageGatesRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable shared usage gates transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.sharedUsageGatesSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The pre-gate bytes of a recorded producer's current image; older images pass through. */
export function beforeSharedUsageGates(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file) return current;
  const now = identity(current);
  if (now.bytes !== file.after.bytes || now.sha256 !== file.after.sha256) return current;
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `shared usage gates hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `shared usage gates predecessor: ${path}`);
  return current;
}

/** Advance a composed shared preamble across the fallthrough source gate, the key-set attr value and the shared emit gate. */
export function applySharedUsageGatesIn(previous: string): string {
  let current = previous;
  for (const hunk of load().compiledPreamble.hunks) {
    assert.equal(current.split(hunk.before).length, 2, "shared usage gates predecessor line");
    current = current.replace(hunk.before, () => hunk.after);
  }
  return current;
}

/** Inverse of {@link applySharedUsageGatesIn}. */
export function beforeSharedUsageGatesIn(current: string): string {
  let previous = current;
  for (const hunk of load().compiledPreamble.hunks) {
    assert.equal(previous.split(hunk.after).length, 2, "shared usage gates current lines");
    previous = previous.replace(hunk.after, () => hunk.before);
  }
  return previous;
}
