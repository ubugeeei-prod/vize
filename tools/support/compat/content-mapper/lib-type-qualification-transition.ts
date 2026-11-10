/** Invert the lib type qualification of pinned emitters (#12536) before older custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeListenerLibTypeQualification } from "./lib-type-qualification-listeners-transition.ts";

export const libTypeQualificationRecord =
  "tests/_fixtures/tooling/lib-type-qualification-12536.json";
const recordSha256 = "f518d9170414c91f1134227ec810aca7c1bb5488311e374056f1f39305d7c9db";

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
    new URL(`../../../../${libTypeQualificationRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable lib type qualification transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.libTypeQualificationSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The exact pre-qualification bytes of a recorded producer's current image; older images pass through. */
export function beforeLibTypeQualification(path: string, current: string): string {
  current = beforeListenerLibTypeQualification(path, current);
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(
    Buffer.byteLength(current),
    file.after.bytes,
    `lib type qualification producer: ${path}`,
  );
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `lib type qualification hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `lib type qualification predecessor: ${path}`);
  return current;
}
