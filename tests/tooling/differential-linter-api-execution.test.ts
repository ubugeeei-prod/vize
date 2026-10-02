import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { buildProductObserver } from "../differential/observer-build.ts";
import { LINTER_OBSERVER, runLinterApiPack } from "../differential/linter-api.ts";
import { sha256 } from "../differential/harness.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("shared linter corpus retains all 36 complete actual public observations from source-built Rust", (t) => {
  const evidenceDir = path.resolve(
    root,
    process.env.VIZE_LINTER_API_EVIDENCE_DIR ?? "target/differential/linter-api",
  );
  const built = buildProductObserver({
    spec: LINTER_OBSERVER,
    repoRoot: root,
    targetDir: path.resolve(root, process.env.CARGO_TARGET_DIR ?? "target"),
    evidenceDir,
    profile: process.env.CI ? "ci" : "dev",
    offline: !process.env.CI,
  });
  const report = runLinterApiPack({
    manifestPath: path.join(root, "tests/_fixtures/differential/linter/manifest.json"),
    repoRoot: root,
    ...built,
  });
  fs.writeFileSync(path.join(evidenceDir, "report.json"), `${JSON.stringify(report, null, 2)}\n`);
  captureOriginalComponentNames(built, evidenceDir);
  t.diagnostic(
    `Complete raw public observations, Cargo artifact and build receipt: ${evidenceDir}`,
  );
  assert.deepEqual(
    report.summary,
    {
      plannedCases: 36,
      legacyMatches: 36,
      legacyFailures: 0,
      baselineDrift: 0,
      nativeUnsupported: 36,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
    JSON.stringify(report.rows, null, 2),
  );
});

// Preparation only: actual raw observations must become immutable complete
// oracles before these eight inputs can enter the shared accepted registry.
function captureOriginalComponentNames(
  built: ReturnType<typeof buildProductObserver>,
  evidenceDir: string,
) {
  const inputPath = "crates/vize_patina/tests/fixtures/component-name-history/cases.json";
  const bytes = fs.readFileSync(path.join(root, inputPath));
  const inputSha256 = sha256(bytes);
  assert.equal(inputSha256, "867a29d7c0a92c17d59fcaa6063be1e3649fb92375024fcd2932a57f27fe062c");
  const cases = JSON.parse(bytes.toString());
  assert.equal(cases.length, 8);
  const rows = cases.map((fixture: any) => {
    const input = Buffer.from(JSON.stringify(fixture));
    const attempts = Array.from({ length: 2 }, () => {
      const result = spawnSync(built.binaryPath, ["--current-api"], {
        input,
        timeout: 30_000,
        maxBuffer: 8 * 1024 * 1024,
      });
      const stdout = result.stdout ?? Buffer.alloc(0);
      const stderr = result.stderr ?? Buffer.alloc(0);
      return {
        stdoutBase64: stdout.toString("base64"),
        stdoutSha256: sha256(stdout),
        stderrBase64: stderr.toString("base64"),
        stderrSha256: sha256(stderr),
        exitStatus: result.status,
        signal: result.signal,
        processError: result.error?.message ?? null,
      };
    });
    return {
      id: fixture.id,
      argv: ["--current-api"],
      inputBase64: input.toString("base64"),
      inputSha256: sha256(input),
      attempts,
      repeatedCompleteBytes: attempts[0].stdoutBase64 === attempts[1].stdoutBase64,
      native: { state: "unsupported", reason: "whole-product native linter path unavailable" },
      comparison: { state: "not-compared" },
    };
  });
  const capture = {
    schema: "vize.linter-original-capture",
    version: 1,
    sourceRevision: built.receipt.source.sourceRevision,
    sourceTree: built.receipt.source.sourceTree,
    history: "eaafa5a1f67883277407fcf1c5f3f2b2101ef2ed",
    inputArtifact: { path: inputPath, sha256: inputSha256 },
    buildReceipt: built.receipt,
    rows,
    fixtureAcceptance: "pending-complete-oracle-registration",
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  };
  fs.writeFileSync(
    path.join(evidenceDir, "component-name-original-capture.json"),
    `${JSON.stringify(capture, null, 2)}\n`,
  );
  for (const row of rows) {
    assert(row.repeatedCompleteBytes, row.id);
    for (const attempt of row.attempts) {
      assert.equal(attempt.exitStatus, 0, row.id);
      assert.equal(attempt.signal, null, row.id);
      assert.equal(attempt.processError, null, row.id);
      assert.equal(attempt.stderrBase64, "", row.id);
      assert(Buffer.from(attempt.stdoutBase64, "base64").length > 0, row.id);
    }
  }
}
