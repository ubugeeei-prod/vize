/** Advance helper custody across the `$PropValueOr` declaration (#12431). */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeComponentBoxes } from "./component-boxes-transition.ts";

export const propValueOrRecord = "tests/_fixtures/tooling/prop-value-or-12431.json";
export const propValueOrRecordSha256 =
  "3d59c50e8efb0b0b983302d422397af84a830a4b8ab2477886e91f0520ab9c6b";

type Identity = { bytes: number; sha256: string };
type Hunk = { before: string; after: string };
interface Transition {
  schema: string;
  issue: number;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: Array<{ path: string; before: Identity; after: Identity; hunks: Hunk[] }>;
  compiledPreamble: { path: string; before: Identity; after: Identity; hunks: Hunk[] };
}

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

let record: Transition | undefined;
function load(): Transition {
  if (record) return record;
  const raw = readFileSync(new URL(`../../../../${propValueOrRecord}`, import.meta.url), "utf8");
  assert.equal(identity(raw).sha256, propValueOrRecordSha256, "immutable PropValueOr transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.propValueOrSource.v1");
  assert.equal(parsed.issue, 12431);
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The one preamble edit: `$PropValueOr` declared right after `$PropValue`. */
export function propValueOrPreambleEdit(): Hunk {
  const { hunks } = load().compiledPreamble;
  assert.equal(hunks.length, 1);
  return hunks[0]!;
}

/** Advance a preamble image across the declaration; it must hold the predecessor line once. */
export function applyPropValueOrIn(previous: string): string {
  const edit = propValueOrPreambleEdit();
  assert.equal(previous.split(edit.before).length, 2, "PropValueOr predecessor line");
  assert.ok(!previous.includes("$PropValueOr<"), "PropValueOr is not yet declared");
  return previous.replace(edit.before, () => edit.after);
}

/** The exact pre-addition bytes of a recorded helper's current image; others pass through. */
export function beforePropValueOr(path: string, current: string): string {
  current = beforeComponentBoxes(path, current);
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `PropValueOr hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `PropValueOr predecessor: ${path}`);
  return current;
}

/** Inverse of {@link applyPropValueOrIn}. */
export function beforePropValueOrIn(current: string): string {
  const edit = propValueOrPreambleEdit();
  assert.equal(current.split(edit.after).length, 2, "PropValueOr current line");
  return current.replace(edit.after, () => edit.before);
}
