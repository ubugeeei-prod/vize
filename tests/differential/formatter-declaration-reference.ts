// #7866 qualifies one authored current reference. The original captured bytes
// and their strict comparison remain observable; they are never rewritten.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";

const ID = "capture/style-block-keeps-box-values-and-implicit-nested-selectors/style/1";
const AUTHORITY = "tests/_fixtures/differential/formatter-history/current-references-7866.json";
const AUTHORITY_SHA256 = "62f13a706244764aa24ebf1edb1b2f46e087f1dbc5f305c19951ab57a8a30c22";

export function declarationCurrentReference(
  root: string,
  fixture: any,
  input: Buffer,
  historical: Buffer,
) {
  if (fixture.id !== ID) return {};
  const bytes = fs.readFileSync(path.join(root, AUTHORITY));
  assert.equal(sha256(bytes), AUTHORITY_SHA256, "reviewed current-reference authority changed");
  const authority = JSON.parse(bytes.toString());
  assert.equal(authority.schema, "vize.formatter-current-reference-refinements");
  assert.equal(authority.version, 1);
  assert.equal(authority.issue, 7866);
  assert.equal(authority.cases.length, 1, "unreviewed refinement scope");
  const row = authority.cases[0];
  assert.equal(row.id, fixture.id);
  assert.equal(row.api, fixture.api);
  assert.equal(fixture.kind, "CSS");
  assert.deepEqual(fixture.options, {
    base: "FormatOptions::default()",
    userOverrides: {},
    internalOverrides: {},
  });
  assert.deepEqual(fixture.witness, row.witness, "original current-owner witness changed");
  assert.equal(fixture.profile, "default");
  assert.equal(fixture.outcome, undefined);
  assert.equal(row.inputSha256, sha256(input), "original refinement input changed");
  assert.equal(row.historicalExpectedSha256, sha256(historical), "historical output changed");
  const current = Buffer.from(row.currentExpected);
  assert.equal(row.currentExpectedSha256, sha256(current), "authored current output changed");
  assert(!current.equals(historical), "historical mismatch cannot be called a byte match");
  return {
    currentExpected: current,
    currentReference: {
      issue: 7866,
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
