import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import {
  admitV1FiveCaseReceipts,
  buildV1FiveCaseShardPlan,
  digestV1FiveCaseValue,
  sealV1FiveCaseReceipt,
  sealV1FiveCaseReceiptBundle,
  validateV1FiveCaseReceiptBundles,
  validateV1FiveCaseSelectorRegistry,
  V1_FIVE_CASE_RECEIPT_BUNDLE_SCHEMA,
  V1_FIVE_CASE_RECEIPT_SCHEMA,
  type V1FiveCaseReceipt,
  type V1FiveCaseReceiptBody,
  type V1FiveCaseReceiptBundle,
  type V1FiveCaseReceiptOutcome,
  type V1FiveCasePacketIdentity,
} from "../../tools/support/compat/v1/five-case-receipts.ts";
import {
  buildV1SupportMatrix,
  V1_CASES,
  V1_FIVE_CASE_PACKET_HASH_FIELDS,
} from "../../tools/support/compat/v1/support-matrix.ts";

const root = path.resolve(import.meta.dirname, "../..");
const matrix = buildV1SupportMatrix(root);
const sourceSha = "1".repeat(40);
const candidateSha = "2".repeat(40);
const host = matrix.environments.hostRuntimeCombinations[0]!;

function hash(value: string): string {
  return digestV1FiveCaseValue(value);
}

function packet(id: string): V1FiveCasePacketIdentity {
  return {
    diagnosticsSha256: hash(`${id}:diagnostics`),
    authoredRangesSha256: hash(`${id}:ranges`),
    editsSha256: hash(`${id}:edits`),
    generatedOutputSha256: hash(`${id}:output`),
    orderingSha256: hash(`${id}:ordering`),
    exitBehaviorSha256: hash(`${id}:exit`),
  };
}

function receipt(
  selector: (typeof matrix.caseSelectors)[number],
  overrides: Partial<V1FiveCaseReceiptBody> = {},
): V1FiveCaseReceipt {
  const runtime = overrides.runtime ?? {
    name: host.runtime,
    version: host.version,
    target: host.target,
    os: host.os,
    architecture: host.architecture,
    libc: host.libc,
    vue: selector.vue,
  };
  const id = overrides.id ?? `receipt:${selector.id}`;
  return sealV1FiveCaseReceipt({
    schema: V1_FIVE_CASE_RECEIPT_SCHEMA,
    id,
    selectorId: selector.id,
    selectorSha256: selector.selectorSha256,
    combinationId: selector.combinationId,
    case: selector.case,
    sourceSha,
    candidateSha,
    mode: "release",
    runId: 44_000_000_000,
    runAttempt: 1,
    job: `qualify-${selector.surface}`,
    shardIndex: 0,
    shardTotal: 1,
    outcome: "passed",
    reason: null,
    fixtureSha256: hash(`${id}:fixture`),
    dependencySha256: hash(`${id}:dependency`),
    runtime,
    runtimeSha256: digestV1FiveCaseValue(runtime),
    sourceBinarySha256: hash(`${id}:source-binary`),
    candidateBinarySha256: hash(`${id}:candidate-binary`),
    sourcePacket: packet(id),
    candidatePacket: packet(id),
    ...overrides,
  });
}

function receiptsExcept(selectorId?: string): V1FiveCaseReceipt[] {
  return matrix.caseSelectors
    .filter(({ id }) => id !== selectorId)
    .map((selector) => receipt(selector));
}

function bundle(
  id: string,
  receipts: V1FiveCaseReceipt[],
  identity: {
    sourceSha?: string;
    candidateSha?: string;
    mode?: "pr" | "release";
    runId?: number;
    runAttempt?: number;
    shardIndex?: number;
    shardTotal?: number;
  } = {},
): V1FiveCaseReceiptBundle {
  const first = receipts[0];
  return sealV1FiveCaseReceiptBundle({
    schema: V1_FIVE_CASE_RECEIPT_BUNDLE_SCHEMA,
    id,
    sourceSha: identity.sourceSha ?? first?.sourceSha ?? sourceSha,
    candidateSha: identity.candidateSha ?? first?.candidateSha ?? candidateSha,
    mode: identity.mode ?? first?.mode ?? "release",
    runId: identity.runId ?? first?.runId ?? 44_000_000_000,
    runAttempt: identity.runAttempt ?? first?.runAttempt ?? 1,
    shardIndex: identity.shardIndex ?? first?.shardIndex ?? 0,
    shardTotal: identity.shardTotal ?? first?.shardTotal ?? 1,
    receipts,
  });
}

function completeBundle(): V1FiveCaseReceiptBundle {
  return bundle("bundle:complete", receiptsExcept());
}

function withOutcome(
  selector: (typeof matrix.caseSelectors)[number],
  outcome: V1FiveCaseReceiptOutcome,
): V1FiveCaseReceipt {
  return receipt(selector, {
    outcome,
    reason: outcome === "passed" ? null : `${outcome} by the exact producer`,
  });
}

void test("the generated registry owns every combination and five-case selector exactly once", () => {
  validateV1FiveCaseSelectorRegistry(matrix);
  assert.equal(matrix.combinations.length, 88);
  assert.equal(matrix.caseSelectors.length, 440);
  assert.equal(matrix.counts.caseSelectors, 440);
  assert.equal(matrix.counts.caseObligations, 440);
  const byCombination = new Map<string, string[]>();
  for (const selector of matrix.caseSelectors) {
    assert.match(selector.selectorSha256, /^[0-9a-f]{64}$/u, selector.id);
    assert.deepEqual(selector.packetHashFields, V1_FIVE_CASE_PACKET_HASH_FIELDS, selector.id);
    assert.equal(selector.qualification, "unbound", selector.id);
    assert.deepEqual(selector.receiptIds, [], selector.id);
    byCombination.set(selector.combinationId, [
      ...(byCombination.get(selector.combinationId) ?? []),
      selector.case,
    ]);
  }
  assert.equal(byCombination.size, matrix.combinations.length);
  for (const [combinationId, cases] of byCombination)
    assert.deepEqual(cases.toSorted(), [...V1_CASES].toSorted(), combinationId);
});

void test("missing, duplicate and edited selectors fail the registry closed", () => {
  const missing = structuredClone(matrix);
  missing.caseSelectors.pop();
  assert.throws(() => validateV1FiveCaseSelectorRegistry(missing), /selector count/u);

  const duplicate = structuredClone(matrix);
  duplicate.caseSelectors.splice(-1, 1, structuredClone(duplicate.caseSelectors[0]!));
  assert.throws(
    () => validateV1FiveCaseSelectorRegistry(duplicate),
    /duplicate selector ID|duplicate positive/u,
  );

  const edited = structuredClone(matrix);
  edited.caseSelectors[0]!.owner = "testing/foreign-owner";
  assert.throws(() => validateV1FiveCaseSelectorRegistry(edited), /owner|selector digest/u);
});

void test("PR and release plans partition the same stable selector IDs exactly once", () => {
  const pr = buildV1FiveCaseShardPlan(matrix, "pr", 8);
  const release = buildV1FiveCaseShardPlan(matrix, "release", 32);
  const selectorIds = matrix.caseSelectors.map(({ id }) => id).toSorted();
  assert.equal(pr.registrySha256, release.registrySha256);
  assert.deepEqual(pr.shards.flatMap(({ selectorIds: ids }) => ids).toSorted(), selectorIds);
  assert.deepEqual(release.shards.flatMap(({ selectorIds: ids }) => ids).toSorted(), selectorIds);
  assert.equal(new Set(pr.shards.flatMap(({ selectorIds: ids }) => ids)).size, 440);
  assert.equal(new Set(release.shards.flatMap(({ selectorIds: ids }) => ids)).size, 440);
  for (const shard of [...pr.shards, ...release.shards]) {
    assert.ok(shard.selectorIds.length > 0, shard.id);
    assert.equal(shard.selectorInventorySha256, digestV1FiveCaseValue(shard.selectorIds), shard.id);
  }
});

void test("one exact candidate admits all 440 selectors without granting unrelated gaps", () => {
  const admitted = admitV1FiveCaseReceipts(matrix, { sourceSha, candidateSha, mode: "release" }, [
    completeBundle(),
  ]);
  assert.equal(admitted.counts.admittedCaseSelectors, 440);
  assert.equal(admitted.counts.missingCaseSelectors, 0);
  assert.equal(admitted.counts.skippedCaseSelectors, 0);
  assert.equal(admitted.counts.staleCaseSelectors, 0);
  assert.equal(admitted.counts.unsupportedCaseSelectors, 0);
  assert.equal(
    admitted.caseSelectors.every(({ receiptState }) => receiptState === "complete"),
    true,
  );
  assert.equal(admitted.knownGaps.find(({ id }) => id === "case-packet-join")?.state, "complete");
  assert.equal(admitted.counts.blockingGaps, matrix.counts.blockingGaps - 1);
  assert.equal(admitted.releaseReady, false);
  assert.match(admitted.fiveCaseAdmission.registrySha256, /^[0-9a-f]{64}$/u);
  assert.match(admitted.fiveCaseAdmission.bundleSha256, /^[0-9a-f]{64}$/u);
});

void test("missing, skipped, stale and unsupported selector evidence stay distinct", () => {
  const selected = matrix.caseSelectors[0]!;
  const exactIdentity = { sourceSha, candidateSha, mode: "release" as const };
  const base = receiptsExcept(selected.id);

  const missing = admitV1FiveCaseReceipts(matrix, exactIdentity, [bundle("bundle:missing", base)]);
  assert.equal(missing.caseSelectors.find(({ id }) => id === selected.id)?.receiptState, "missing");

  for (const outcome of ["skipped", "unsupported"] as const) {
    const admitted = admitV1FiveCaseReceipts(matrix, exactIdentity, [
      bundle(`bundle:${outcome}`, [...base, withOutcome(selected, outcome)]),
    ]);
    assert.equal(
      admitted.caseSelectors.find(({ id }) => id === selected.id)?.receiptState,
      outcome,
    );
  }

  const staleSource = "3".repeat(40);
  const staleReceipt = receipt(selected, {
    id: `stale:${selected.id}`,
    sourceSha: staleSource,
  });
  const stale = admitV1FiveCaseReceipts(matrix, exactIdentity, [
    bundle("bundle:exact-rest", base),
    bundle("bundle:stale", [staleReceipt], { sourceSha: staleSource }),
  ]);
  assert.equal(stale.caseSelectors.find(({ id }) => id === selected.id)?.receiptState, "stale");
});

void test("receipt, runtime and bundle identities reject partial or duplicate evidence", () => {
  const selected = matrix.caseSelectors[0]!;
  const invalidHash = receipt(selected, {
    candidatePacket: { ...packet(`receipt:${selected.id}`), diagnosticsSha256: "missing" },
  });
  assert.throws(
    () => validateV1FiveCaseReceiptBundles(matrix, [bundle("bundle:invalid-hash", [invalidHash])]),
    /invalid candidatePacket\.diagnosticsSha256/u,
  );

  const invalidRuntime = receipt(selected, { runtimeSha256: "0".repeat(64) });
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [
        bundle("bundle:invalid-runtime", [invalidRuntime]),
      ]),
    /runtime digest/u,
  );

  const missingOs = receipt(selected, {
    runtime: {
      name: host.runtime,
      version: host.version,
      target: host.target,
      architecture: host.architecture,
      libc: host.libc,
      vue: selected.vue,
    } as V1FiveCaseReceiptBody["runtime"],
  });
  assert.throws(
    () => validateV1FiveCaseReceiptBundles(matrix, [bundle("bundle:missing-os", [missingOs])]),
    /runtime os/u,
  );

  const conflictingPlatform = receipt(selected, {
    runtime: {
      name: host.runtime,
      version: host.version,
      target: host.target,
      os: host.os === "linux" ? "darwin" : "linux",
      architecture: host.architecture,
      libc: host.libc,
      vue: selected.vue,
    },
  });
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [
        bundle("bundle:conflicting-platform", [conflictingPlatform]),
      ]),
    /unknown host\/runtime identity/u,
  );

  const shared = receipt(selected);
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [
        bundle("bundle:one", [shared]),
        bundle("bundle:two", [shared]),
      ]),
    /duplicate receipt ID/u,
  );

  const tampered = completeBundle();
  tampered.bundleSha256 = "0".repeat(64);
  assert.throws(() => validateV1FiveCaseReceiptBundles(matrix, [tampered]), /bundle digest/u);

  const changedPacket = receipt(selected, {
    candidatePacket: {
      ...packet(`receipt:${selected.id}`),
      diagnosticsSha256: hash("changed diagnostics"),
    },
  });
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [bundle("bundle:changed-packet", [changedPacket])]),
    /passed receipt changes behavior/u,
  );

  const shardPlan = buildV1FiveCaseShardPlan(matrix, "release", 8);
  const expectedShard = shardPlan.shards.find(({ selectorIds }) =>
    selectorIds.includes(selected.id),
  )!.index;
  const wrongShard = receipt(selected, {
    shardIndex: (expectedShard + 1) % 8,
    shardTotal: 8,
  });
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [
        bundle("bundle:wrong-shard", [wrongShard], {
          shardIndex: wrongShard.shardIndex,
          shardTotal: wrongShard.shardTotal,
        }),
      ]),
    /selector shard ownership/u,
  );
});

void test("selector, candidate and Vue identity changes cannot receive current credit", () => {
  const selected = matrix.caseSelectors[0]!;
  const rest = receiptsExcept(selected.id);
  const staleSelector = receipt(selected, {
    id: `stale-selector:${selected.id}`,
    selectorSha256: "9".repeat(64),
  });
  const admitted = admitV1FiveCaseReceipts(matrix, { sourceSha, candidateSha, mode: "release" }, [
    bundle("bundle:stale-selector", [...rest, staleSelector]),
  ]);
  assert.equal(admitted.caseSelectors.find(({ id }) => id === selected.id)?.receiptState, "stale");

  const wrongRuntime = receipt(selected, {
    runtime: {
      name: "bun",
      version: matrix.versions.bun,
      target: "not-a-shipped-target",
      os: "linux",
      architecture: "x64",
      libc: "glibc",
      vue: selected.vue,
    },
  });
  assert.throws(
    () =>
      validateV1FiveCaseReceiptBundles(matrix, [bundle("bundle:wrong-runtime", [wrongRuntime])]),
    /unknown host\/runtime identity/u,
  );

  const editorSelector = matrix.caseSelectors.find(({ product }) => product === "shipped-editors")!;
  const wsl = matrix.environments.hostRuntimeCombinations.find(({ target }) =>
    target.startsWith("wsl2-"),
  )!;
  const wslEditor = receipt(editorSelector, {
    runtime: {
      name: wsl.runtime,
      version: wsl.version,
      target: wsl.target,
      os: wsl.os,
      architecture: wsl.architecture,
      libc: wsl.libc,
      vue: editorSelector.vue,
    },
  });
  const unsupportedHost = admitV1FiveCaseReceipts(
    matrix,
    { sourceSha, candidateSha, mode: "release" },
    [bundle("bundle:wsl-editor", [wslEditor])],
  );
  assert.equal(
    unsupportedHost.caseSelectors.find(({ id }) => id === editorSelector.id)?.receiptState,
    "stale",
  );
});
