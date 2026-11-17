/** Invert the generated-name spelling of producers (`__vize_*` → `$name`) before older custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

type Identity = { bytes: number; sha256: string };
type Hunk = { before: string; after: string };
type FileTransition = { path: string; before: Identity; after: Identity; hunks: Hunk[] };
type Record = {
  schema: string;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: FileTransition[];
};

/** Mirrors payload_update/generated_names.rs. */
export const generatedNamesRecord = "tests/_fixtures/tooling/generated-names-source.json";
const recordSha256 = "a855c7e52a149bbdcabbbe8430c89d58ee1c5737a5b6400dc6908f6d4f231024";

const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

let record: Record | undefined;
function load(): Record {
  if (record) return record;
  const bytes = readFileSync(new URL(`../../../../${generatedNamesRecord}`, import.meta.url));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), recordSha256);
  const parsed = JSON.parse(bytes.toString("utf8")) as Record;
  assert.equal(parsed.schema, "vize.generatedNameSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The pre-rename bytes of a recorded producer's current image; other images pass through. */
export function beforeGeneratedNames(path: string, current: string): string {
  const transition = load().files.find((candidate) => candidate.path === path);
  if (!transition) return current;
  const now = identity(current);
  if (now.bytes !== transition.after.bytes || now.sha256 !== transition.after.sha256)
    return current;
  let previous = current;
  for (const hunk of transition.hunks) {
    assert.equal(previous.split(hunk.after).length - 1, 1, `generated name hunk: ${path}`);
    previous = previous.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(previous), transition.before, `generated name predecessor: ${path}`);
  return previous;
}
