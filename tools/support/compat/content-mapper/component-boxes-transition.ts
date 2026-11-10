/** Invert the shared component boxes and emit gate before older emitter and SDK custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const componentBoxesRecord = "tests/_fixtures/tooling/component-boxes-source.json";
const recordSha256 = "20a3ed1946a4dbc5b08a901853da466d835418e2066b51be4173388290d4c24d";

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
  const raw = readFileSync(new URL(`../../../../${componentBoxesRecord}`, import.meta.url), "utf8");
  assert.equal(identity(raw).sha256, recordSha256, "immutable component boxes transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.componentBoxesSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The exact pre-box bytes of a recorded source's current image; older images pass through. */
export function beforeComponentBoxes(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(Buffer.byteLength(current), file.after.bytes, `component boxes source: ${path}`);
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `component boxes hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `component boxes predecessor: ${path}`);
  return current;
}

/** `read` with every recorded source restored to its pre-box bytes. */
export function readBeforeComponentBoxes(read: (path: string) => string): (path: string) => string {
  return (path) => beforeComponentBoxes(path, read(path));
}

const asset = "crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt";

/** A compiled preamble with the asset's box and gate lines added, as it embeds them. */
export function applyComponentBoxesIn(previous: string): string {
  const file = load().files.find((entry) => entry.path === asset);
  assert.ok(file, "component boxes asset");
  for (const hunk of file.hunks) {
    assert.equal(previous.split(hunk.before).length, 2, "component boxes predecessor lines");
    previous = previous.replace(hunk.before, () => hunk.after);
  }
  return previous;
}
