/** Invert the complete lib type shadowing audit (#12536) before older custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const libTypeShadowingAuditRecord =
  "tests/_fixtures/tooling/lib-type-shadowing-audit-12536.json";
const recordSha256 = "f6f7cddde3c81b920bdf595e8cb5666384731c4baabfacaedf13b4a066c1cb22";

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
    new URL(`../../../../${libTypeShadowingAuditRecord}`, import.meta.url),
    "utf8",
  );
  assert.equal(identity(raw).sha256, recordSha256, "immutable lib type shadowing audit transition");
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.libTypeShadowingAuditSource.v1");
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The exact pre-audit bytes of a recorded producer; older images pass through. */
export function beforeLibTypeShadowingAudit(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  assert.equal(Buffer.byteLength(current), file.after.bytes, `lib type audit producer: ${path}`);
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `lib type audit hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `lib type audit predecessor: ${path}`);
  return current;
}
