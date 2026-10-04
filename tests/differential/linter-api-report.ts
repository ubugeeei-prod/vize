import assert from "node:assert/strict";
import { compareBytes } from "./compare.mjs";
import { sha256, validateResultEnvelope } from "./harness.mjs";
import type { loadLinterManifest } from "./linter-api.ts";
import { validateNativeContract, validateNativeRow } from "./linter-native.ts";

export function summary(rows: any[]) {
  return {
    plannedCases: rows.length,
    legacyMatches: rows.filter((row) => row.legacy.verdict === "matched-reference").length,
    legacyFailures: rows.filter((row) => row.legacy.state === "failed").length,
    baselineDrift: rows.filter((row) => row.legacy.verdict === "baseline-drift").length,
    nativeUnsupported: rows.filter((row) => row.native.state === "unsupported").length,
    nativeFailures: rows.filter((row) => row.native.state === "failed").length,
    nativeHandled: rows.filter((row) => row.native.state === "completed").length,
    nativeEquivalent: rows.filter(
      (row) =>
        row.native.state === "completed" &&
        row.comparison.state === "equal" &&
        row.native.verdict === "matched-reference" &&
        row.legacy.verdict === "matched-reference",
    ).length,
    pairedComparisons: rows.filter((row) => ["equal", "different"].includes(row.comparison.state))
      .length,
  };
}

export function validateLinterReport(
  loaded: ReturnType<typeof loadLinterManifest>,
  report: any,
  receipt: any,
) {
  validateResultEnvelope(loaded, report, receipt.source.sourceRevision);
  assert.deepEqual(report.buildReceipt, receipt);
  validateNativeContract(receipt);
  for (const row of report.rows) {
    const fixture = loaded.cases.find((fixture: any) => fixture.id === row.id);
    assert(fixture);
    assert.equal(row.target, "lint");
    const observations = validateNativeRow(fixture, row.native, receipt);
    if (row.legacy.state === "completed" && row.native.state === "completed") {
      assert.deepEqual(
        row.comparison,
        compareBytes(Buffer.from(row.legacy.attempts[0].stdoutBase64, "base64"), observations[0]),
      );
      for (const original of row.legacy.attempts)
        for (const actual of observations) {
          assert.deepEqual(
            row.comparison,
            compareBytes(Buffer.from(original.stdoutBase64, "base64"), actual),
          );
        }
    } else {
      assert.deepEqual(row.comparison, { state: "not-compared" });
    }
    assert.deepEqual(row.legacy.argv, fixture.argv);
    assert.equal(row.legacy.inputSha256, sha256(fixture.input));
    assert(["completed", "failed"].includes(row.legacy.state));
    assert(Array.isArray(row.legacy.attempts) && row.legacy.attempts.length <= 2);
    let previous: Buffer | undefined;
    for (const attempt of row.legacy.attempts) {
      const bytes = Buffer.from(attempt.stdoutBase64, "base64");
      const stderr = Buffer.from(attempt.stderrBase64, "base64");
      assert.equal(attempt.stdoutSha256, sha256(bytes));
      assert.equal(attempt.stderrSha256, sha256(stderr));
      assert.deepEqual(attempt.referenceComparison, compareBytes(fixture.expected, bytes));
      if (row.legacy.state === "completed") {
        assert.equal(attempt.exitStatus, 0);
        assert.equal(attempt.signal, null);
        assert.equal(attempt.processError, null);
        assert.equal(stderr.length, 0);
        if (previous) assert(bytes.equals(previous), "actual complete observation did not repeat");
      }
      previous = bytes;
    }
    if (row.legacy.state === "completed") {
      assert.equal(row.legacy.attempts.length, 2, "both fresh process observations are required");
      const matched = row.legacy.attempts.every(
        (attempt: any) => attempt.referenceComparison.state === "equal",
      );
      assert.equal(row.legacy.verdict, matched ? "matched-reference" : "baseline-drift");
    } else {
      assert.equal(row.legacy.verdict, "failed");
      assert(typeof row.legacy.error === "string" && row.legacy.error.length > 0);
    }
  }
  assert.deepEqual(report.summary, summary(report.rows), "inconsistent actual result accounting");
  return report.summary;
}
