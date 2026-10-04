import assert from "node:assert/strict";
import { sha256 } from "./harness.mjs";
import {
  decodeFocusRefusal,
  FOCUS_CASES_SHA256,
  rawBytes,
  utf8,
  validateFocusProbe,
} from "./focus-history.ts";

function keys(value: any, expected: string[]) {
  assert(value && typeof value === "object" && !Array.isArray(value));
  assert.deepEqual(
    Object.keys(value).sort(),
    expected.toSorted(),
    "exact capture protocol fields required",
  );
}

function processShape(attempt: any) {
  keys(attempt, [
    "stdoutBase64",
    "stdoutSha256",
    "stderrBase64",
    "stderrSha256",
    "exitStatus",
    "signal",
    "processError",
  ]);
  assert(
    attempt.exitStatus === null ||
      (Number.isInteger(attempt.exitStatus) && attempt.exitStatus >= 0),
  );
  assert(
    attempt.signal === null ||
      (typeof attempt.signal === "string" && /^SIG[A-Z0-9]+$/.test(attempt.signal)),
  );
  assert(
    attempt.processError === null ||
      (typeof attempt.processError === "string" && attempt.processError.length > 0),
  );
}

export function focusSummary(rows: any[]) {
  return {
    plannedOriginalWitnesses: rows.length,
    legacyCaptured: rows.filter((row) => row.legacy.state === "captured").length,
    nativeRefused: rows.filter((row) => row.native.state === "refused").length,
    captureFailures: rows
      .flatMap((row) => [row.legacy, row.native])
      .filter((lane) => lane.state === "failed").length,
    acceptedCompleteOracles: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  };
}

export function validateFocusCapture(fixtures: any[], report: any, receipt: any) {
  keys(report, [
    "schema",
    "version",
    "acceptance",
    "sourceRevision",
    "fixtureSha256",
    "buildReceipt",
    "rows",
    "summary",
  ]);
  assert.equal(report.schema, "vize.focus-history.capture");
  assert.equal(report.version, 1);
  assert.equal(report.acceptance, "unreviewed");
  assert.equal(report.fixtureSha256, FOCUS_CASES_SHA256);
  assert.match(
    report.sourceRevision,
    /^[a-f0-9]{40}$/,
    "actual committed source revision required",
  );
  assert.equal(report.sourceRevision, receipt.source.sourceRevision);
  assert.deepEqual(report.buildReceipt, receipt);
  validateFocusProbe(receipt);
  assert.deepEqual(
    report.rows.map((row: any) => row.id),
    fixtures.map((fixture) => fixture.id),
  );
  for (const [index, fixture] of fixtures.entries()) {
    const row = report.rows[index];
    keys(row, [
      "id",
      "inputSha256",
      "inputBase64",
      "sourceSha256",
      "comparison",
      "legacy",
      "native",
    ]);
    assert.equal(row.inputSha256, sha256(fixture.input));
    assert(rawBytes(row.inputBase64).equals(fixture.input));
    assert.equal(row.sourceSha256, fixture.authored.source_sha256);
    assert.deepEqual(row.comparison, {
      state: "not-compared",
      reason: "no-reviewed-complete-oracle",
    });
    for (const [name, expected] of [
      ["legacy", "captured"],
      ["native", "refused"],
    ]) {
      const lane = row[name];
      assert([expected, "failed"].includes(lane.state));
      keys(
        lane,
        lane.state === "failed"
          ? ["state", "argv", "attempts", "error"]
          : ["state", "argv", "attempts"],
      );
      assert.deepEqual(lane.argv, [name === "legacy" ? "--legacy" : "--native"]);
      assert.equal(lane.provenance, undefined, "unaccepted captures confer no native provenance");
      assert.equal(lane.attempts.length, 2, "both fresh attempts are retained even after failure");
      let previous: Buffer | undefined;
      for (const attempt of lane.attempts) {
        processShape(attempt);
        const bytes = rawBytes(attempt.stdoutBase64);
        const stderr = rawBytes(attempt.stderrBase64);
        assert.equal(attempt.stdoutSha256, sha256(bytes));
        assert.equal(attempt.stderrSha256, sha256(stderr));
        assert.equal(
          attempt.referenceComparison,
          undefined,
          "there is no reviewed complete oracle",
        );
        if (lane.state === "failed") continue;
        assert.equal(attempt.exitStatus, 0);
        assert.equal(attempt.signal, null);
        assert.equal(attempt.processError, null);
        assert.equal(stderr.length, 0);
        assert(bytes.length > 0 && bytes.at(-1) === 10);
        if (previous) assert(bytes.equals(previous), "whole fresh-process capture did not repeat");
        previous = bytes;
        if (name === "native") decodeFocusRefusal(bytes, fixture);
        else {
          const body = utf8(bytes);
          assert(body.startsWith("Case {\n"));
          assert(body.includes("\nRuleIdentity {\n") && body.includes("\nObservation {\n"));
          // Structure/repeat checks cannot authenticate a complete historical golden.
        }
      }
      if (lane.state === "failed") {
        assert.equal(typeof lane.error, "string");
        assert(lane.error.length > 0);
      } else assert.equal(lane.error, undefined);
    }
  }
  assert.deepEqual(report.summary, focusSummary(report.rows));
  return report.summary;
}
