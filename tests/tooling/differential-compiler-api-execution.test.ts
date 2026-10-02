import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  COMPILER_TARGET_CONTRACTS,
  loadCompilerApiManifest,
  runCompilerApiPack,
} from "../differential/compiler-api.ts";
import { buildProductObserver } from "../differential/observer-build.ts";
import { compilerFailure, type CompilerBuildReceipt } from "../differential/compiler-api-types.ts";
import { summarizeNativeAcceptance } from "../differential/acceptance-rates.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

for (const [target, contract] of Object.entries(COMPILER_TARGET_CONTRACTS)) {
  void test(`shared compiler ${target} corpus observes five complete outputs from its source-built artifact`, (t) => {
    const evidenceDir = path.resolve(
      root,
      process.env.VIZE_COMPILER_API_EVIDENCE_DIR ?? "target/differential/compiler-api",
      target,
    );
    const sourceRevision = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: root,
      encoding: "utf8",
    }).trim();
    let built: { binaryPath: string | null; receipt: CompilerBuildReceipt | null } = {
      binaryPath: null,
      receipt: null,
    };
    let buildFailure: string | null = null;
    try {
      built = buildProductObserver({
        spec: contract.spec,
        repoRoot: root,
        targetDir: path.resolve(root, process.env.CARGO_TARGET_DIR ?? "target"),
        evidenceDir,
        profile: process.env.CI ? "ci" : "dev",
        offline: !process.env.CI,
      });
    } catch (error) {
      buildFailure = compilerFailure(error);
    }
    const manifestPath = path.join(
      root,
      `tests/_fixtures/differential/compiler/${contract.name}-fix-history.manifest.json`,
    );
    const report = runCompilerApiPack({
      manifestPath,
      repoRoot: root,
      sourceRevision,
      ...built,
      buildFailure,
      evidenceDir,
    });
    t.diagnostic(`Compiler public API evidence: ${evidenceDir}`);
    assert.equal(buildFailure, null, buildFailure ?? "observer build failed");
    assert.deepEqual(
      report.summary,
      {
        plannedCases: 5,
        legacyMatches: 5,
        baselineDrift: 0,
        legacyFailures: 0,
        nativeUnsupported: 5,
        nativeHandled: 0,
        nativeEquivalent: 0,
        pairedComparisons: 0,
      },
      JSON.stringify(report.rows, null, 2),
    );
    const loaded = loadCompilerApiManifest(manifestPath, root);
    assert.deepEqual(
      summarizeNativeAcceptance(loaded, report, {
        sourceRevision: report.sourceRevision,
      }).total,
      {
        planned: 5,
        nativeHandled: 0,
        nativeEquivalent: 0,
        unsupported: 5,
        legacyBacked: 0,
        unverified: 0,
      },
    );
  });
}
