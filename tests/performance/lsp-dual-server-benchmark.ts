import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { inspect } from "node:util";

import { repoRoot } from "../_helpers/realworld-patch.ts";
import {
  BUILD_RECIPE,
  PROFILE_BUILD_RECIPE,
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../differential/build-receipt.ts";
import {
  assertOwnedWorkspaceCachesAbsent,
  createDualLspFixture,
  resetOwnedWorkspaceCaches,
  type OwnedCachePathReceipt,
} from "./support/dual-lsp-fixture.ts";
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
  primaryOperationReceipt,
  productOrder,
  stableOperationTranscript,
  type PrimaryOperationReceipt,
  type ProductKind,
} from "./support/dual-lsp-product.ts";
import {
  assertRevisionReceiptReachability,
  cleanupFailuresFor,
  distribution,
  retainPrimaryCleanupFailures,
  selectAuthoritativeFailure,
  summarizeScheduleReceipt,
  summarizeDiagnostics,
  summarizeSamples,
  writeDualLspReport,
  type FailureSelection,
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

type QualifiedSample = { diagnostics: RevisionEvidence; sample: MeasuredSample };

type FreshCacheResetReceipt = {
  sampleIndex: number;
  orderIndex: number;
  product: ProductKind;
  nextProduct: ProductKind | null;
  paths: OwnedCachePathReceipt[];
  verificationStage: "before-next-product" | "before-fixture-dispose" | null;
  absenceVerified: boolean;
};

type CustodyEvidence = {
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

function throwSelectedFailure(selection: FailureSelection): void {
  if (selection.kind === "none") return;
  retainPrimaryCleanupFailures(selection);
  throw selection.error;
}

async function closeProduct(product: DualLspProduct | undefined): Promise<unknown[]> {
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

function pairValue(
  pair: ReadonlyMap<ProductKind, QualifiedSample>,
  kind: ProductKind,
): QualifiedSample {
  const value = pair.get(kind);
  if (value == null) throw new Error(`qualified pair omitted ${kind}`);
  return value;
}

function oracleEquivalenceReceipt(
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

function primaryOperationPairReceipt(
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

async function collectFresh(
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

async function collectWarm(
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
        const revisionReceiptSha256 = sessionRevisionReceipts.get(kind);
        if (product == null || revisionReceiptSha256 == null) {
          throw new Error(`${kind} warm session lost its product or revision receipt`);
        }
        const operation = await runQualifiedOperationPacket(product, fixture);
        const sample: MeasuredSample = { ...operation, revisionReceiptSha256 };
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
      const revisionReceiptSha256 = sessionRevisionReceipts.get(kind);
      if (diagnostics == null || revisionReceiptSha256 == null) {
        throw new Error(`${kind} warmup boundary lost its diagnostic receipt`);
      }
      const samples = candidateSamples[kind];
      assert.ok(sessionStartupMs.has(kind));
      assert.equal(samples.length, count);
      assert.ok(samples.every((sample) => sample.revisionReceiptSha256 === revisionReceiptSha256));
      const payload = {
        product: kind,
        sessionIndex: 0,
        initializedSessions: 1,
        eventOrder: ["initialize", "qualify-v1", "qualify-v2", "measure-packets"],
        diagnosticQualification: {
          v1Fingerprint: diagnostics.v1.fingerprint,
          v2Fingerprint: diagnostics.v2.fingerprint,
          provenVersion: diagnostics.v2.provenVersion,
          revisionReceiptSha256,
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

async function main(): Promise<void> {
  const config = options();
  const evidence: Record<ProductKind, ProductEvidence> = {
    vize: emptyProductEvidence(),
    "vue-language-tools": emptyProductEvidence(),
  };
  const transcriptHashes: string[] = [];
  const freshSchedule: SampleOrderReceipt[] = [];
  const warmSchedule: SampleOrderReceipt[] = [];
  const custody: CustodyEvidence = {
    qualifiedCandidates: { freshRounds: 0, warmRounds: 0 },
    freshCacheResets: [],
    primaryOperations: { fresh: [], warm: [] },
    oracleEquivalence: { fresh: [], warm: [] },
    warmupBoundaries: [],
  };
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
