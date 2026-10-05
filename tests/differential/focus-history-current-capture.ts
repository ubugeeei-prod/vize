import assert from "node:assert/strict";
import path from "node:path";
import { sha256 } from "./harness.mjs";
import { validateProductObserverReceipt } from "./observer-build.ts";
import { collectFocusAttempts } from "./focus-history-capture.ts";
import {
  FOCUS_CASES_SHA256,
  FOCUS_OBSERVER,
  loadFocusCases,
  rawBytes,
  utf8,
  validateFocusProbe,
} from "./focus-history.ts";
import {
  currentFocusSummary,
  decodeFocusHandled,
  validateFocusCurrentCapture,
} from "./focus-history-current-report.ts";

export function captureCurrentFocusLane(
  binary: string,
  fixture: any,
  native: boolean,
  invoke?: Parameters<typeof collectFocusAttempts>[3],
) {
  const argv = [native ? "--native" : "--legacy"];
  const lane: any = {
    state: "failed",
    argv,
    attempts: collectFocusAttempts(binary, argv, fixture.input, invoke),
  };
  try {
    let previous: Buffer | undefined;
    for (const attempt of lane.attempts) {
      assert.equal(attempt.processError, null);
      assert.equal(attempt.signal, null);
      assert.equal(attempt.exitStatus, 0);
      assert.equal(rawBytes(attempt.stderrBase64).length, 0);
      const bytes = rawBytes(attempt.stdoutBase64);
      assert(bytes.length > 0 && bytes.at(-1) === 10);
      if (previous) assert(bytes.equals(previous), "whole capture changed across fresh processes");
      previous = bytes;
      if (native) decodeFocusHandled(bytes);
      else {
        const body = utf8(bytes);
        assert(body.startsWith("Case {\n"));
        assert(body.includes("\nRuleIdentity {\n") && body.includes("\nObservation {\n"));
      }
    }
    lane.state = native ? "handled" : "captured";
  } catch (error) {
    lane.error = String(error);
  }
  return lane;
}

export function runFocusCurrentCapture({
  repoRoot,
  binaryPath,
  receipt,
}: {
  repoRoot: string;
  binaryPath: string;
  receipt: any;
}) {
  assert(path.isAbsolute(binaryPath));
  validateProductObserverReceipt(receipt, { spec: FOCUS_OBSERVER, repoRoot, binaryPath });
  validateFocusProbe(receipt);
  const fixtures = loadFocusCases(repoRoot);
  const rows = fixtures.map((fixture: any) => ({
    id: fixture.id,
    inputBase64: fixture.input.toString("base64"),
    inputSha256: sha256(fixture.input),
    sourceSha256: fixture.authored.source_sha256,
    legacy: captureCurrentFocusLane(binaryPath, fixture, false),
    native: captureCurrentFocusLane(binaryPath, fixture, true),
    comparison: { state: "not-compared", reason: "capture-only" },
  }));
  const report = {
    schema: "vize.focus-history.capture",
    version: 2,
    acceptance: "unreviewed",
    sourceRevision: receipt.source.sourceRevision,
    fixtureSha256: FOCUS_CASES_SHA256,
    buildReceipt: receipt,
    rows,
    summary: currentFocusSummary(rows),
  };
  validateFocusCurrentCapture(fixtures, report, receipt);
  return report;
}
