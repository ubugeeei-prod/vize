import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { inspect } from "node:util";

import { repoRoot } from "../_helpers/realworld-patch.ts";
import {
  BUILD_RECIPE,
  PROFILE_BUILD_RECIPE,
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../differential/build-receipt.ts";
import { collectFresh } from "./support/dual-lsp-fresh-sampling.ts";
import {
  diagnosticBarrierCount,
  diagnosticDeadlineMs,
  diagnosticQuietMs,
} from "./support/dual-lsp-oracles.ts";
import { dualLspToolReceipt, type ProductKind } from "./support/dual-lsp-product.ts";
import { dualLspProviderDependencyReceipt } from "./support/dual-lsp-provider-receipt.ts";
import {
  assertRevisionReceiptReachability,
  cleanupFailuresFor,
  distribution,
  summarizeDiagnostics,
  summarizeSamples,
  summarizeScheduleReceipt,
  writeDualLspReport,
  type SampleOrderReceipt,
} from "./support/dual-lsp-report.ts";
import {
  emptyCustodyEvidence,
  emptyProductEvidence,
  products,
  revisionReceiptSha256,
  type CustodyEvidence,
  type ProductEvidence,
} from "./support/dual-lsp-sampling.ts";
import { collectWarm } from "./support/dual-lsp-warm-sampling.ts";
import { gitHead } from "./support/process-metrics.ts";

type Options = {
  freshSamples: number;
  warmSamples: number;
  outputDir: string;
};

function option(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index < 0 ? fallback : (process.argv[index + 1] ?? "");
}

function parsePositiveInteger(name: string, fallback: number): number {
  const value = Number(option(name, String(fallback)));
  assert.ok(Number.isSafeInteger(value) && value > 0, `${name} must be a positive integer`);
  return value;
}

function options(): Options {
  const result = {
    freshSamples: parsePositiveInteger("--fresh", 100),
    warmSamples: parsePositiveInteger("--warm", 100),
    outputDir: path.resolve(
      option("--out", path.join(repoRoot, "target/vize-tests/metrics/lsp-dual-server")),
    ),
  };
  if (process.env.LSP_DUAL_ALLOW_SMOKE !== "1") {
    assert.ok(
      result.freshSamples >= 100,
      "qualification requires at least 100 fresh process samples",
    );
    assert.ok(result.warmSamples >= 100, "qualification requires at least 100 warm samples");
  }
  return result;
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function vizeBinaryCustody(sourceSha: string): Record<string, unknown> {
  const configuredBinary = process.env.VIZE_LSP_BIN;
  assert.ok(
    configuredBinary && path.isAbsolute(configuredBinary),
    "VIZE_LSP_BIN must identify an absolute source-built binary",
  );
  const binaryPath = path.resolve(configuredBinary);
  const relativeBinaryPath = path.relative(repoRoot, binaryPath);
  assert.ok(
    relativeBinaryPath !== "" &&
      relativeBinaryPath !== ".." &&
      !relativeBinaryPath.startsWith(`..${path.sep}`),
    "VIZE_LSP_BIN must remain inside the exact source checkout",
  );
  const receiptPath = `${binaryPath}.differential-build.json`;
  const receiptText = fs.readFileSync(receiptPath, "utf8");
  const receipt = JSON.parse(receiptText) as Record<string, unknown>;
  const expected = expectedBuildIdentity(repoRoot, relativeBinaryPath);
  const expectedRecipe =
    relativeBinaryPath === binaryRelativePath("ci")
      ? BUILD_RECIPE
      : relativeBinaryPath === binaryRelativePath("ci-opt")
        ? PROFILE_BUILD_RECIPE
        : null;
  assert.ok(expectedRecipe, `unsupported Vize benchmark binary path: ${relativeBinaryPath}`);
  validateBuildReceipt(receipt, expected, expectedRecipe);
  assert.equal(receipt.sourceRevision, sourceSha, "Vize build receipt source drifted");
  assert.equal(receipt.binaryPath, relativeBinaryPath, "Vize build receipt path drifted");
  const binarySha256 = createHash("sha256").update(fs.readFileSync(binaryPath)).digest("hex");
  assert.equal(binarySha256, receipt.binarySha256, "Vize binary bytes drifted from its receipt");
  return {
    configuredBinary: binaryPath,
    relativeBinaryPath,
    binarySha256,
    receiptPath,
    receiptSha256: sha256(receiptText),
    receipt,
  };
}

function describeFailure(error: unknown): string {
  return error instanceof Error ? (error.stack ?? error.message) : inspect(error);
}

function failureReceipt(error: unknown): string | Record<string, unknown> {
  const cleanupFailures = cleanupFailuresFor(error);
  if (cleanupFailures.length === 0) return describeFailure(error);
  return {
    authoritative: describeFailure(error),
    cleanupFailures: cleanupFailures.map(describeFailure),
  };
}

async function main(): Promise<void> {
  const config = options();
  const evidence: Record<ProductKind, ProductEvidence> = {
    vize: emptyProductEvidence(),
    "vue-language-tools": emptyProductEvidence(),
  };
  const transcriptHashes: string[] = [];
  const freshSchedule: SampleOrderReceipt[] = [];
  const warmSchedule: SampleOrderReceipt[] = [];
  const custody: CustodyEvidence = emptyCustodyEvidence();
  const startedAt = new Date().toISOString();
  try {
    const commit = gitHead();
    const tools = dualLspToolReceipt();
    const vizeBinary = vizeBinaryCustody(commit);
    const fixture = {
      sourceV1Sha256: sha256(
        fs.readFileSync(
          path.join(repoRoot, "tests/_fixtures/performance/lsp-dual-server/sources/App.v1.vue.txt"),
          "utf8",
        ),
      ),
      sourceV2Sha256: sha256(
        fs.readFileSync(
          path.join(repoRoot, "tests/_fixtures/performance/lsp-dual-server/sources/App.v2.vue.txt"),
          "utf8",
        ),
      ),
    };
    const sourceToolInputs = {
      sourceSha: commit,
      fixture,
      vizeBinary,
      provider: dualLspProviderDependencyReceipt(),
      resolvedTools: tools,
    };
    await collectFresh(config.freshSamples, evidence, transcriptHashes, freshSchedule, custody);
    await collectWarm(config.warmSamples, evidence, transcriptHashes, warmSchedule, custody);
    for (const kind of products) {
      assert.equal(evidence[kind].fresh.length, config.freshSamples);
      assert.equal(evidence[kind].warm.length, config.warmSamples);
      assert.equal(evidence[kind].freshDiagnostics.length, config.freshSamples);
      assert.equal(evidence[kind].warmDiagnostics.length, 1);
    }
    assert.equal(custody.freshCacheResets.length, config.freshSamples * products.length);
    assert.ok(custody.freshCacheResets.every((receipt) => receipt.absenceVerified));
    assert.equal(custody.primaryOperations.fresh.length, config.freshSamples);
    assert.equal(custody.primaryOperations.warm.length, 1);
    assert.equal(custody.oracleEquivalence.fresh.length, config.freshSamples);
    assert.equal(custody.oracleEquivalence.warm.length, config.warmSamples);
    assert.equal(custody.warmupBoundaries.length, products.length);
    const report = {
      schemaVersion: 3,
      status: "passed",
      commit,
      startedAt,
      finishedAt: new Date().toISOString(),
      configuration: {
        freshSamplesPerProduct: config.freshSamples,
        warmSamplesPerProduct: config.warmSamples,
        schedule:
          "ABBA by round; both products receive byte-identical primary method/params sequences",
        diagnosticQuietMs,
        diagnosticDeadlineMs,
        diagnosticBarriers: diagnosticBarrierCount,
      },
      tools,
      fixture,
      sourceToolReceipt: {
        ...sourceToolInputs,
        sha256: sha256(JSON.stringify(sourceToolInputs)),
      },
      scheduleReceipt: {
        fresh: summarizeScheduleReceipt(freshSchedule),
        warm: summarizeScheduleReceipt(warmSchedule),
      },
      wire: {
        transcriptSha256: transcriptHashes,
        primaryOperationReceipts: custody.primaryOperations,
        bridge:
          "Vue tsserver/request is forwarded to typescript.tsserverRequest; didOpen/didChange are mirrored to both halves",
      },
      cacheResetReceipt: {
        rows: custody.freshCacheResets,
        sha256: sha256(JSON.stringify(custody.freshCacheResets)),
      },
      warmupBoundaryReceipts: custody.warmupBoundaries,
      oracleEquivalenceReceipts: custody.oracleEquivalence,
      products: Object.fromEntries(
        products.map((kind) => {
          const freshReceiptIds = evidence[kind].freshDiagnostics.map((diagnostics, sampleIndex) =>
            revisionReceiptSha256({ product: kind, phase: "fresh", sampleIndex }, diagnostics),
          );
          const warmReceiptIds = evidence[kind].warmDiagnostics.map((diagnostics, sessionIndex) =>
            revisionReceiptSha256({ product: kind, phase: "warm", sessionIndex }, diagnostics),
          );
          assert.equal(
            new Set(freshReceiptIds).size,
            config.freshSamples,
            `${kind} fresh samples must each bind their own revision receipt`,
          );
          freshReceiptIds.forEach((receiptSha256, sampleIndex) => {
            assert.equal(
              evidence[kind].fresh[sampleIndex]?.revisionReceiptSha256,
              receiptSha256,
              `${kind} fresh sample ${sampleIndex} bound the wrong revision receipt`,
            );
          });
          assert.equal(
            new Set(warmReceiptIds).size,
            1,
            `${kind} warm samples must bind one unchanged-session revision receipt`,
          );
          const warmReceiptSha256 = warmReceiptIds[0];
          if (warmReceiptSha256 == null) {
            throw new Error(`${kind} warm session omitted its revision receipt`);
          }
          evidence[kind].warm.forEach((sample, sampleIndex) => {
            assert.equal(
              sample.revisionReceiptSha256,
              warmReceiptSha256,
              `${kind} warm sample ${sampleIndex} bound the wrong revision receipt`,
            );
          });
          return [
            kind,
            {
              ...evidence[kind],
              diagnosticReceiptIndex: {
                fresh: freshReceiptIds.map((receiptSha256, sampleIndex) => ({
                  sampleIndex,
                  revisionReceiptSha256: receiptSha256,
                })),
                warmSession: warmReceiptIds.map((receiptSha256, sessionIndex) => ({
                  sessionIndex,
                  revisionReceiptSha256: receiptSha256,
                })),
              },
              revisionReceiptReachability: {
                fresh: assertRevisionReceiptReachability(evidence[kind].fresh, freshReceiptIds),
                warm: assertRevisionReceiptReachability(evidence[kind].warm, warmReceiptIds),
              },
              freshSummary: summarizeSamples(evidence[kind].fresh),
              warmSummary: summarizeSamples(evidence[kind].warm),
              startupSummary: {
                freshMs: distribution(evidence[kind].fresh.map((sample) => sample.startupMs!)),
                warmSessionMs: distribution(evidence[kind].warmStartupMs),
              },
              diagnosticSummary: {
                fresh: summarizeDiagnostics(evidence[kind].freshDiagnostics),
                warmSession: summarizeDiagnostics(evidence[kind].warmDiagnostics),
              },
            },
          ];
        }),
      ),
      metricBoundary: {
        comparable: ["same-wire latency", "complete process-tree RSS"],
        attributionOnly: [
          "Vize Rust allocation/function profile",
          "Vue and TypeScript LS Node CPU profiles",
        ],
      },
    };
    writeDualLspReport(config.outputDir, report);
  } catch (error) {
    writeDualLspReport(config.outputDir, {
      schemaVersion: 3,
      status: "failed",
      commit: gitHead(),
      startedAt,
      finishedAt: new Date().toISOString(),
      failure: failureReceipt(error),
      qualifiedCandidates: custody.qualifiedCandidates,
      partialCustody: custody,
      admittedSamples: Object.fromEntries(
        products.map((kind) => [
          kind,
          { fresh: evidence[kind].fresh.length, warm: evidence[kind].warm.length },
        ]),
      ),
      rule: "failed, stale, duplicate, incomplete, extra, or timed-out results are not admitted as fast samples",
    });
    throw error;
  }
}

await main();
