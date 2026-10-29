import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { performance } from "node:perf_hooks";

import { createDualLspFixture } from "./dual-lsp-fixture.ts";
import { diagnosticBarrierCount, qualifyDocumentRevisions } from "./dual-lsp-oracles.ts";
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
  primaryOperationPairReceipt,
  products,
  revisionReceiptSha256,
  throwSelectedFailure,
  type CustodyEvidence,
  type ProductEvidence,
  type QualifiedSample,
  type RevisionEvidence,
} from "./dual-lsp-sampling.ts";

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

export async function collectWarm(
  count: number,
  evidence: Record<ProductKind, ProductEvidence>,
  transcriptHashes: string[],
  schedule: SampleOrderReceipt[],
  custody: CustodyEvidence,
): Promise<void> {
  const fixture = createDualLspFixture();
  const sessions = new Map<ProductKind, DualLspProduct>();
  const sessionStartupMs = new Map<ProductKind, number>();
  const sessionQualified = new Map<ProductKind, RevisionEvidence>();
  const sessionRevisionReceipts = new Map<ProductKind, string>();
  const candidateSamples: Record<ProductKind, MeasuredSample[]> = {
    vize: [],
    "vue-language-tools": [],
  };
  const acceptedEquivalence: Array<Record<string, unknown>> = [];
  let primaryFailed = false;
  let primaryFailure: unknown;
  let transcriptHash: string | undefined;
  let operationReceipt: Record<string, unknown> | undefined;
  const warmupBoundaries: Array<Record<string, unknown>> = [];
  try {
    for (const kind of productOrder(0)) {
      const startupStartedAt = performance.now();
      const product = await DualLspProduct.start(kind, fixture.workspaceDir);
      sessionStartupMs.set(kind, performance.now() - startupStartedAt);
      sessions.set(kind, product);
      const diagnostics = await qualifyDocumentRevisions(product, fixture);
      sessionQualified.set(kind, diagnostics);
      sessionRevisionReceipts.set(
        kind,
        revisionReceiptSha256({ product: kind, phase: "warm", sessionIndex: 0 }, diagnostics),
      );
    }
    for (let round = 0; round < count; round += 1) {
      const order = [...productOrder(round)];
      const pairQualified = new Map<ProductKind, QualifiedSample>();
      schedule.push({ sampleIndex: round, order });
      for (const kind of order) {
        const product = sessions.get(kind);
        const receiptSha256 = sessionRevisionReceipts.get(kind);
        if (product == null || receiptSha256 == null) {
          throw new Error(`${kind} warm session lost its product or revision receipt`);
        }
        const operation = await runQualifiedOperationPacket(product, fixture);
        const sample: MeasuredSample = {
          ...operation,
          revisionReceiptSha256: receiptSha256,
        };
        candidateSamples[kind].push(sample);
        const diagnostics = sessionQualified.get(kind);
        if (diagnostics == null) throw new Error(`${kind} warm session lost its qualification`);
        pairQualified.set(kind, { diagnostics, sample });
      }
      acceptedEquivalence.push(oracleEquivalenceReceipt("warm", round, order, pairQualified));
      custody.qualifiedCandidates.warmRounds += 1;
    }
    const vizeSession = sessions.get("vize");
    const vueSession = sessions.get("vue-language-tools");
    if (vizeSession == null || vueSession == null) {
      throw new Error("warm comparison lost one of its product sessions");
    }
    const vizeOperations = vizeSession.primaryOperations();
    const vueOperations = vueSession.primaryOperations();
    const vize = stableOperationTranscript(vizeOperations);
    const vue = stableOperationTranscript(vueOperations);
    assert.equal(vize, vue, "warm products did not receive an identical primary wire sequence");
    transcriptHash = sha256(vize);
    operationReceipt = primaryOperationPairReceipt(
      "warm",
      null,
      productOrder(0),
      new Map<ProductKind, PrimaryOperationReceipt>([
        ["vize", primaryOperationReceipt(vizeOperations, count, diagnosticBarrierCount)],
        [
          "vue-language-tools",
          primaryOperationReceipt(vueOperations, count, diagnosticBarrierCount),
        ],
      ]),
    );
    for (const kind of products) {
      const diagnostics = sessionQualified.get(kind);
      const receiptSha256 = sessionRevisionReceipts.get(kind);
      if (diagnostics == null || receiptSha256 == null) {
        throw new Error(`${kind} warmup boundary lost its diagnostic receipt`);
      }
      const samples = candidateSamples[kind];
      assert.ok(sessionStartupMs.has(kind));
      assert.equal(samples.length, count);
      assert.ok(samples.every((sample) => sample.revisionReceiptSha256 === receiptSha256));
      const payload = {
        product: kind,
        sessionIndex: 0,
        initializedSessions: 1,
        eventOrder: ["initialize", "qualify-v1", "qualify-v2", "measure-packets"],
        diagnosticQualification: {
          v1Fingerprint: diagnostics.v1.fingerprint,
          v2Fingerprint: diagnostics.v2.fingerprint,
          provenVersion: diagnostics.v2.provenVersion,
          revisionReceiptSha256: receiptSha256,
        },
        qualificationCompletedBeforePacketIndex: 0,
        measuredPacketIndices: samples.map((_, sampleIndex) => sampleIndex),
        measuredPacketCount: samples.length,
        allPacketsBoundToRevisionReceipt: true,
      };
      warmupBoundaries.push({ ...payload, sha256: sha256(JSON.stringify(payload)) });
    }
  } catch (error) {
    primaryFailed = true;
    primaryFailure = error;
  }

  const cleanupFailures: unknown[] = [];
  for (const session of sessions.values()) {
    cleanupFailures.push(...(await closeProduct(session)));
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
      "warm product cleanup failed",
    ),
  );
  if (transcriptHash == null || operationReceipt == null) {
    throw new Error("warm comparison omitted its accepted custody receipt");
  }
  for (const kind of products) {
    const startupMs = sessionStartupMs.get(kind);
    const diagnostics = sessionQualified.get(kind);
    if (startupMs == null || diagnostics == null) {
      throw new Error(`${kind} warm acceptance omitted its session evidence`);
    }
    evidence[kind].warmStartupMs.push(startupMs);
    evidence[kind].warmDiagnostics.push(diagnostics);
    evidence[kind].warm.push(...candidateSamples[kind]);
  }
  transcriptHashes.push(transcriptHash);
  custody.primaryOperations.warm.push(operationReceipt);
  custody.oracleEquivalence.warm.push(...acceptedEquivalence);
  custody.warmupBoundaries.push(...warmupBoundaries);
}
