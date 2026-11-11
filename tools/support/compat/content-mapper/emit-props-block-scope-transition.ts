/** Invert the block-keyed emit-props inference declarations before older emitter custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { beforeSharedUsageGates } from "./shared-usage-gates-transition.ts";

export const emitPropsBlockScopeRecord = "tests/_fixtures/tooling/emit-props-block-scope.json";
const recordSha256 = "edc0eaff6dcf0189992bbcb5e86c058c43a92348017a7c87923719cd8eae3d5f";

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
    new URL(`../../../../${emitPropsBlockScopeRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable emit props block scope transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.emitPropsBlockScopeSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The exact pre-block-key bytes of a recorded producer's current image; older images pass through. */
export function beforeEmitPropsBlockScope(path: string, current: string): string {
  current = beforeSharedUsageGates(path, current);
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(
    Buffer.byteLength(current),
    file.after.bytes,
    `emit props block scope producer: ${path}`,
  );
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `emit props block scope hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `emit props block scope predecessor: ${path}`);
  return current;
}
