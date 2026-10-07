// Explicit current outputs for #7968; original fixture/capture bytes stay frozen.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { compareBytes } from "../../differential/compare.mjs";
import { sha256 } from "../../differential/manifest.mjs";

const authorityPath =
  "tests/_fixtures/differential/formatter-regressions/css-multi-value-7968/current-references.json";
const authoritySha256 = "8e53853840d474c0d9cd6a29eaa0ab061a8e3e46df7f1c45fd82d705e0a6220c";

export function multiValueReference(root, owner, fixture, input, historical, corpus) {
  const bytes = fs.readFileSync(path.join(root, authorityPath));
  assert.equal(sha256(bytes), authoritySha256, "reviewed multi-value authority changed");
  const authority = JSON.parse(bytes.toString());
  assert.equal(authority.schema, "vize.css-multi-value-current-references");
  assert.equal(authority.version, 1);
  assert.equal(authority.issue, 7968);
  assert.equal(authority.cases.length, 12);
  const row = authority.cases.find((row) => row.owner === owner && row.id === fixture.id);
  if (!row) return { expected: historical, cliExpected: fixture.cliExpected };
  assert.equal(sha256(corpus), row.corpusSha256, "original complete corpus changed");
  assert.equal(sha256(input), row.inputSha256, "original multi-value input changed");
  assert.equal(sha256(historical), row.historicalExpectedSha256, "original output changed");
  assert.deepEqual(fixture.options ?? {}, row.options, "configured original options changed");
  assert.equal(
    sha256(JSON.stringify(fixture.cliExpected)),
    row.historicalCliSha256,
    "original complete CLI plan changed",
  );
  const current = Buffer.from(row.currentExpected);
  assert.equal(sha256(current), row.currentExpectedSha256);
  assert(!current.equals(historical), "historical mismatch cannot become original equality");
  const cliExpected = row.currentInitialCli
    ? [...row.currentInitialCli, ...fixture.cliExpected.slice(2)]
    : fixture.cliExpected;
  return {
    expected: current,
    cliExpected,
    reference: {
      issue: 7968,
      owner,
      id: row.id,
      authority: { path: authorityPath, sha256: authoritySha256 },
      inputSha256: row.inputSha256,
      historicalExpectedSha256: row.historicalExpectedSha256,
      currentExpectedSha256: row.currentExpectedSha256,
    },
  };
}

export function multiValueComparisons(qualification, historical, output) {
  const referenceComparison = compareBytes(historical, output);
  const currentReferenceComparison = qualification.reference
    ? compareBytes(qualification.expected, output)
    : undefined;
  assert.equal(
    (currentReferenceComparison ?? referenceComparison).state,
    "equal",
    "whole current output changed",
  );
  if (qualification.reference) assert.equal(referenceComparison.state, "different");
  return {
    referenceComparison,
    ...(currentReferenceComparison ? { currentReferenceComparison } : {}),
  };
}
