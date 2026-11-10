/**
 * Invert the closed-SFC emit factory of the listener gate (gate aliases, shared
 * helpers, SDK entries and the compiled preamble) before older emitter custody.
 */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const closedEmitFactoryRecord = "tests/_fixtures/tooling/closed-emit-factory.json";
const recordSha256 = "09a50e591040a6d05767ce7aef74f7f5e1dbe549d2796a4f3535047a6207ec76";

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

/** Advance a composed shared preamble across the closed-SFC helpers. */
export function applyClosedEmitFactoryIn(previous: string): string {
  let current = previous;
  for (const hunk of load().compiledPreamble.hunks) {
    assert.equal(current.split(hunk.before).length, 2, "closed emit factory predecessor line");
    current = current.replace(hunk.before, () => hunk.after);
  }
  return current;
}

/** Inverse of {@link applyClosedEmitFactoryIn}. */
export function beforeClosedEmitFactoryIn(current: string): string {
  let previous = current;
  for (const hunk of load().compiledPreamble.hunks) {
    assert.equal(previous.split(hunk.after).length, 2, "closed emit factory current lines");
    previous = previous.replace(hunk.after, () => hunk.before);
  }
  return previous;
}
