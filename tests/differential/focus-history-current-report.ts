import assert from "node:assert/strict";
import { sha256 } from "./harness.mjs";
import { keys, processShape } from "./focus-history-report.ts";
import { FOCUS_CASES_SHA256, rawBytes, utf8, validateFocusProbe } from "./focus-history.ts";

// This checks capture structure only. Complete current output authority is the
// immutable reviewed legacy packet, joined separately by the v3 comparison.
export function decodeFocusHandled(bytes: Buffer) {
  assert(bytes.length > 0 && bytes.at(-1) === 10);
  const outcome = JSON.parse(utf8(bytes));
  assert.equal(outcome.state, "handled", `actual native outcome: ${utf8(bytes)}`);
  keys(outcome, ["state", "observation"]);
  assert.equal(typeof outcome.observation, "string");
  const observation = Buffer.from(outcome.observation);
  const body = utf8(observation);
  assert(body.startsWith("Case {\n") && body.endsWith("\n"));
  assert(body.includes("\nRuleIdentity {\n") && body.includes("\nObservation {\n"));
  return observation;
}

export function currentFocusSummary(rows: any[]) {
  return {
    plannedOriginalWitnesses: rows.length,
    legacyCaptured: rows.filter((row) => row.legacy.state === "captured").length,
    nativeHandled: rows.filter((row) => row.native.state === "handled").length,
    captureFailures: rows
      .flatMap((row) => [row.legacy, row.native])
      .filter((lane) => lane.state === "failed").length,
    acceptedCompleteOracles: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  };
}

/** Current v2 is explicit; historical capture-v1 remains refusal-only. */
export function validateFocusCurrentCapture(fixtures: any[], report: any, receipt: any) {
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
  assert.equal(report.version, 2);
  assert.equal(report.acceptance, "unreviewed");
  assert.equal(report.fixtureSha256, FOCUS_CASES_SHA256);
  assert.match(report.sourceRevision, /^[a-f0-9]{40}$/);
  assert.equal(report.sourceRevision, receipt.source.sourceRevision);
  assert.deepEqual(report.buildReceipt, receipt);
  // The original producer already declares capture-only and emits Handled.
  // Its exact v1 probe remains truthful for both old and current captures.
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
    assert.deepEqual(row.comparison, { state: "not-compared", reason: "capture-only" });
    for (const [name, expected] of [
      ["legacy", "captured"],
      ["native", "handled"],
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
      assert.equal(lane.attempts.length, 2, "both fresh attempts survive every failure");
      let previous: Buffer | undefined;
      for (const attempt of lane.attempts) {
        processShape(attempt);
        const bytes = rawBytes(attempt.stdoutBase64);
        const stderr = rawBytes(attempt.stderrBase64);
        assert.equal(attempt.stdoutSha256, sha256(bytes));
        assert.equal(attempt.stderrSha256, sha256(stderr));
        if (lane.state === "failed") continue;
        assert.equal(attempt.exitStatus, 0);
        assert.equal(attempt.signal, null);
        assert.equal(attempt.processError, null);
        assert.equal(stderr.length, 0);
        assert(bytes.length > 0 && bytes.at(-1) === 10);
        if (previous) assert(bytes.equals(previous), "whole fresh capture did not repeat");
        previous = bytes;
        if (name === "native") decodeFocusHandled(bytes);
        else {
          const body = utf8(bytes);
          assert(body.startsWith("Case {\n"));
          assert(body.includes("\nRuleIdentity {\n") && body.includes("\nObservation {\n"));
        }
      }
      if (lane.state === "failed") assert(typeof lane.error === "string" && lane.error.length > 0);
    }
  }
  assert.deepEqual(report.summary, currentFocusSummary(report.rows));
  return report.summary;
}
