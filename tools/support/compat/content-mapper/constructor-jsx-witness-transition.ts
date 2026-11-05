/** Certify only the reviewed JSX witness replacement before complete packet comparison. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const recordUrl = new URL(
  "../../../../tests/_fixtures/differential/typechecker/tsx-single-required-exact-keys/source-transition.json",
  import.meta.url,
);
const recordSha256 = "c647fda7816e1684a864d015efde1fe8a4ada2f447e38f9930fce5e6833c6bcd";
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

export function certifyJsxWitnessChange(file: string, before: Buffer, after: Buffer) {
  const bytes = readFileSync(recordUrl);
  assert.equal(digest(bytes), recordSha256);
  const record = JSON.parse(bytes.toString());
  assert.equal(record.schema, "vize.tsx-exact-keys-source-transition.v1");
  assert.equal(record.nativeAcceptanceCredit, false);
  const owned = record.files.find((entry: { path: string }) => entry.path === file);
  assert.ok(owned, `canonical helper body changed: ${file}`);
  assert.equal(before.length, owned.beforeBytes, "canonical helper body changed");
  assert.equal(after.length, owned.afterBytes, "canonical helper body changed");
  assert.equal(digest(before), owned.beforeSha256, "canonical helper body changed");
  assert.equal(digest(after), owned.afterSha256, "canonical helper body changed");
  let previous = after.toString();
  for (const edit of owned.edits as { before: string; after: string }[]) {
    assert.equal(previous.split(edit.after).length, 2);
    assert.equal(previous.split(edit.before).length, 1);
    previous = previous.replace(edit.after, edit.before);
  }
  assert.deepEqual(Buffer.from(previous), before, "only the exact JSX witness may change");
  return {
    file,
    beforeSha256: owned.beforeSha256 as string,
    afterSha256: owned.afterSha256 as string,
    recordSha256,
    comparatorSha256: digest(readFileSync(new URL(import.meta.url))),
    nativeAcceptanceCredit: false,
  };
}
