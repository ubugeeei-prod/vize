/** Invert the generated-comment byte cut before older emitter custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const generatedCommentBytesRecord = "tests/_fixtures/tooling/generated-comment-bytes.json";
const recordSha256 = "e0f9efc86d1d2ac94b84bcace95a209b4e0aa0c2d1e3f505fd25d96c010c4381";

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
    new URL(`../../../../${generatedCommentBytesRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable generated comment bytes transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.generatedCommentBytesSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/**
 * The producer bytes before generated TS dropped its debug-only `@vize-map`
 * comments and the reasons after `@ts-ignore`; older images pass through.
 */
export function beforeGeneratedCommentBytes(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file) return current;
  const now = identity(current);
  if (now.bytes !== file.after.bytes || now.sha256 !== file.after.sha256) return current;
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `generated comment bytes hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `generated comment bytes predecessor: ${path}`);
  return current;
}
