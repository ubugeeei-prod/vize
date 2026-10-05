import assert from "node:assert/strict";
import { compareBytes } from "./compare.mjs";
import { loadFocusCases, rawBytes } from "./focus-history.ts";
import { decodeFocusHandled, validateFocusCurrentCapture } from "./focus-history-current-report.ts";
import {
  FOCUS_CURRENT_CAPTURE_SHA256,
  FOCUS_CURRENT_QUALIFICATION,
  loadFocusCurrentOracle,
} from "./focus-history-current-oracle.ts";

function nativeObservation(attempt: any) {
  try {
    return { bytes: decodeFocusHandled(rawBytes(attempt.stdoutBase64)) };
  } catch (error) {
    return { error: String(error) };
  }
}

function unavailable(error: string) {
  return { state: "not-compared", reason: "invalid-native-observation", error };
}

function laneComparison(lane: any, comparisons: any[], admitted: string) {
  return {
    state:
      lane.state !== admitted
        ? "failed"
        : comparisons.every((comparison) => comparison.state === "equal")
          ? "equal-reviewed-complete-current-output"
          : "current-output-drift",
    wholeComparisons: comparisons,
  };
}

/** Current v3 joins real native observation bytes to the unchanged legacy
 * authority. Historical native refusal bytes retain their original meaning. */
export function compareFocusNativeOutputs(root: string, capture: any, receipt: any) {
  const fixtures = loadFocusCases(root);
  validateFocusCurrentCapture(fixtures, capture, receipt);
  const oracle = loadFocusCurrentOracle(root);
  const rows = capture.rows.map((row: any, index: number) => {
    const expected = rawBytes(oracle.rows[index].legacy.attempts[0].stdoutBase64);
    const originals = row.legacy.attempts.map((attempt: any) => rawBytes(attempt.stdoutBase64));
    const native = row.native.attempts.map(nativeObservation);
    const legacyComparisons = originals.map((bytes: Buffer) => compareBytes(expected, bytes));
    const nativeComparisons = native.map((observation: any) =>
      observation.bytes
        ? compareBytes(expected, observation.bytes)
        : unavailable(observation.error),
    );
    const pairs = originals.flatMap((original: Buffer, legacyAttempt: number) =>
      native.map((observation: any, nativeAttempt: number) => ({
        legacyAttempt,
        nativeAttempt,
        comparison: observation.bytes
          ? compareBytes(original, observation.bytes)
          : unavailable(observation.error),
      })),
    );
    return {
      id: row.id,
      inputSha256: row.inputSha256,
      sourceSha256: row.sourceSha256,
      legacy: laneComparison(row.legacy, legacyComparisons, "captured"),
      native: laneComparison(row.native, nativeComparisons, "handled"),
      pairs,
      comparison:
        row.legacy.state !== "captured" || row.native.state !== "handled"
          ? { state: "not-compared", reason: "capture-failed" }
          : {
              state: pairs.every((pair: any) => pair.comparison.state === "equal")
                ? "equal"
                : "different",
            },
    };
  });
  return {
    schema: "vize.focus-history.current-output-comparison",
    version: 3,
    qualification: FOCUS_CURRENT_QUALIFICATION,
    oraclePacketSha256: FOCUS_CURRENT_CAPTURE_SHA256,
    sourceRevision: capture.sourceRevision,
    buildReceipt: receipt,
    // All 32 old raw attempts, refusal bodies and unreviewed metadata survive.
    historicalCapture: oracle,
    capture,
    rows,
    summary: {
      reviewedCurrentOutputOracles: 8,
      completeLegacyMatches: rows.filter(
        (row: any) => row.legacy.state === "equal-reviewed-complete-current-output",
      ).length,
      nativeHandled: capture.summary.nativeHandled,
      nativeEquivalent: rows.filter(
        (row: any) =>
          row.legacy.state === "equal-reviewed-complete-current-output" &&
          row.native.state === "equal-reviewed-complete-current-output" &&
          row.comparison.state === "equal",
      ).length,
      pairedComparisons: rows.filter((row: any) =>
        ["equal", "different"].includes(row.comparison.state),
      ).length,
      wholeCurrentComparisons: rows.reduce(
        (count: number, row: any) =>
          count + row.legacy.wholeComparisons.length + row.native.wholeComparisons.length,
        0,
      ),
      wholePairedComparisons: rows.reduce((count: number, row: any) => count + row.pairs.length, 0),
      currentOutputDrift: rows
        .flatMap((row: any) => [row.legacy, row.native])
        .filter((lane: any) => lane.state === "current-output-drift").length,
      captureFailures: capture.summary.captureFailures,
      historicalCompleteOutputAuthorities: 0,
    },
  };
}

export function validateFocusNativeComparison(root: string, comparison: any, receipt: any) {
  assert.deepEqual(comparison, compareFocusNativeOutputs(root, comparison.capture, receipt));
  return comparison.summary;
}
