/** Certify only the reviewed JSX witness replacement before complete packet comparison. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import {
  beforeDiscriminatedJsxUnion,
  certifyDiscriminatedJsxUnionTransition,
  discriminatedJsxUnionAfterSha256,
  discriminatedJsxUnionPath,
} from "./discriminated-jsx-union-transition.ts";
import {
  beforeCanonLeftovers,
  beforeStrictExtraKeys,
  canonLeftoversSha256,
  certifyStrictExtraKeysTransition,
  strictExtraKeysAfterSha256,
  strictExtraKeysPath,
} from "./strict-extra-keys-transition.ts";

const recordUrl = new URL(
  "../../../../tests/_fixtures/differential/typechecker/tsx-single-required-exact-keys/source-transition.json",
  import.meta.url,
);
const recordSha256 = "c647fda7816e1684a864d015efde1fe8a4ada2f447e38f9930fce5e6833c6bcd";
const integratedRecordUrl = new URL(
  "../../../../tests/_fixtures/differential/typechecker/tsx-single-required-exact-keys/integrated-source-transition-11964.json",
  import.meta.url,
);
const integratedRecordSha256 = "9f4babc2f1d99fc129ceab018266503a78b60d8174374122fcce679694159518";
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const identity = (bytes: Buffer) => ({ bytes: bytes.length, sha256: digest(bytes) });

interface JsxWitnessChangeProof {
  file: string;
  beforeSha256: string;
  afterSha256: string;
  recordSha256: string;
  comparatorSha256: string;
  nativeAcceptanceCredit: false;
  successorRecordSha256?: string;
  successorRecordSha256s?: string[];
}

export function certifyJsxWitnessChange(
  file: string,
  before: Buffer,
  after: Buffer,
): JsxWitnessChangeProof {
  if (
    file === strictExtraKeysPath &&
    after.length === 6026 &&
    digest(after) === "78d6af92022c9779819704cfb63b91955101396c4929bde5e6dddc308e546b2a"
  ) {
    const predecessor = Buffer.from(beforeCanonLeftovers(file, after.toString("utf8")));
    if (!predecessor.equals(after)) {
      if (before.equals(predecessor))
        return {
          file,
          beforeSha256: digest(before),
          afterSha256: digest(after),
          recordSha256: canonLeftoversSha256,
          comparatorSha256: digest(readFileSync(new URL(import.meta.url))),
          nativeAcceptanceCredit: false,
        };
      const historical = certifyJsxWitnessChange(file, before, predecessor);
      const predecessorSuccessors =
        historical.successorRecordSha256s ??
        (historical.successorRecordSha256 ? [historical.successorRecordSha256] : []);
      return {
        ...historical,
        afterSha256: digest(after),
        successorRecordSha256: canonLeftoversSha256,
        successorRecordSha256s: [...predecessorSuccessors, canonLeftoversSha256],
        comparatorSha256: digest(readFileSync(new URL(import.meta.url))),
      };
    }
  }
  const strict = certifyStrictExtraKeysTransition(file, before, after);
  if (strict)
    return {
      ...strict,
      comparatorSha256: digest(readFileSync(new URL(import.meta.url))),
    };
  if (
    file === strictExtraKeysPath &&
    after.length === 5739 &&
    digest(after) === strictExtraKeysAfterSha256
  ) {
    const predecessor = Buffer.from(beforeStrictExtraKeys(after.toString("utf8")));
    const historical = certifyJsxWitnessChange(file, before, predecessor);
    const current = certifyStrictExtraKeysTransition(file, predecessor, after);
    assert.ok(current);
    const predecessorSuccessors =
      historical.successorRecordSha256s ??
      (historical.successorRecordSha256 ? [historical.successorRecordSha256] : []);
    return {
      ...historical,
      afterSha256: current.afterSha256,
      successorRecordSha256: current.recordSha256,
      successorRecordSha256s: [...predecessorSuccessors, current.recordSha256],
      comparatorSha256: digest(readFileSync(new URL(import.meta.url))),
    };
  }
  const successor = certifyDiscriminatedJsxUnionTransition(file, before, after);
  if (successor)
    return {
      ...successor,
      nativeAcceptanceCredit: false,
      comparatorSha256: digest(readFileSync(new URL(import.meta.url))),
    };
  if (
    file === discriminatedJsxUnionPath &&
    after.length === 5870 &&
    digest(after) === discriminatedJsxUnionAfterSha256
  ) {
    const predecessor = Buffer.from(beforeDiscriminatedJsxUnion(after.toString("utf8")));
    const historical = certifyJsxWitnessChange(file, before, predecessor);
    const current = certifyDiscriminatedJsxUnionTransition(file, predecessor, after);
    assert.ok(current);
    return {
      ...historical,
      afterSha256: current.afterSha256,
      successorRecordSha256: current.recordSha256,
      comparatorSha256: digest(readFileSync(new URL(import.meta.url))),
    };
  }
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
