// #7826 qualifies one authored current reference. The original captured bytes
// and their strict comparison remain observable; they are never rewritten.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { compareBytes } from "./compare.mjs";
import { sha256 } from "./manifest.mjs";
import { declarationCurrentReference } from "./formatter-declaration-reference.ts";
import { huggedInterpolationCurrentReference } from "./formatter-hugged-interpolation-reference.ts";
import { rootCommentCurrentReference } from "./formatter-root-comment-reference.ts";

const ID = "capture/style-block-keeps-box-values-and-implicit-nested-selectors/sfc/0";
const AUTHORITY = "tests/_fixtures/differential/formatter-history/current-references-7826.json";
const AUTHORITY_SHA256 = "38cad2e6920e922e10e5064608f81aeb40f5a9d7244ee06481c0190c0daab02f";

export function currentFormatterReference(
  root: string,
  fixture: any,
  input: Buffer,
  historical: Buffer,
) {
  if (fixture.id !== ID) {
    const hugged = huggedInterpolationCurrentReference(root, fixture, input, historical);
    if (Object.hasOwn(hugged, "currentReference")) return hugged;
    const declaration = declarationCurrentReference(root, fixture, input, historical);
    return Object.keys(declaration).length
      ? declaration
      : rootCommentCurrentReference(root, fixture, input, historical);
  }
  const bytes = fs.readFileSync(path.join(root, AUTHORITY));
  assert.equal(sha256(bytes), AUTHORITY_SHA256, "reviewed current-reference authority changed");
  const authority = JSON.parse(bytes.toString());
  assert.equal(authority.schema, "vize.formatter-current-reference-refinements");
  assert.equal(authority.version, 1);
  assert.equal(authority.issue, 7826);
  assert.equal(authority.cases.length, 1, "unreviewed refinement scope");
  const row = authority.cases[0];
  assert.equal(row.id, fixture.id);
  assert.equal(row.api, fixture.api);
  assert.equal(fixture.kind, "Vue");
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
      issue: 7826,
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

export function formatterReferenceComparisons(fixture: any, output: Buffer) {
  return {
    referenceComparison: compareBytes(fixture.expected, output),
    ...(fixture.currentReference
      ? { currentReferenceComparison: compareBytes(fixture.currentExpected, output) }
      : {}),
  };
}

export function validateFormatterReferencePass(fixture: any, row: any, pass: any, output: Buffer) {
  assert.deepEqual(row.currentReference, fixture.currentReference, "current qualification changed");
  const comparisons = formatterReferenceComparisons(fixture, output);
  assert.deepEqual(pass.referenceComparison, comparisons.referenceComparison);
  assert.deepEqual(pass.currentReferenceComparison, comparisons.currentReferenceComparison);
  if (row.legacy.state === "matched-reference") {
    assert.equal(
      (comparisons.currentReferenceComparison ?? comparisons.referenceComparison).state,
      "equal",
      "complete current output drift",
    );
    if (fixture.currentReference) {
      assert.equal(comparisons.referenceComparison.state, "different", "historical mismatch lost");
    }
  }
}

export function currentFormatterReferenceSummary(rows: any[]) {
  if (!rows.some((row) => row.currentReference)) return {};
  return {
    currentReferenceMatches: rows.filter(
      (row) => row.currentReference && row.legacy.state === "matched-reference",
    ).length,
  };
}
