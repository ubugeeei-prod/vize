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
  diagnosticQuietMs,
  qualifyDocumentRevisions,
  runQualifiedOperationPacket,
  type DiagnosticEvidence,
} from "./support/dual-lsp-oracles.ts";
import {
  DualLspProduct,
  dualLspToolReceipt,
  productOrder,
  stableOperationTranscript,
  type ProductKind,
} from "./support/dual-lsp-product.ts";
import {
  summarizeSamples,
  writeDualLspReport,
  type MeasuredSample,
} from "./support/dual-lsp-report.ts";
import { gitHead } from "./support/process-metrics.ts";

type ProductEvidence = {
  fresh: MeasuredSample[];
  warm: MeasuredSample[];
  freshDiagnostics: Array<{ v1: DiagnosticEvidence; v2: DiagnosticEvidence }>;
  warmDiagnostics: Array<{ v1: DiagnosticEvidence; v2: DiagnosticEvidence }>;
};

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
      option(
        "--out",
        path.join(repoRoot, "target/vize-tests/metrics/lsp-dual-server"),
      ),
    ),
  };
  if (process.env.LSP_DUAL_ALLOW_SMOKE !== "1") {
    assert.ok(result.freshSamples >= 100, "qualification requires at least 100 fresh process samples");
    assert.ok(result.warmSamples >= 100, "qualification requires at least 100 warm samples");
  }
  return result;
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function resetOwnedWorkspaceCaches(workspaceDir: string): void {
  for (const target of [path.join(workspaceDir, ".vize"), path.join(workspaceDir, "node_modules/.vize")]) {
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
): Promise<void> {
  for (let round = 0; round < count; round += 1) {
    const fixture = createDualLspFixture();
    const pairTranscripts = new Map<ProductKind, string>();
    try {
      for (const kind of productOrder(round)) {
        let product: DualLspProduct | undefined;
        let failure: unknown;
        try {
          const startedAt = performance.now();
          product = await DualLspProduct.start(kind, fixture.workspaceDir);
          const diagnostics = await qualifyDocumentRevisions(product, fixture);
          const sample = (await runQualifiedOperationPacket(product, fixture)) as MeasuredSample;
          sample.qualifiedFreshProcessMs = performance.now() - startedAt;
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
): Promise<void> {
  const fixture = createDualLspFixture();
  const sessions = new Map<ProductKind, DualLspProduct>();
  let failure: unknown;
  try {
    for (const kind of productOrder(0)) {
      const product = await DualLspProduct.start(kind, fixture.workspaceDir);
      sessions.set(kind, product);
      evidence[kind].warmDiagnostics.push(await qualifyDocumentRevisions(product, fixture));
    }
    for (let round = 0; round < count; round += 1) {
      for (const kind of productOrder(round)) {
        evidence[kind].warm.push(
          await runQualifiedOperationPacket(sessions.get(kind)!, fixture),
        );
      }
    }
    const vize = stableOperationTranscript(sessions.get("vize")!.primaryOperations());
    const vue = stableOperationTranscript(sessions.get("vue-language-tools")!.primaryOperations());
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
  const evidence = Object.fromEntries(
    products.map((kind) => [
      kind,
      { fresh: [], warm: [], freshDiagnostics: [], warmDiagnostics: [] },
    ]),
  ) as Record<ProductKind, ProductEvidence>;
  const transcriptHashes: string[] = [];
  const startedAt = new Date().toISOString();
  try {
    await collectFresh(config.freshSamples, evidence, transcriptHashes);
    await collectWarm(config.warmSamples, evidence, transcriptHashes);
    const report = {
      schemaVersion: 1,
      status: "passed",
      commit: gitHead(),
      startedAt,
      finishedAt: new Date().toISOString(),
      configuration: {
        freshSamplesPerProduct: config.freshSamples,
        warmSamplesPerProduct: config.warmSamples,
        schedule: "ABBA by round; both products receive byte-identical primary method/params sequences",
        diagnosticQuietMs,
        diagnosticBarriers: diagnosticBarrierCount,
      },
      tools: dualLspToolReceipt(),
      fixture: {
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
      },
      wire: {
        transcriptSha256: transcriptHashes,
        bridge:
          "Vue tsserver/request is forwarded to typescript.tsserverRequest; didOpen/didChange are mirrored to both halves",
      },
      products: Object.fromEntries(
        products.map((kind) => [
          kind,
          {
            ...evidence[kind],
            freshSummary: summarizeSamples(evidence[kind].fresh),
            warmSummary: summarizeSamples(evidence[kind].warm),
          },
        ]),
      ),
      metricBoundary: {
        comparable: ["same-wire latency", "complete process-tree RSS"],
        attributionOnly: ["Vize Rust allocation/function profile", "Vue and TypeScript LS Node CPU profiles"],
      },
    };
    writeDualLspReport(config.outputDir, report);
  } catch (error) {
    writeDualLspReport(config.outputDir, {
      schemaVersion: 1,
      status: "failed",
      commit: gitHead(),
      startedAt,
      finishedAt: new Date().toISOString(),
      failure: error instanceof Error ? error.stack ?? error.message : inspect(error),
      admittedSamples: Object.fromEntries(
        products.map((kind) => [kind, { fresh: evidence[kind].fresh.length, warm: evidence[kind].warm.length }]),
      ),
      rule: "failed, stale, duplicate, incomplete, extra, or timed-out results are not admitted as fast samples",
    });
    throw error;
  }
}

await main();
