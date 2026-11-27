/** Invert the authored kebab listener key of generic library exports before older emitter custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const libraryKebabListenersRecord = "tests/_fixtures/tooling/library-kebab-listeners.json";
const recordSha256 = "f496542acb367cd3649ef517007fb355fa2e2039ae1d65c971796c148f5ff41c";

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
    new URL(`../../../../${libraryKebabListenersRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable library kebab listeners transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.libraryKebabListenersSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The exact pre-raw-key bytes of a recorded producer's current image; older images pass through. */
export function beforeLibraryKebabListeners(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(
    Buffer.byteLength(current),
    file.after.bytes,
    `library kebab listeners producer: ${path}`,
  );
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `library kebab listeners hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `library kebab listeners predecessor: ${path}`);
  return current;
}
