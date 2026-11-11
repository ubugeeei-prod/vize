/** Invert the guarded emit-props inference arrow before older emitter custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeEmitPropsBlockScope } from "./emit-props-block-scope-transition.ts";
import { beforeSharedUsageGates } from "./shared-usage-gates-transition.ts";

export const guardedInferenceArrowRecord = "tests/_fixtures/tooling/guarded-inference-arrow.json";
const recordSha256 = "8d296ee5db5d0d4ad0afda82f788593343e19bb663ab0a90af40dc4169425463";

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
    new URL(`../../../../${guardedInferenceArrowRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable guarded inference arrow transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.guardedInferenceArrowSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The exact pre-arrow bytes of a recorded producer's current image; older images pass through. */
export function beforeGuardedInferenceArrow(path: string, current: string): string {
  current = beforeSharedUsageGates(path, current);
  current = beforeEmitPropsBlockScope(path, current);
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(
    Buffer.byteLength(current),
    file.after.bytes,
    `guarded inference arrow producer: ${path}`,
  );
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `guarded inference arrow hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `guarded inference arrow predecessor: ${path}`);
  return current;
}
