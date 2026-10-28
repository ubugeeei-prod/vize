import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { inspect } from "node:util";

import { repoRoot } from "../_helpers/realworld-patch.ts";
import { createDualLspFixture } from "./support/dual-lsp-fixture.ts";
import {
  diagnosticBarrierCount,
  diagnosticDeadlineMs,
  diagnosticQuietMs,
  qualifyDocumentRevisions,
  runQualifiedOperationPacket,
  type DiagnosticEvidence,
} from "./support/dual-lsp-oracles.ts";
import {
  DualLspProduct,
  dualLspProviderRoot,
  dualLspToolReceipt,
  productOrder,
  stableOperationTranscript,
  type ProductKind,
} from "./support/dual-lsp-product.ts";
import {
  assertRevisionReceiptReachability,
  distribution,
  summarizeScheduleReceipt,
  summarizeDiagnostics,
  summarizeSamples,
  writeDualLspReport,
  type MeasuredSample,
  type SampleOrderReceipt,
} from "./support/dual-lsp-report.ts";
import { gitHead } from "./support/process-metrics.ts";

type RevisionEvidence = { v1: DiagnosticEvidence; v2: DiagnosticEvidence };

type RevisionReceiptScope =
  | { product: ProductKind; phase: "fresh"; sampleIndex: number }
  | { product: ProductKind; phase: "warm"; sessionIndex: number };

type ProductEvidence = {
  fresh: MeasuredSample[];
  warm: MeasuredSample[];
  freshDiagnostics: RevisionEvidence[];
  warmDiagnostics: RevisionEvidence[];
  warmStartupMs: number[];
};

function emptyProductEvidence(): ProductEvidence {
  return {
    fresh: [],
    warm: [],
    freshDiagnostics: [],
    warmDiagnostics: [],
    warmStartupMs: [],
  };
}

type Options = {
  freshSamples: number;
  warmSamples: number;
  outputDir: string;
};

const products: readonly ProductKind[] = ["vize", "vue-language-tools"];

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

function revisionReceiptSha256(scope: RevisionReceiptScope, evidence: RevisionEvidence): string {
  return sha256(
    JSON.stringify({
      scope,
      evidence,
      custody: {
        diagnosticBarrierCount,
        diagnosticDeadlineMs,
        diagnosticQuietMs,
      },
    }),
  );
}

function resetOwnedWorkspaceCaches(workspaceDir: string): void {
  for (const target of [
    path.join(workspaceDir, ".vize"),
    path.join(workspaceDir, "node_modules/.vize"),
  ]) {
    fs.rmSync(target, { force: true, recursive: true });
  }
}

async function closeProduct(product: DualLspProduct | undefined, failure: unknown): Promise<void> {
  if (!product) return;
  try {
    await product.shutdown();
  } catch (shutdownError) {
    await product.kill();
    if (failure == null) throw shutdownError;
  }
}

async function collectFresh(
  count: number,
  evidence: Record<ProductKind, ProductEvidence>,
  transcriptHashes: string[],
  schedule: SampleOrderReceipt[],
): Promise<void> {
  for (let round = 0; round < count; round += 1) {
    const fixture = createDualLspFixture();
    const pairTranscripts = new Map<ProductKind, string>();
    const order = [...productOrder(round)];
    schedule.push({ sampleIndex: round, order });
    try {
      for (const kind of order) {
        let product: DualLspProduct | undefined;
        let failure: unknown;
        try {
          const startupStartedAt = performance.now();
          product = await DualLspProduct.start(kind, fixture.workspaceDir);
          const startupMs = performance.now() - startupStartedAt;
          const diagnostics = await qualifyDocumentRevisions(product, fixture);
          const operation = await runQualifiedOperationPacket(product, fixture);
          const sample: MeasuredSample = {
            ...operation,
            revisionReceiptSha256: revisionReceiptSha256(
              { product: kind, phase: "fresh", sampleIndex: round },
              diagnostics,
            ),
            startupMs,
            diagnosticV1QualificationMs: diagnostics.v1.qualificationMs,
            diagnosticV2QualificationMs: diagnostics.v2.qualificationMs,
          };
          evidence[kind].freshDiagnostics.push(diagnostics);
          evidence[kind].fresh.push(sample);
          pairTranscripts.set(kind, stableOperationTranscript(product.primaryOperations()));
        } catch (error) {
          failure = error;
          throw error;
        } finally {
          await closeProduct(product, failure);
          resetOwnedWorkspaceCaches(fixture.workspaceDir);
        }
      }
      const vize = pairTranscripts.get("vize");
      const vue = pairTranscripts.get("vue-language-tools");
      assert.ok(vize && vue, `fresh round ${round} did not run both products`);
      assert.equal(vize, vue, `fresh round ${round} changed the primary wire sequence`);
      transcriptHashes.push(sha256(vize));
    } finally {
      fixture.dispose();
    }
  }
}

async function collectWarm(
  count: number,
  evidence: Record<ProductKind, ProductEvidence>,
  transcriptHashes: string[],
  schedule: SampleOrderReceipt[],
): Promise<void> {
  const fixture = createDualLspFixture();
  const sessions = new Map<ProductKind, DualLspProduct>();
  const sessionRevisionReceipts = new Map<ProductKind, string>();
  let failure: unknown;
  try {
    for (const kind of productOrder(0)) {
      const startupStartedAt = performance.now();
      const product = await DualLspProduct.start(kind, fixture.workspaceDir);
      evidence[kind].warmStartupMs.push(performance.now() - startupStartedAt);
      sessions.set(kind, product);
      const diagnostics = await qualifyDocumentRevisions(product, fixture);
      evidence[kind].warmDiagnostics.push(diagnostics);
      sessionRevisionReceipts.set(
        kind,
        revisionReceiptSha256({ product: kind, phase: "warm", sessionIndex: 0 }, diagnostics),
      );
    }
    for (let round = 0; round < count; round += 1) {
      const order = [...productOrder(round)];
      schedule.push({ sampleIndex: round, order });
      for (const kind of order) {
        const product = sessions.get(kind);
        const revisionReceiptSha256 = sessionRevisionReceipts.get(kind);
        if (product == null || revisionReceiptSha256 == null) {
          throw new Error(`${kind} warm session lost its product or revision receipt`);
        }
        const operation = await runQualifiedOperationPacket(product, fixture);
        const sample: MeasuredSample = { ...operation, revisionReceiptSha256 };
        evidence[kind].warm.push(sample);
      }
    }
    const vizeSession = sessions.get("vize");
    const vueSession = sessions.get("vue-language-tools");
    if (vizeSession == null || vueSession == null) {
      throw new Error("warm comparison lost one of its product sessions");
    }
    const vize = stableOperationTranscript(vizeSession.primaryOperations());
    const vue = stableOperationTranscript(vueSession.primaryOperations());
    assert.equal(vize, vue, "warm products did not receive an identical primary wire sequence");
    transcriptHashes.push(sha256(vize));
  } catch (error) {
    failure = error;
    throw error;
  } finally {
    const shutdownFailures: unknown[] = [];
    for (const session of sessions.values()) {
      try {
        await closeProduct(session, failure);
      } catch (error) {
        shutdownFailures.push(error);
      }
    }
    fixture.dispose();
    if (failure == null && shutdownFailures.length > 0) {
      throw new AggregateError(shutdownFailures, "warm product shutdown failed");
    }
  }
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
  const startedAt = new Date().toISOString();
  try {
    await collectFresh(config.freshSamples, evidence, transcriptHashes, freshSchedule);
    await collectWarm(config.warmSamples, evidence, transcriptHashes, warmSchedule);
    const commit = gitHead();
    const tools = dualLspToolReceipt();
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
      provider: {
        packageJsonSha256: sha256(
          fs.readFileSync(path.join(dualLspProviderRoot, "package.json"), "utf8"),
        ),
        lockfileSha256: sha256(
          fs.readFileSync(path.join(dualLspProviderRoot, "pnpm-lock.yaml"), "utf8"),
        ),
      },
      resolvedTools: tools,
    };
    for (const kind of products) {
      assert.equal(evidence[kind].fresh.length, config.freshSamples);
      assert.equal(evidence[kind].warm.length, config.warmSamples);
      assert.equal(evidence[kind].freshDiagnostics.length, config.freshSamples);
      assert.equal(evidence[kind].warmDiagnostics.length, 1);
    }
    const report = {
      schemaVersion: 2,
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
        bridge:
          "Vue tsserver/request is forwarded to typescript.tsserverRequest; didOpen/didChange are mirrored to both halves",
      },
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
          freshReceiptIds.forEach((revisionReceiptSha256, sampleIndex) => {
            assert.equal(
              evidence[kind].fresh[sampleIndex]?.revisionReceiptSha256,
              revisionReceiptSha256,
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
                fresh: freshReceiptIds.map((revisionReceiptSha256, sampleIndex) => ({
                  sampleIndex,
                  revisionReceiptSha256,
                })),
                warmSession: warmReceiptIds.map((revisionReceiptSha256, sessionIndex) => ({
                  sessionIndex,
                  revisionReceiptSha256,
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
      schemaVersion: 2,
      status: "failed",
      commit: gitHead(),
      startedAt,
      finishedAt: new Date().toISOString(),
      failure: error instanceof Error ? (error.stack ?? error.message) : inspect(error),
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
