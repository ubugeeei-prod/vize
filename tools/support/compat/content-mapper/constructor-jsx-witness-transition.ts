/** Certify only the reviewed JSX witness replacement before complete packet comparison. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const recordUrl = new URL(
  "../../../../tests/_fixtures/differential/typechecker/tsx-single-required-exact-keys/source-transition.json",
  import.meta.url,
);
const recordSha256 = "c647fda7816e1684a864d015efde1fe8a4ada2f447e38f9930fce5e6833c6bcd";
const integratedRecordUrl = new URL(
  "../../../../tests/_fixtures/differential/typechecker/tsx-single-required-exact-keys/integrated-source-transition-11964.json",
  import.meta.url,
);
const integratedRecordSha256 = "b5a575101d75212826bef5930fb83f4bc5a2895451d09d9a8defe123fe72c6e2";
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const identity = (bytes: Buffer) => ({ bytes: bytes.length, sha256: digest(bytes) });

export function certifyJsxWitnessChange(file: string, before: Buffer, after: Buffer) {
  const bytes = readFileSync(recordUrl);
  assert.equal(digest(bytes), recordSha256);
  const record = JSON.parse(bytes.toString());
  assert.equal(record.schema, "vize.tsx-exact-keys-source-transition.v1");
  assert.equal(record.nativeAcceptanceCredit, false);
  let owned = record.files.find((entry: { path: string }) => entry.path === file);
  assert.ok(owned, `canonical helper body changed: ${file}`);
  let selectedRecordSha256 = recordSha256;
  if (
    before.length !== owned.beforeBytes ||
    after.length !== owned.afterBytes ||
    digest(before) !== owned.beforeSha256 ||
    digest(after) !== owned.afterSha256
  ) {
    const integratedBytes = readFileSync(integratedRecordUrl);
    assert.equal(digest(integratedBytes), integratedRecordSha256);
    const integrated = JSON.parse(integratedBytes.toString());
    assert.equal(integrated.schema, "vize.tsx-exact-keys-integrated-source-transition.v1");
    assert.equal(integrated.baseSource, "e967ca6c76de2b12018e141f34fdf754a256ee37");
    assert.deepEqual(integrated.sourceHeads, [
      { pullRequest: 11712, head: "727db3b16a8eb5151d326c73725d275498f9d54d" },
      { pullRequest: 11951, head: "f9746958867856e10474a16654fc8a4b34e87645" },
    ]);
    assert.equal(integrated.observedIntegratedHead, "6fdcf4866b37a5ae0452bcb32cdd77315cbf701d");
    assert.equal(integrated.nativeAcceptanceCredit, false);
    assert.deepEqual(integrated.originalTransition, {
      path: "tests/_fixtures/differential/typechecker/tsx-single-required-exact-keys/source-transition.json",
      sha256: recordSha256,
    });
    owned = {
      path: integrated.file.path,
      beforeBytes: integrated.file.before.bytes,
      beforeSha256: integrated.file.before.sha256,
      afterBytes: integrated.file.after.bytes,
      afterSha256: integrated.file.after.sha256,
      edits: integrated.file.edits,
    };
    assert.equal(owned.path, file, `canonical helper body changed: ${file}`);
    selectedRecordSha256 = integratedRecordSha256;
  }
  assert.deepEqual(identity(before), {
    bytes: owned.beforeBytes,
    sha256: owned.beforeSha256,
  });
  assert.deepEqual(identity(after), {
    bytes: owned.afterBytes,
    sha256: owned.afterSha256,
  });
  let previous = after.toString();
  for (const edit of [...owned.edits].reverse() as { before: string; after: string }[]) {
    assert.equal(previous.split(edit.after).length, 2);
    assert.equal(previous.split(edit.before).length, 1);
    previous = previous.replace(edit.after, edit.before);
  }
  assert.deepEqual(Buffer.from(previous), before, "only the exact JSX witness may change");
  return {
    file,
    beforeSha256: owned.beforeSha256 as string,
    afterSha256: owned.afterSha256 as string,
    recordSha256: selectedRecordSha256,
    comparatorSha256: digest(readFileSync(new URL(import.meta.url))),
    nativeAcceptanceCredit: false,
  };
}
