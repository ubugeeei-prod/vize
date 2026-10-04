import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { compareBytes } from "./compare.mjs";
import { readPinnedArtifact, sha256 } from "./harness.mjs";
import {
  decodeFocusRefusal,
  FOCUS_CASES_PATH,
  FOCUS_CASES_SHA256,
  loadFocusCases,
  rawBytes,
  utf8,
  validateFocusProbe,
} from "./focus-history.ts";
import { validateFocusCapture } from "./focus-history-report.ts";

export const FOCUS_CURRENT_INDEX =
  "tests/_fixtures/differential/linter-focus-current-output/index.json";
export const FOCUS_CURRENT_INDEX_SHA256 =
  "2b9191af075c63253cb7a549bbd312f71b0b97c7d152c69b11606baeb1153d01";

// These complete bytes were reviewed at the recorded actual source revision.
// They do not extend count-only historical assertions into whole-output authority.
export function loadFocusCurrentBaseline(
  root: string,
  indexBytes = fs.readFileSync(path.join(root, FOCUS_CURRENT_INDEX)),
) {
  assert.equal(sha256(indexBytes), FOCUS_CURRENT_INDEX_SHA256, "reviewed current baseline drift");
  const index = JSON.parse(utf8(indexBytes));
  assert.equal(index.schema, "vize.focus-history.current-output-baseline");
  assert.equal(index.version, 1);
  assert.equal(index.authority, "reviewed-current-output-only");
  assert.deepEqual(index.originalFixture, { path: FOCUS_CASES_PATH, sha256: FOCUS_CASES_SHA256 });
  const fixtures = loadFocusCases(root, readPinnedArtifact(root, index.originalFixture));
  const observedReceiptBytes = readPinnedArtifact(root, index.observed.buildReceipt);
  const observedReceipt = JSON.parse(utf8(observedReceiptBytes));
  assert.deepEqual(observedReceipt.source, index.observed.source);
  validateFocusProbe(observedReceipt);
  assert.deepEqual(index.review, {
    completeLegacyOutputs: 8,
    completeNativeRefusals: 8,
    freshProcessAttempts: 32,
    authoredOptions: "None",
    observedDefaults: { locale: "En", helpLevel: "Full" },
    historicalFullOutputAuthority: false,
    otherOptionsAuthority: false,
    exercisedFixAuthority: false,
    nativeEquivalenceAuthority: false,
  });
  assert.deepEqual(
    index.rows.map((row: any) => row.id),
    fixtures.map((fixture) => fixture.id),
  );
  const rows = fixtures.map((fixture, ordinal) => {
    const row = index.rows[ordinal];
    assert.equal(row.inputSha256, sha256(fixture.input));
    assert.equal(row.sourceSha256, fixture.authored.source_sha256);
    const legacy = readPinnedArtifact(root, row.legacy);
    const native = readPinnedArtifact(root, row.native);
    const refusal = decodeFocusRefusal(native, fixture);
    const body = utf8(legacy);
    assert(body.startsWith("Case {\n") && body.endsWith("\n"));
    const observation = body.indexOf("Observation {\n");
    assert(observation > 0);
    assert.equal(body.slice(0, observation), refusal.context);
    return { ...row, fixture, legacy, native };
  });
  return { index, observedReceipt, rows };
}

function compareLane(lane: any, expected: Buffer, expectedState: string, matchedState: string) {
  // Keep both complete streams in the enclosing capture, including failed ones.
  const wholeComparisons = lane.attempts.map((attempt: any) =>
    compareBytes(expected, rawBytes(attempt.stdoutBase64)),
  );
  return {
    state:
      lane.state !== expectedState
        ? "failed"
        : wholeComparisons.every((comparison: any) => comparison.state === "equal")
          ? matchedState
          : "current-output-drift",
    wholeComparisons,
  };
}

function currentSummary(rows: any[]) {
  return {
    plannedOriginalInputs: rows.length,
    legacyCurrentMatches: rows.filter((row) => row.legacy.state === "matched-current-output")
      .length,
    nativeRefusalMatches: rows.filter((row) => row.native.state === "matched-current-refusal")
      .length,
    currentOutputDrift: rows
      .flatMap((row) => [row.legacy, row.native])
      .filter((lane) => lane.state === "current-output-drift").length,
    captureFailures: rows
      .flatMap((row) => [row.legacy, row.native])
      .filter((lane) => lane.state === "failed").length,
    wholeCurrentComparisons: rows
      .flatMap((row) => [row.legacy, row.native])
      .reduce((count, lane) => count + lane.wholeComparisons.length, 0),
    historicalCompleteOutputAuthorities: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  };
}

export function compareFocusCurrentOutputs(root: string, capture: any, receipt: any) {
  const baseline = loadFocusCurrentBaseline(root);
  validateFocusCapture(
    baseline.rows.map((row) => row.fixture),
    capture,
    receipt,
  );
  const rows = capture.rows.map((row: any, ordinal: number) => ({
    id: row.id,
    legacy: compareLane(
      row.legacy,
      baseline.rows[ordinal].legacy,
      "captured",
      "matched-current-output",
    ),
    native: compareLane(
      row.native,
      baseline.rows[ordinal].native,
      "refused",
      "matched-current-refusal",
    ),
    comparison: { state: "not-compared", reason: "unprovided-rule" },
  }));
  return {
    schema: "vize.focus-history.current-output-result",
    version: 1,
    authority: "reviewed-current-output-only",
    baselineIndexSha256: FOCUS_CURRENT_INDEX_SHA256,
    observedSourceRevision: baseline.observedReceipt.source.sourceRevision,
    // Original build/status/streams remain complete; this does not change the
    // exploratory packet's schema or invent native whole-product provenance.
    capture,
    rows,
    summary: currentSummary(rows),
  };
}

export function validateFocusCurrentReport(root: string, report: any, receipt: any) {
  assert.deepEqual(report, compareFocusCurrentOutputs(root, report.capture, receipt));
  return report.summary;
}
