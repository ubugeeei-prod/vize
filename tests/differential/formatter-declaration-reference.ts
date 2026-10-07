// #7866 qualifies one authored current reference. The original captured bytes
// and their strict comparison remain observable; they are never rewritten.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";
import { retainedFormatterFunction } from "./formatter-history-source-artifact.ts";

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
  validateDeclarationSnapshotAuthority(root);
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

// L090 retains its exact original law body and complete named snapshot witness.
// The current live snapshot qualifies only this punctuation change, separately
// from all original300 captures and public-output comparisons.
export function validateDeclarationSnapshotAuthority(root: string) {
  const authorityPath = "tests/_fixtures/differential/formatter-history/current-snapshot-7866.json";
  const bytes = fs.readFileSync(path.join(root, authorityPath));
  assert.equal(sha256(bytes), "c14ea44fa22125a75dd9442e73e9fa814b0159ad607f2a36af6cb111f0d4892b");
  const authority = JSON.parse(bytes.toString());
  assert.equal(authority.schema, "vize.formatter-current-snapshot-refinement");
  assert.equal(authority.version, 1);
  assert.equal(authority.issue, 7866);
  assert.equal(authority.law, "L090");
  assert.equal(authority.function, "test_format_nested_css_at_rule");
  assert.equal(authority.source.path, "crates/vize_glyph/src/style.rs");
  assert.equal(
    authority.source.originalSha256,
    "8e13635eab09e08a32282d372ef5379c22210cc4117fbccf05d164df76233a61",
  );
  const source = fs.readFileSync(path.join(root, authority.source.path));
  assert.equal(
    sha256(retainedFormatterFunction(source, authority.function)),
    authority.source.functionSha256,
  );
  const historical = fs.readFileSync(path.join(root, authority.historicalSnapshot.path));
  const current = fs.readFileSync(path.join(root, authority.currentSnapshot.path));
  assert.equal(
    sha256(historical),
    authority.historicalSnapshot.sha256,
    "original whole named snapshot changed",
  );
  assert.equal(
    sha256(current),
    authority.currentSnapshot.sha256,
    "current whole named snapshot changed",
  );
  const snapshotOutput = (snapshot: Buffer) => {
    const boundary = snapshot.indexOf("\n---\n");
    assert(boundary >= 0, "whole named snapshot metadata missing");
    return snapshot.subarray(boundary + "\n---\n".length).toString();
  };
  assert.equal(snapshotOutput(historical), authority.historicalExpected);
  assert.equal(snapshotOutput(current), authority.currentExpected);
  assert.equal(
    historical.toString().replace("    color:red\n", "    color: red;\n"),
    current.toString(),
    "only the recognized declaration punctuation may change",
  );
  assert(!current.equals(historical), "named current snapshot cannot become historical equality");
}
