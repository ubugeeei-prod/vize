/** Invert the lib type qualification of the listener chain and component export (#12536) before older custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeGeneratedCommentBytes } from "./generated-comment-bytes-transition.ts";

export const libTypeListenerRecord =
  "tests/_fixtures/tooling/lib-type-qualification-listeners-12536.json";
const recordSha256 = "cb1d226754a14ba1a153134c22bbe0f688a7ebf1871202569abe22c2e35c41d3";

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
    new URL(`../../../../${libTypeListenerRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(
    identity(raw).sha256,
    recordSha256,
    "immutable listener lib type qualification transition",
  );
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.libTypeListenerQualificationSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The exact pre-qualification bytes of a recorded producer's current image; older images pass through. */
export function beforeListenerLibTypeQualification(path: string, current: string): string {
  current = beforeGeneratedCommentBytes(path, current);
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(
    Buffer.byteLength(current),
    file.after.bytes,
    `listener lib type qualification producer: ${path}`,
  );
  for (const hunk of file.hunks) {
    assert.equal(
      current.split(hunk.after).length,
      2,
      `listener lib type qualification hunk: ${path}`,
    );
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(
    identity(current),
    file.before,
    `listener lib type qualification predecessor: ${path}`,
  );
  return current;
}
