// #7877 changes only two complete root-comment attachment references.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";

const IDS = [
  "capture/keeps-comments-between-and-after-sfc-blocks/sfc/0",
  "capture/comments-follow-their-block-when-sfc-blocks-are-sorted/sfc/0",
];
const AUTHORITY = "tests/_fixtures/differential/formatter-history/current-references-7877.json";
const AUTHORITY_SHA256 = "2b69ec2d44f88a5ca0433833ef01a6b1b6860d44e7dfee74231a4932267e7af8";

export function rootCommentCurrentReference(
  root: string,
  fixture: any,
  input: Buffer,
  historical: Buffer,
) {
  if (!IDS.includes(fixture.id)) return {};
  const bytes = fs.readFileSync(path.join(root, AUTHORITY));
  assert.equal(sha256(bytes), AUTHORITY_SHA256, "reviewed root-comment authority changed");
  const authority = JSON.parse(bytes.toString());
  assert.equal(authority.schema, "vize.formatter-current-reference-refinements");
  assert.equal(authority.version, 1);
  assert.equal(authority.issue, 7877);
  assert.deepEqual(
    authority.cases.map((row: any) => row.id),
    IDS,
  );
  const row = authority.cases[IDS.indexOf(fixture.id)];
  assert.equal(fixture.api, "format_sfc");
  assert.equal(row.api, fixture.api);
  assert.equal(fixture.kind, "Vue");
  assert.equal(fixture.profile, "default");
  assert.equal(fixture.outcome, undefined);
  assert.deepEqual(fixture.options, {
    base: "FormatOptions::default()",
    userOverrides: {},
    internalOverrides: {},
  });
  assert.deepEqual(fixture.witness, row.witness, "original root-comment witness changed");
  assert.equal(row.inputSha256, sha256(input), "original root-comment input changed");
  assert.equal(row.historicalExpectedSha256, sha256(historical), "historical output changed");
  const current = Buffer.from(row.currentExpected);
  assert.equal(row.currentExpectedSha256, sha256(current), "authored current output changed");
  assert(!current.equals(historical), "historical mismatch must remain observable");
  return {
    currentExpected: current,
    currentReference: {
      issue: 7877,
      witness: row.witness,
      authority: { path: AUTHORITY, sha256: AUTHORITY_SHA256 },
      inputSha256: row.inputSha256,
      historicalExpectedSha256: row.historicalExpectedSha256,
      currentExpectedSha256: row.currentExpectedSha256,
      provenance: row.provenance,
      justification: row.justification,
    },
  };
}

// The old witness still validates its original snapshot, while the live Rust
// test must admit only the corrected current bytes for these exact two names.
export function validateRootCommentSnapshotTransition(
  root: string,
  name: string,
  current: Buffer,
  originalHash: string,
) {
  if (
    ![
      "keeps_comments_between_and_after_sfc_blocks",
      "comments_follow_their_block_when_sfc_blocks_are_sorted",
    ].includes(name)
  )
    return false;
  const bytes = fs.readFileSync(path.join(root, AUTHORITY));
  assert.equal(sha256(bytes), AUTHORITY_SHA256, "reviewed root-comment authority changed");
  const authority = JSON.parse(bytes.toString());
  const row = authority.cases.find((entry: any) => entry.witness.function === name);
  assert(row && IDS.includes(row.id), "unregistered root-comment snapshot");
  assert.equal(row.historicalSnapshot.sha256, originalHash, "original snapshot pin changed");
  const historical = fs.readFileSync(path.join(root, row.historicalSnapshot.path));
  assert.equal(sha256(historical), originalHash, "historical snapshot bytes changed");
  assert.equal(sha256(current), row.currentSnapshot.sha256, "current snapshot changed");
  const boundary = current.indexOf("\n---\n");
  assert(boundary >= 0, "current snapshot metadata is missing");
  assert.equal(current.subarray(boundary + "\n---\n".length).toString(), row.currentExpected);
  return true;
}
