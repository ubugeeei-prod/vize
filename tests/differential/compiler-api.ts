import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { isDeepStrictEqual } from "node:util";
import { sha256, validateResultEnvelope } from "./harness.mjs";
import { validateProductObserverReceipt } from "./observer-build.ts";
import {
  compilerFailure,
  type CompilerApiReport,
  type CompilerAttempt,
  type CompilerBuildReceipt,
  type CompilerObservation,
  type CompilerObservationPacket,
  type CompilerRow,
} from "./compiler-api-types.ts";

import { loadCompilerApiManifest, validateCompilerOutput } from "./compiler-api-manifest.ts";
export {
  COMPILER_OBSERVER_SPEC,
  COMPILER_TARGET_CONTRACTS,
  loadCompilerApiManifest,
  originalSfcOptions,
} from "./compiler-api-manifest.ts";
const ARGV = ["--observe"];
const NO_NATIVE = "whole-product native compiler path unavailable";

type LoadedCompilerPack = ReturnType<typeof loadCompilerApiManifest>;

function retain(evidenceDir: string | null, filename: string, value: unknown) {
  if (evidenceDir === null) return;
  assert(path.isAbsolute(evidenceDir));
  fs.mkdirSync(evidenceDir, { recursive: true });
  fs.writeFileSync(path.join(evidenceDir, filename), `${JSON.stringify(value, null, 2)}\n`);
}

function observe(binaryPath: string, evidenceDir: string | null, label: string): CompilerAttempt {
  const result = spawnSync(binaryPath, ARGV, { timeout: 30_000, maxBuffer: 4 * 1024 * 1024 });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const attempt = {
    argv: ARGV,
    exitStatus: result.status,
    signal: result.signal,
    processError: result.error?.message ?? null,
    stdoutBase64: stdout.toString("base64"),
    stderrBase64: (result.stderr ?? Buffer.alloc(0)).toString("base64"),
    stdoutSha256: sha256(stdout),
  };
  retain(evidenceDir, `${label}.attempt.json`, attempt);
  return attempt;
}

export function compilerApiRows(
  loaded: LoadedCompilerPack,
  attempts: CompilerAttempt[],
  failure: string | null,
): CompilerRow[] {
  let observations: CompilerObservation[] = [];
  try {
    if (failure !== null) throw new Error(failure);
    assert.equal(attempts.length, 2, "two real compiler observations are required");
    for (const attempt of attempts) {
      assert.deepEqual(attempt.argv, ARGV);
      assert.equal(attempt.exitStatus, 0);
      assert.equal(attempt.signal, null);
      assert.equal(attempt.processError, null);
      assert.equal(attempt.stderrBase64, "");
      assert.equal(sha256(Buffer.from(attempt.stdoutBase64, "base64")), attempt.stdoutSha256);
    }
    assert.equal(
      attempts[0].stdoutBase64,
      attempts[1].stdoutBase64,
      "compiler observations did not repeat exactly",
    );
    const packet: CompilerObservationPacket = JSON.parse(
      Buffer.from(attempts[0].stdoutBase64, "base64").toString(),
    );
    assert.deepEqual(Object.keys(packet).sort(), ["cases", "schema", "version"]);
    assert.equal(packet.schema, loaded.contract.packetSchema);
    assert.equal(packet.version, 1);
    assert.deepEqual(
      packet.cases.map((row) => row.id),
      loaded.contract.ids,
    );
    observations = packet.cases;
    for (const [index, actual] of observations.entries()) {
      const fixture = loaded.cases[index];
      assert.deepEqual(Object.keys(actual).sort(), [
        "entrypoint",
        "id",
        "options",
        "result",
        "source",
      ]);
      assert.equal(actual.source, fixture.input.toString());
      assert.equal(actual.entrypoint, fixture.entrypoint);
      assert.deepEqual(
        actual.options,
        fixture.originalOptions,
        "original compiler options drifted",
      );
      validateCompilerOutput(actual.result, loaded.target);
    }
  } catch (error) {
    failure = compilerFailure(error);
  }
  return loaded.cases.map((fixture, index) => {
    const failed = failure !== null;
    const result = failed ? null : observations[index].result;
    return {
      id: fixture.id,
      target: loaded.target,
      legacy: {
        state: failed ? "failed" : "completed",
        verdict: failed
          ? "failed"
          : isDeepStrictEqual(result, fixture.expected)
            ? "matched-reference"
            : "baseline-drift",
        inputSha256: sha256(fixture.input),
        result,
        ...(failure !== null ? { error: failure } : {}),
      },
      native: { state: "unsupported", reason: fixture.adapters.reasons.native },
      comparison: { state: "not-compared", reason: NO_NATIVE },
    };
  });
}

function summarize(loaded: LoadedCompilerPack, rows: CompilerRow[]) {
  return {
    plannedCases: loaded.cases.length,
    legacyMatches: rows.filter((row) => row.legacy.verdict === "matched-reference").length,
    baselineDrift: rows.filter((row) => row.legacy.verdict === "baseline-drift").length,
    legacyFailures: rows.filter((row) => row.legacy.state === "failed").length,
    nativeUnsupported: loaded.cases.length,
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  };
}

export function validateCompilerApiReport(
  loaded: LoadedCompilerPack,
  report: CompilerApiReport,
  context: {
    repoRoot: string;
    binaryPath: string | null;
    sourceRevision: string;
    evidenceDir?: string | null;
  },
) {
  assert.match(context.sourceRevision, /^[a-f0-9]{40}$/);
  validateResultEnvelope(loaded, report, context.sourceRevision);
  if (report.failure === null) {
    assert(context.binaryPath && path.isAbsolute(context.binaryPath));
    assert(report.buildReceipt, "a source-built compiler receipt is required");
    validateProductObserverReceipt(report.buildReceipt, {
      spec: loaded.contract.spec,
      repoRoot: context.repoRoot,
      binaryPath: context.binaryPath,
    });
    assert.equal(report.buildReceipt.source.sourceRevision, context.sourceRevision);
    // Reobserve the real source-bound executable; a recomputed JSON hash does
    // not turn an invented compiler transcript into execution evidence.
    assert.deepEqual(
      observe(context.binaryPath, context.evidenceDir ?? null, "revalidation"),
      report.attempts[0],
      "actual compiler transcript changed",
    );
  } else assert.equal(typeof report.failure, "string");
  const rows = compilerApiRows(loaded, report.attempts, report.failure);
  assert.deepEqual(report.rows, rows, "compiler rows or native completion were forged");
  const summary = summarize(loaded, rows);
  assert.deepEqual(report.summary, summary);
  return summary;
}

export function runCompilerApiPack({
  manifestPath,
  repoRoot,
  sourceRevision,
  binaryPath,
  receipt = null,
  buildFailure = null,
  evidenceDir = null,
}: {
  manifestPath: string;
  repoRoot: string;
  sourceRevision: string;
  binaryPath: string | null;
  receipt?: CompilerBuildReceipt | null;
  buildFailure?: string | null;
  evidenceDir?: string | null;
}) {
  const loaded = loadCompilerApiManifest(manifestPath, repoRoot);
  const attempts: CompilerAttempt[] = [];
  let failure: string | null = null;
  try {
    if (buildFailure !== null) throw new Error(buildFailure);
    assert(
      binaryPath && path.isAbsolute(binaryPath),
      "an explicit source-built observer is required",
    );
    validateProductObserverReceipt(receipt, { spec: loaded.contract.spec, repoRoot, binaryPath });
    assert(receipt, "a source-built compiler receipt is required");
    assert.equal(receipt.source.sourceRevision, sourceRevision);
    attempts.push(observe(binaryPath, evidenceDir, "first"));
    attempts.push(observe(binaryPath, evidenceDir, "repeat"));
  } catch (error) {
    failure = compilerFailure(error);
  }
  const rows = compilerApiRows(loaded, attempts, failure);
  const report: CompilerApiReport = {
    schema: "vize.differential.result",
    version: 1,
    product: "compiler",
    sourceRevision,
    manifestSha256: loaded.manifestSha256,
    buildReceipt: failure !== null ? null : receipt,
    attempts,
    failure,
    rows,
    summary: summarize(loaded, rows),
  };
  const context = { repoRoot, binaryPath, sourceRevision, evidenceDir };
  retain(evidenceDir, "unverified-report.json", report);
  try {
    validateCompilerApiReport(loaded, report, context);
  } catch (error) {
    report.failure = `compiler report validation failed: ${compilerFailure(error)}`;
    report.rows = compilerApiRows(loaded, attempts, report.failure);
    report.summary = summarize(loaded, report.rows);
    validateCompilerApiReport(loaded, report, context);
  }
  retain(evidenceDir, "report.json", report);
  return report;
}
