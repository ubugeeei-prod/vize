import assert from "node:assert/strict";
import { createHash } from "node:crypto";

import type { OwnedCachePathReceipt } from "./dual-lsp-fixture.ts";
import {
  diagnosticBarrierCount,
  diagnosticDeadlineMs,
  diagnosticQuietMs,
  type DiagnosticEvidence,
} from "./dual-lsp-oracles.ts";
import type { DualLspProduct, PrimaryOperationReceipt, ProductKind } from "./dual-lsp-product.ts";
import {
  retainPrimaryCleanupFailures,
  type FailureSelection,
  type MeasuredSample,
} from "./dual-lsp-report.ts";

export type RevisionEvidence = { v1: DiagnosticEvidence; v2: DiagnosticEvidence };

type RevisionReceiptScope =
  | { product: ProductKind; phase: "fresh"; sampleIndex: number }
  | { product: ProductKind; phase: "warm"; sessionIndex: number };

export type ProductEvidence = {
  fresh: MeasuredSample[];
  warm: MeasuredSample[];
  freshDiagnostics: RevisionEvidence[];
  warmDiagnostics: RevisionEvidence[];
  warmStartupMs: number[];
};

export type QualifiedSample = { diagnostics: RevisionEvidence; sample: MeasuredSample };

export type FreshCacheResetReceipt = {
  sampleIndex: number;
  orderIndex: number;
  product: ProductKind;
  nextProduct: ProductKind | null;
  paths: OwnedCachePathReceipt[];
  verificationStage: "before-next-product" | "before-fixture-dispose" | null;
  absenceVerified: boolean;
};

export type CustodyEvidence = {
  qualifiedCandidates: { freshRounds: number; warmRounds: number };
  freshCacheResets: FreshCacheResetReceipt[];
  primaryOperations: {
    fresh: Array<Record<string, unknown>>;
    warm: Array<Record<string, unknown>>;
  };
  oracleEquivalence: {
    fresh: Array<Record<string, unknown>>;
    warm: Array<Record<string, unknown>>;
  };
  warmupBoundaries: Array<Record<string, unknown>>;
};

export const products: readonly ProductKind[] = ["vize", "vue-language-tools"];

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

export function emptyProductEvidence(): ProductEvidence {
  return {
    fresh: [],
    warm: [],
    freshDiagnostics: [],
    warmDiagnostics: [],
    warmStartupMs: [],
  };
}

export function emptyCustodyEvidence(): CustodyEvidence {
  return {
    qualifiedCandidates: { freshRounds: 0, warmRounds: 0 },
    freshCacheResets: [],
    primaryOperations: { fresh: [], warm: [] },
    oracleEquivalence: { fresh: [], warm: [] },
    warmupBoundaries: [],
  };
}

export function revisionReceiptSha256(
  scope: RevisionReceiptScope,
  evidence: RevisionEvidence,
): string {
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

export function throwSelectedFailure(selection: FailureSelection): void {
  if (selection.kind === "none") return;
  retainPrimaryCleanupFailures(selection);
  throw selection.error;
}

export async function closeProduct(product: DualLspProduct | undefined): Promise<unknown[]> {
  if (!product) return [];
  try {
    await product.shutdown();
    return [];
  } catch (shutdownError) {
    const failures: unknown[] = [shutdownError];
    try {
      await product.kill();
    } catch (killError) {
      failures.push(killError);
    }
    return failures;
  }
}

export function pairValue(
  pair: ReadonlyMap<ProductKind, QualifiedSample>,
  kind: ProductKind,
): QualifiedSample {
  const value = pair.get(kind);
  if (value == null) throw new Error(`qualified pair omitted ${kind}`);
  return value;
}

export function oracleEquivalenceReceipt(
  phase: "fresh" | "warm",
  sampleIndex: number,
  order: readonly ProductKind[],
  pair: ReadonlyMap<ProductKind, QualifiedSample>,
): Record<string, unknown> {
  const vize = pairValue(pair, "vize");
  const vue = pairValue(pair, "vue-language-tools");
  assert.equal(vize.diagnostics.v1.fingerprint, vue.diagnostics.v1.fingerprint);
  assert.equal(vize.diagnostics.v2.fingerprint, vue.diagnostics.v2.fingerprint);
  assert.deepEqual(vize.sample.oracleReceipt, vue.sample.oracleReceipt);
  const payload = {
    phase,
    sampleIndex,
    order,
    authoredUri: vize.sample.oracleReceipt.authoredUri,
    diagnostics: {
      v1Fingerprint: vize.diagnostics.v1.fingerprint,
      v2Fingerprint: vize.diagnostics.v2.fingerprint,
      provenVersion: 2,
    },
    queryOracle: vize.sample.oracleReceipt,
    products: {
      vize: {
        revisionReceiptSha256: vize.sample.revisionReceiptSha256,
        queryOracleSha256: vize.sample.oracleReceipt.sha256,
      },
      "vue-language-tools": {
        revisionReceiptSha256: vue.sample.revisionReceiptSha256,
        queryOracleSha256: vue.sample.oracleReceipt.sha256,
      },
    },
  };
  return { ...payload, sha256: sha256(JSON.stringify(payload)) };
}

export function primaryOperationPairReceipt(
  phase: "fresh" | "warm",
  sampleIndex: number | null,
  order: readonly ProductKind[],
  receipts: ReadonlyMap<ProductKind, PrimaryOperationReceipt>,
): Record<string, unknown> {
  const vize = receipts.get("vize");
  const vue = receipts.get("vue-language-tools");
  if (vize == null || vue == null)
    throw new Error(`${phase} primary operation receipt is incomplete`);
  assert.deepEqual(vize, vue, `${phase} primary method order/counts differed by product`);
  return {
    phase,
    sampleIndex,
    order,
    products: { vize, "vue-language-tools": vue },
  };
}
