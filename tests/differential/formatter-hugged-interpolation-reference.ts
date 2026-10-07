// #7969 qualifies exactly two whole current layouts; historical bytes stay strict.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";

const IDS = ["prepared/sfc-layout-wrapped-interpolation", "prepared/template-wrapped"];
const AUTHORITY = "tests/_fixtures/differential/formatter-history/current-references-7969.json";
const HASH = "e7804e3a2223cda547ead0271917799f833981eb5b30fac9bfa3b3e6503d3d88";

function authority(root: string) {
  const bytes = fs.readFileSync(path.join(root, AUTHORITY));
  assert.equal(sha256(bytes), HASH, "reviewed interpolation authority changed");
  const result = JSON.parse(bytes.toString());
  assert.equal(result.schema, "vize.formatter-current-reference-refinements");
  assert.equal(result.version, 1);
  assert.equal(result.issue, 7969);
  assert.deepEqual(
    result.cases.map((row: any) => row.id),
    IDS,
  );
  return result;
}

export function huggedInterpolationCurrentReference(
  root: string,
  fixture: any,
  input: Buffer,
  historical: Buffer,
) {
  if (!IDS.includes(fixture.id)) return {};
  const row = authority(root).cases[IDS.indexOf(fixture.id)];
  assert.equal(row.api, fixture.api);
  assert.equal(row.kind, fixture.kind);
  assert.equal(fixture.profile, "default");
  assert.equal(fixture.outcome, undefined);
  assert.deepEqual(fixture.options, row.options, "original complete options changed");
  assert.deepEqual(fixture.witness, row.witness, "original complete witness changed");
  assert.equal(sha256(input), row.inputSha256, "original whole input changed");
  assert.equal(sha256(historical), row.historicalExpectedSha256, "historical bytes changed");
  const current = Buffer.from(row.currentExpected);
  assert.equal(sha256(current), row.currentExpectedSha256);
  assert(!current.equals(historical), "a historical mismatch cannot become a byte match");
  return {
    currentExpected: current,
    currentReference: {
      issue: 7969,
      authority: { path: AUTHORITY, sha256: HASH },
      witness: row.witness,
      inputSha256: row.inputSha256,
      historicalExpectedSha256: row.historicalExpectedSha256,
      currentExpectedSha256: row.currentExpectedSha256,
      provenance: row.provenance,
      justification: row.justification,
    },
  };
}

export function preservedHuggedInterpolationSnapshot(
  root: string,
  artifact: { path: string; sha256: string },
) {
  const paths = [
    "crates/vize_glyph/tests/snapshots/history_sfc_layout_wrapped_interpolation.snap.txt",
    "crates/vize_glyph/tests/snapshots/history_template_wrapped.snap.txt",
  ];
  if (!paths.includes(artifact.path)) return null;
  const row = authority(root).cases[paths.indexOf(artifact.path)];
  assert.equal(artifact.path, row.currentSnapshot.path);
  assert.equal(artifact.sha256, row.historicalExpectedSha256, "original snapshot pin changed");
  const current = fs.readFileSync(path.join(root, artifact.path));
  assert.equal(sha256(current), row.currentSnapshot.sha256, "current snapshot changed");
  assert.equal(current.toString(), row.currentExpected, "whole current reference changed");
  const historical = fs.readFileSync(path.join(root, row.historicalSnapshot.path));
  assert.equal(sha256(historical), artifact.sha256, "whole historical snapshot changed");
  return historical;
}
