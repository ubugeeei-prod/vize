import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { sha256 } from "./harness.mjs";
import { validateProductObserverReceipt } from "./observer-build.ts";
import {
  decodeFocusRefusal,
  FOCUS_CASES_SHA256,
  FOCUS_OBSERVER,
  loadFocusCases,
  rawBytes,
  utf8,
  validateFocusProbe,
} from "./focus-history.ts";
import { focusSummary, validateFocusCapture } from "./focus-history-report.ts";

export function collectFocusAttempts(
  binary: string,
  argv: string[],
  input: Buffer,
  run = spawnSync,
) {
  const attempts = [];
  for (let pass = 0; pass < 2; pass++) {
    let result;
    try {
      result = run(binary, argv, { input, timeout: 30_000, maxBuffer: 8 * 1024 * 1024 });
    } catch (error) {
      // No child streams/status exist when invocation itself throws. Keep the
      // actual exception separately and still make the second fresh attempt.
      attempts.push({
        stdoutBase64: "",
        stdoutSha256: sha256(Buffer.alloc(0)),
        stderrBase64: "",
        stderrSha256: sha256(Buffer.alloc(0)),
        exitStatus: null,
        signal: null,
        processError:
          error instanceof Error
            ? `${error.name}: ${error.message}`
            : `Thrown value: ${String(error)}`,
      });
      continue;
    }
    const stdout = result.stdout ?? Buffer.alloc(0);
    const stderr = result.stderr ?? Buffer.alloc(0);
    attempts.push({
      stdoutBase64: stdout.toString("base64"),
      stdoutSha256: sha256(stdout),
      stderrBase64: stderr.toString("base64"),
      stderrSha256: sha256(stderr),
      exitStatus: result.status,
      signal: result.signal,
      processError: result.error?.message ?? null,
    });
  }
  return attempts;
}

function captureLane(binary: string, fixture: any, native: boolean) {
  const argv = [native ? "--native" : "--legacy"];
  const lane: any = {
    state: "failed",
    argv,
    attempts: collectFocusAttempts(binary, argv, fixture.input),
  };
  try {
    let previous: Buffer | undefined;
    for (const attempt of lane.attempts) {
      assert.equal(attempt.processError, null);
      assert.equal(attempt.signal, null);
      assert.equal(attempt.exitStatus, 0);
      const bytes = rawBytes(attempt.stdoutBase64);
      assert.equal(rawBytes(attempt.stderrBase64).length, 0);
      assert(bytes.length > 0 && bytes.at(-1) === 10);
      if (previous) assert(bytes.equals(previous), "whole capture changed across fresh processes");
      previous = bytes;
      if (native) decodeFocusRefusal(bytes, fixture);
      else {
        const body = utf8(bytes);
        assert(
          body.startsWith("Case {\n") &&
            body.includes("\nRuleIdentity {\n") &&
            body.includes("\nObservation {\n"),
        );
      }
    }
    lane.state = native ? "refused" : "captured";
  } catch (error) {
    lane.error = String(error);
  }
  return lane;
}

export function runFocusCapture({
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
    legacy: captureLane(binaryPath, fixture, false),
    native: captureLane(binaryPath, fixture, true),
    comparison: { state: "not-compared", reason: "no-reviewed-complete-oracle" },
  }));
  const report = {
    schema: "vize.focus-history.capture",
    version: 1,
    acceptance: "unreviewed",
    sourceRevision: receipt.source.sourceRevision,
    fixtureSha256: FOCUS_CASES_SHA256,
    buildReceipt: receipt,
    rows,
    summary: focusSummary(rows),
  };
  validateFocusCapture(fixtures, report, receipt);
  return report;
}
