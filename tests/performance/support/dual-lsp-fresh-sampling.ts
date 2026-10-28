import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { performance } from "node:perf_hooks";

import {
  assertOwnedWorkspaceCachesAbsent,
  createDualLspFixture,
  resetOwnedWorkspaceCaches,
} from "./dual-lsp-fixture.ts";
import { qualifyDocumentRevisions, diagnosticBarrierCount } from "./dual-lsp-oracles.ts";
import { runQualifiedOperationPacket } from "./dual-lsp-operation-oracles.ts";
import {
  DualLspProduct,
  primaryOperationReceipt,
  productOrder,
  stableOperationTranscript,
  type PrimaryOperationReceipt,
  type ProductKind,
} from "./dual-lsp-product.ts";
import {
  selectAuthoritativeFailure,
  type MeasuredSample,
  type SampleOrderReceipt,
} from "./dual-lsp-report.ts";
import {
  closeProduct,
  oracleEquivalenceReceipt,
  pairValue,
  primaryOperationPairReceipt,
  products,
  revisionReceiptSha256,
  throwSelectedFailure,
  type CustodyEvidence,
  type FreshCacheResetReceipt,
  type ProductEvidence,
  type QualifiedSample,
} from "./dual-lsp-sampling.ts";

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

export async function collectFresh(
  count: number,
  evidence: Record<ProductKind, ProductEvidence>,
  transcriptHashes: string[],
  schedule: SampleOrderReceipt[],
  custody: CustodyEvidence,
): Promise<void> {
  for (let round = 0; round < count; round += 1) {
    const fixture = createDualLspFixture();
    const pairTranscripts = new Map<ProductKind, string>();
    const pairOperations = new Map<ProductKind, PrimaryOperationReceipt>();
    const pairQualified = new Map<ProductKind, QualifiedSample>();
    const order = [...productOrder(round)];
    schedule.push({ sampleIndex: round, order });
    let primaryFailed = false;
    let primaryFailure: unknown;
    const cleanupFailures: unknown[] = [];
    let pendingReset: FreshCacheResetReceipt | undefined;
    for (const [orderIndex, kind] of order.entries()) {
      if (pendingReset != null) {
        try {
          assertOwnedWorkspaceCachesAbsent(pendingReset.paths);
          pendingReset.verificationStage = "before-next-product";
          pendingReset.absenceVerified = true;
        } catch (error) {
          cleanupFailures.push(error);
        }
        pendingReset = undefined;
        if (cleanupFailures.length > 0) break;
      }

      let product: DualLspProduct | undefined;
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
        pairQualified.set(kind, { diagnostics, sample });
        const operations = product.primaryOperations();
        pairTranscripts.set(kind, stableOperationTranscript(operations));
        pairOperations.set(kind, primaryOperationReceipt(operations, 1, diagnosticBarrierCount));
      } catch (error) {
        primaryFailed = true;
        primaryFailure = error;
      }

      cleanupFailures.push(...(await closeProduct(product)));
      try {
        const cacheReceipt: FreshCacheResetReceipt = {
          sampleIndex: round,
          orderIndex,
          product: kind,
          nextProduct: order[orderIndex + 1] ?? null,
          paths: resetOwnedWorkspaceCaches(fixture.workspaceDir),
          verificationStage: null,
          absenceVerified: false,
        };
        custody.freshCacheResets.push(cacheReceipt);
        pendingReset = cacheReceipt;
      } catch (error) {
        cleanupFailures.push(error);
      }
      if (primaryFailed || cleanupFailures.length > 0) break;
    }

    if (pendingReset != null) {
      try {
        assertOwnedWorkspaceCachesAbsent(pendingReset.paths);
        pendingReset.verificationStage = "before-fixture-dispose";
        pendingReset.absenceVerified = true;
      } catch (error) {
        cleanupFailures.push(error);
      }
    }

    let transcriptHash: string | undefined;
    let operationReceipt: Record<string, unknown> | undefined;
    let equivalenceReceipt: Record<string, unknown> | undefined;
    if (!primaryFailed && cleanupFailures.length === 0) {
      try {
        const vize = pairTranscripts.get("vize");
        const vue = pairTranscripts.get("vue-language-tools");
        assert.ok(vize && vue, `fresh round ${round} did not run both products`);
        assert.equal(vize, vue, `fresh round ${round} changed the primary wire sequence`);
        transcriptHash = sha256(vize);
        operationReceipt = primaryOperationPairReceipt("fresh", round, order, pairOperations);
        equivalenceReceipt = oracleEquivalenceReceipt("fresh", round, order, pairQualified);
        custody.qualifiedCandidates.freshRounds += 1;
      } catch (error) {
        primaryFailed = true;
        primaryFailure = error;
      }
    }

    try {
      fixture.dispose();
    } catch (error) {
      cleanupFailures.push(error);
    }
    throwSelectedFailure(
      selectAuthoritativeFailure(
        primaryFailed,
        primaryFailure,
        cleanupFailures,
        `fresh round ${round} cleanup failed`,
      ),
    );
    if (transcriptHash == null || operationReceipt == null || equivalenceReceipt == null) {
      throw new Error(`fresh round ${round} omitted its accepted custody receipt`);
    }
    for (const kind of products) {
      const accepted = pairValue(pairQualified, kind);
      evidence[kind].freshDiagnostics.push(accepted.diagnostics);
      evidence[kind].fresh.push(accepted.sample);
    }
    transcriptHashes.push(transcriptHash);
    custody.primaryOperations.fresh.push(operationReceipt);
    custody.oracleEquivalence.fresh.push(equivalenceReceipt);
  }
}
