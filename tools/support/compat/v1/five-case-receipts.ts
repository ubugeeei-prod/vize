import assert from "node:assert/strict";
import { createHash } from "node:crypto";

import {
  buildV1SupportMatrix,
  digestV1FiveCaseSelector,
  V1_CASES,
  V1_FIVE_CASE_PACKET_HASH_FIELDS,
  V1_FIVE_CASE_SELECTOR_SCHEMA,
  type V1FiveCaseSelector,
  type V1FiveCaseSelectorBody,
} from "./support-matrix.ts";

const SHA256 = /^[0-9a-f]{64}$/u;
const SOURCE_SHA = /^[0-9a-f]{40}$/u;
const RECEIPT_ID = /^[a-z0-9][a-z0-9._:/@+-]*$/u;

export const V1_FIVE_CASE_RECEIPT_SCHEMA = "vize.v1-five-case-receipt.v1" as const;
export const V1_FIVE_CASE_RECEIPT_BUNDLE_SCHEMA = "vize.v1-five-case-receipt-bundle.v1" as const;
export const V1_FIVE_CASE_SHARD_PLAN_SCHEMA = "vize.v1-five-case-shard-plan.v1" as const;
export const V1_FIVE_CASE_ADMISSION_SCHEMA = "vize.v1-five-case-admission.v1" as const;

export type V1FiveCaseMode = "pr" | "release";
export type V1FiveCaseReceiptOutcome = "passed" | "skipped" | "unsupported";
export type V1FiveCaseReceiptState = "complete" | "missing" | "skipped" | "stale" | "unsupported";

export interface V1FiveCaseRuntimeIdentity {
  name: string;
  version: string;
  target: string;
  os: string;
  architecture: string;
  libc: string;
  vue: string;
}

export interface V1FiveCasePacketIdentity {
  diagnosticsSha256: string;
  authoredRangesSha256: string;
  editsSha256: string;
  generatedOutputSha256: string;
  orderingSha256: string;
  exitBehaviorSha256: string;
}

export interface V1FiveCaseReceiptBody {
  schema: typeof V1_FIVE_CASE_RECEIPT_SCHEMA;
  id: string;
  selectorId: string;
  selectorSha256: string;
  combinationId: string;
  case: (typeof V1_CASES)[number];
  sourceSha: string;
  candidateSha: string;
  mode: V1FiveCaseMode;
  runId: number;
  runAttempt: number;
  job: string;
  shardIndex: number;
  shardTotal: number;
  outcome: V1FiveCaseReceiptOutcome;
  reason: string | null;
  fixtureSha256: string;
  dependencySha256: string;
  runtime: V1FiveCaseRuntimeIdentity;
  runtimeSha256: string;
  sourceBinarySha256: string;
  candidateBinarySha256: string;
  sourcePacket: V1FiveCasePacketIdentity;
  candidatePacket: V1FiveCasePacketIdentity;
}

export interface V1FiveCaseReceipt extends V1FiveCaseReceiptBody {
  receiptSha256: string;
}

export interface V1FiveCaseReceiptBundleBody {
  schema: typeof V1_FIVE_CASE_RECEIPT_BUNDLE_SCHEMA;
  id: string;
  sourceSha: string;
  candidateSha: string;
  mode: V1FiveCaseMode;
  runId: number;
  runAttempt: number;
  shardIndex: number;
  shardTotal: number;
  receipts: V1FiveCaseReceipt[];
}

export interface V1FiveCaseReceiptBundle extends V1FiveCaseReceiptBundleBody {
  bundleSha256: string;
}

type SupportMatrix = ReturnType<typeof buildV1SupportMatrix>;
type CaseSelector = SupportMatrix["caseSelectors"][number];
type AdmittedSelector = Omit<CaseSelector, "qualification" | "receiptIds"> & {
  qualification: "complete" | "gap";
  receiptState: V1FiveCaseReceiptState;
  receiptIds: string[];
  runIds: number[];
  gap: string | null;
};

function canonicalize(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonicalize);
  if (value != null && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .toSorted(([left], [right]) => left.localeCompare(right))
        .map(([key, entry]) => [key, canonicalize(entry)]),
    );
  return value;
}

export function digestV1FiveCaseValue(value: unknown): string {
  return createHash("sha256")
    .update(JSON.stringify(canonicalize(value)))
    .digest("hex");
}

export function sealV1FiveCaseReceipt(body: V1FiveCaseReceiptBody): V1FiveCaseReceipt {
  return { ...body, receiptSha256: digestV1FiveCaseValue(body) };
}

export function sealV1FiveCaseReceiptBundle(
  body: V1FiveCaseReceiptBundleBody,
): V1FiveCaseReceiptBundle {
  return { ...body, bundleSha256: digestV1FiveCaseValue(body) };
}

function assertSourceSha(name: string, value: string): void {
  assert.match(value, SOURCE_SHA, `invalid ${name}: ${value}`);
}

function assertHash(name: string, value: string, receiptId: string): void {
  assert.match(value, SHA256, `${receiptId}: invalid ${name}`);
}

function selectorBody(selector: V1FiveCaseSelector): V1FiveCaseSelectorBody {
  const {
    selectorSha256: _selectorSha256,
    qualification: _qualification,
    receiptIds: _ids,
    ...body
  } = selector;
  return body;
}

function compareText(left: string, right: string): number {
  return left.localeCompare(right);
}

function uniqueSorted(values: readonly string[]): string[] {
  return [...new Set(values)].toSorted(compareText);
}

function uniqueSortedNumbers(values: readonly number[]): number[] {
  return [...new Set(values)].toSorted((left, right) => left - right);
}

export function validateV1FiveCaseSelectorRegistry(matrix: SupportMatrix): void {
  assert.equal(
    matrix.caseSelectors.length,
    matrix.combinations.length * V1_CASES.length,
    "five-case selector count",
  );
  const combinations = new Map(
    matrix.combinations.map((combination) => [combination.id, combination]),
  );
  const selectorIds = new Set<string>();
  const owned = new Map<string, Set<string>>();
  for (const selector of matrix.caseSelectors) {
    assert.equal(selector.schema, V1_FIVE_CASE_SELECTOR_SCHEMA, `${selector.id}: schema`);
    assert.ok(!selectorIds.has(selector.id), `duplicate selector ID: ${selector.id}`);
    selectorIds.add(selector.id);
    const combination = combinations.get(selector.combinationId);
    assert.ok(combination, `${selector.id}: unknown combination ${selector.combinationId}`);
    assert.equal(selector.id, `${selector.combinationId}/${selector.case}`, selector.id);
    assert.equal(selector.owner, combination.owner, selector.id);
    assert.equal(selector.surface, combination.surface, selector.id);
    assert.equal(selector.product, combination.product, selector.id);
    assert.equal(selector.vue, combination.vue, selector.id);
    assert.equal(selector.backend, combination.backend, selector.id);
    assert.equal(selector.sourceKind, combination.sourceKind, selector.id);
    assert.deepEqual(
      selector.requiredQualificationSets,
      combination.requiredQualificationSets,
      selector.id,
    );
    assert.deepEqual(selector.evidenceCandidates, combination.evidence, selector.id);
    assert.deepEqual(selector.packetHashFields, V1_FIVE_CASE_PACKET_HASH_FIELDS, selector.id);
    assert.equal(
      selector.selectorSha256,
      digestV1FiveCaseSelector(selectorBody(selector)),
      `${selector.id}: selector digest`,
    );
    const cases = owned.get(selector.combinationId) ?? new Set<string>();
    assert.ok(!cases.has(selector.case), `${selector.combinationId}: duplicate ${selector.case}`);
    cases.add(selector.case);
    owned.set(selector.combinationId, cases);
  }
  assert.deepEqual(
    [...owned.keys()].toSorted(compareText),
    [...combinations.keys()].toSorted(compareText),
  );
  for (const [combinationId, cases] of owned)
    assert.deepEqual(
      [...cases].toSorted(compareText),
      [...V1_CASES].toSorted(compareText),
      combinationId,
    );
}

function assertRuntime(matrix: SupportMatrix, receipt: V1FiveCaseReceipt): void {
  const runtime = receipt.runtime;
  for (const name of ["name", "version", "target", "os", "architecture", "libc", "vue"] as const) {
    const value: unknown = runtime[name];
    assert.ok(
      typeof value === "string" && value.trim().length > 0,
      `${receipt.id}: runtime ${name}`,
    );
  }
  assert.ok(
    matrix.environments.hostRuntimeCombinations.some(
      (host) =>
        host.runtime === runtime.name &&
        host.version === runtime.version &&
        host.target === runtime.target &&
        host.os === runtime.os &&
        host.architecture === runtime.architecture &&
        host.libc === runtime.libc,
    ),
    `${receipt.id}: unknown host/runtime identity`,
  );
  assert.equal(
    receipt.runtimeSha256,
    digestV1FiveCaseValue(runtime),
    `${receipt.id}: runtime digest`,
  );
}

function selectorShardIndex(selector: CaseSelector, shardTotal: number): number {
  return Number.parseInt(selector.selectorSha256.slice(0, 12), 16) % shardTotal;
}

function runtimeSupportsSelector(
  selector: CaseSelector,
  runtime: V1FiveCaseRuntimeIdentity,
): boolean {
  return !(
    runtime.target.startsWith("wsl2-") &&
    selector.requiredQualificationSets.includes("shipped-editors")
  );
}

function assertPacket(
  packet: V1FiveCasePacketIdentity,
  packetName: "sourcePacket" | "candidatePacket",
  receiptId: string,
): void {
  assert.deepEqual(
    Object.keys(packet).toSorted(compareText),
    [...V1_FIVE_CASE_PACKET_HASH_FIELDS].toSorted(compareText),
    `${receiptId}: ${packetName}`,
  );
  for (const field of V1_FIVE_CASE_PACKET_HASH_FIELDS)
    assertHash(`${packetName}.${field}`, packet[field], receiptId);
}

function assertReceipt(matrix: SupportMatrix, receipt: V1FiveCaseReceipt): void {
  assert.equal(receipt.schema, V1_FIVE_CASE_RECEIPT_SCHEMA, `${receipt.id}: schema`);
  assert.match(receipt.id, RECEIPT_ID, `invalid receipt ID: ${receipt.id}`);
  const selector = matrix.caseSelectors.find(({ id }) => id === receipt.selectorId);
  assert.ok(selector, `${receipt.id}: unknown selector ${receipt.selectorId}`);
  assert.equal(receipt.combinationId, selector.combinationId, `${receipt.id}: combination`);
  assert.equal(receipt.case, selector.case, `${receipt.id}: case`);
  assertSourceSha("source SHA", receipt.sourceSha);
  assertSourceSha("candidate SHA", receipt.candidateSha);
  assert.ok(receipt.mode === "pr" || receipt.mode === "release", `${receipt.id}: mode`);
  assert.ok(Number.isSafeInteger(receipt.runId) && receipt.runId > 0, `${receipt.id}: runId`);
  assert.ok(
    Number.isSafeInteger(receipt.runAttempt) && receipt.runAttempt > 0,
    `${receipt.id}: runAttempt`,
  );
  assert.ok(receipt.job.trim().length > 0, `${receipt.id}: job`);
  assert.ok(
    Number.isSafeInteger(receipt.shardTotal) && receipt.shardTotal > 0,
    `${receipt.id}: shardTotal`,
  );
  assert.ok(
    Number.isSafeInteger(receipt.shardIndex) &&
      receipt.shardIndex >= 0 &&
      receipt.shardIndex < receipt.shardTotal,
    `${receipt.id}: shardIndex`,
  );
  assert.equal(
    selectorShardIndex(selector, receipt.shardTotal),
    receipt.shardIndex,
    `${receipt.id}: selector shard ownership`,
  );
  assert.ok(
    receipt.outcome === "passed" ||
      receipt.outcome === "skipped" ||
      receipt.outcome === "unsupported",
    `${receipt.id}: outcome`,
  );
  if (receipt.outcome === "passed") assert.equal(receipt.reason, null, `${receipt.id}: reason`);
  else assert.ok(receipt.reason?.trim(), `${receipt.id}: ${receipt.outcome} requires a reason`);
  assertHash("selectorSha256", receipt.selectorSha256, receipt.id);
  assertHash("fixtureSha256", receipt.fixtureSha256, receipt.id);
  assertHash("dependencySha256", receipt.dependencySha256, receipt.id);
  assertHash("runtimeSha256", receipt.runtimeSha256, receipt.id);
  assertHash("sourceBinarySha256", receipt.sourceBinarySha256, receipt.id);
  assertHash("candidateBinarySha256", receipt.candidateBinarySha256, receipt.id);
  assertPacket(receipt.sourcePacket, "sourcePacket", receipt.id);
  assertPacket(receipt.candidatePacket, "candidatePacket", receipt.id);
  if (receipt.outcome === "passed")
    assert.deepEqual(
      receipt.candidatePacket,
      receipt.sourcePacket,
      `${receipt.id}: passed receipt changes behavior`,
    );
  assertRuntime(matrix, receipt);
  const { receiptSha256, ...body } = receipt;
  assertHash("receiptSha256", receiptSha256, receipt.id);
  assert.equal(receiptSha256, digestV1FiveCaseValue(body), `${receipt.id}: receipt digest`);
}

function assertBundle(matrix: SupportMatrix, bundle: V1FiveCaseReceiptBundle): void {
  assert.equal(bundle.schema, V1_FIVE_CASE_RECEIPT_BUNDLE_SCHEMA, `${bundle.id}: schema`);
  assert.match(bundle.id, RECEIPT_ID, `invalid bundle ID: ${bundle.id}`);
  assertSourceSha("bundle source SHA", bundle.sourceSha);
  assertSourceSha("bundle candidate SHA", bundle.candidateSha);
  assert.ok(bundle.mode === "pr" || bundle.mode === "release", `${bundle.id}: mode`);
  assert.ok(Number.isSafeInteger(bundle.runId) && bundle.runId > 0, `${bundle.id}: runId`);
  assert.ok(
    Number.isSafeInteger(bundle.runAttempt) && bundle.runAttempt > 0,
    `${bundle.id}: runAttempt`,
  );
  assert.ok(
    Number.isSafeInteger(bundle.shardTotal) && bundle.shardTotal > 0,
    `${bundle.id}: shardTotal`,
  );
  assert.ok(
    Number.isSafeInteger(bundle.shardIndex) &&
      bundle.shardIndex >= 0 &&
      bundle.shardIndex < bundle.shardTotal,
    `${bundle.id}: shardIndex`,
  );
  const receiptIds = new Set<string>();
  for (const receipt of bundle.receipts) {
    assertReceipt(matrix, receipt);
    assert.ok(!receiptIds.has(receipt.id), `${bundle.id}: duplicate receipt ID ${receipt.id}`);
    receiptIds.add(receipt.id);
    assert.equal(receipt.sourceSha, bundle.sourceSha, `${receipt.id}: bundle source`);
    assert.equal(receipt.candidateSha, bundle.candidateSha, `${receipt.id}: bundle candidate`);
    assert.equal(receipt.mode, bundle.mode, `${receipt.id}: bundle mode`);
    assert.equal(receipt.runId, bundle.runId, `${receipt.id}: bundle run`);
    assert.equal(receipt.runAttempt, bundle.runAttempt, `${receipt.id}: bundle attempt`);
    assert.equal(receipt.shardIndex, bundle.shardIndex, `${receipt.id}: bundle shard index`);
    assert.equal(receipt.shardTotal, bundle.shardTotal, `${receipt.id}: bundle shard total`);
  }
  const { bundleSha256, ...body } = bundle;
  assertHash("bundleSha256", bundleSha256, bundle.id);
  assert.equal(bundleSha256, digestV1FiveCaseValue(body), `${bundle.id}: bundle digest`);
}

export function validateV1FiveCaseReceiptBundles(
  matrix: SupportMatrix,
  bundles: readonly V1FiveCaseReceiptBundle[],
): void {
  validateV1FiveCaseSelectorRegistry(matrix);
  const bundleIds = new Set<string>();
  const receiptIds = new Set<string>();
  for (const bundle of bundles) {
    assertBundle(matrix, bundle);
    assert.ok(!bundleIds.has(bundle.id), `duplicate bundle ID: ${bundle.id}`);
    bundleIds.add(bundle.id);
    for (const receipt of bundle.receipts) {
      assert.ok(!receiptIds.has(receipt.id), `duplicate receipt ID: ${receipt.id}`);
      receiptIds.add(receipt.id);
    }
  }
}

export function buildV1FiveCaseShardPlan(
  matrix: SupportMatrix,
  mode: V1FiveCaseMode,
  shardTotal: number,
) {
  validateV1FiveCaseSelectorRegistry(matrix);
  assert.ok(Number.isSafeInteger(shardTotal) && shardTotal > 0, "shardTotal");
  const shards = Array.from({ length: shardTotal }, (_, index) => ({
    id: `${mode}-${index + 1}-of-${shardTotal}`,
    index,
    total: shardTotal,
    selectorIds: [] as string[],
    selectorInventorySha256: "",
  }));
  for (const selector of matrix.caseSelectors) {
    const index = selectorShardIndex(selector, shardTotal);
    shards[index]!.selectorIds.push(selector.id);
  }
  for (const shard of shards) {
    shard.selectorIds.sort(compareText);
    shard.selectorInventorySha256 = digestV1FiveCaseValue(shard.selectorIds);
  }
  return {
    schema: V1_FIVE_CASE_SHARD_PLAN_SCHEMA,
    mode,
    registrySha256: digestV1FiveCaseValue(
      matrix.caseSelectors.map(({ id, selectorSha256 }) => [id, selectorSha256]),
    ),
    counts: { selectors: matrix.caseSelectors.length, shards: shardTotal },
    shards,
  };
}

function admitSelector(
  matrix: SupportMatrix,
  selector: CaseSelector,
  identity: { sourceSha: string; candidateSha: string; mode: V1FiveCaseMode },
  receipts: readonly V1FiveCaseReceipt[],
): AdmittedSelector {
  const owned = receipts.filter(({ selectorId }) => selectorId === selector.id);
  const exact = owned.filter(
    (receipt) =>
      receipt.sourceSha === identity.sourceSha &&
      receipt.candidateSha === identity.candidateSha &&
      receipt.mode === identity.mode &&
      receipt.selectorSha256 === selector.selectorSha256 &&
      receipt.runtime.vue === selector.vue &&
      runtimeSupportsSelector(selector, receipt.runtime),
  );
  const passed = exact.filter(({ outcome }) => outcome === "passed");
  const skipped = exact.some(({ outcome }) => outcome === "skipped");
  const unsupported = exact.some(({ outcome }) => outcome === "unsupported");
  const state: V1FiveCaseReceiptState =
    passed.length > 0
      ? "complete"
      : unsupported
        ? "unsupported"
        : skipped
          ? "skipped"
          : owned.length > 0
            ? "stale"
            : "missing";
  const gap =
    state === "complete"
      ? null
      : state === "unsupported"
        ? "The exact selector reported an unsupported environment."
        : state === "skipped"
          ? "The exact selector was skipped."
          : state === "stale"
            ? "Only stale source, candidate, selector, mode, Vue or host/product identities exist."
            : "No receipt owns this selector.";
  return {
    ...selector,
    qualification: state === "complete" ? "complete" : "gap",
    receiptState: state,
    receiptIds: uniqueSorted(passed.map(({ id }) => id)),
    runIds: uniqueSortedNumbers(passed.map(({ runId }) => runId)),
    gap,
  };
}

export function admitV1FiveCaseReceipts(
  matrix: SupportMatrix,
  identity: { sourceSha: string; candidateSha: string; mode: V1FiveCaseMode },
  bundles: readonly V1FiveCaseReceiptBundle[],
) {
  assertSourceSha("source SHA", identity.sourceSha);
  assertSourceSha("candidate SHA", identity.candidateSha);
  validateV1FiveCaseReceiptBundles(matrix, bundles);
  const receipts = bundles.flatMap(({ receipts: entries }) => entries);
  const caseSelectors = matrix.caseSelectors.map((selector) =>
    admitSelector(matrix, selector, identity, receipts),
  );
  const caseComplete = caseSelectors.every(({ qualification }) => qualification === "complete");
  const knownGaps = matrix.knownGaps.map((gap) =>
    gap.id === "case-packet-join" && caseComplete
      ? {
          ...gap,
          state: "complete",
          summary: `All ${caseSelectors.length} five-case selectors are admitted from exact receipts.`,
        }
      : gap,
  );
  const blockingGaps = knownGaps.filter(({ state }) => state === "blocking").length;
  const completeRequiredRows = matrix.requiredRows.filter(
    ({ qualification }) => qualification === "complete",
  ).length;
  const admittedCaseSelectors = caseSelectors.filter(
    ({ qualification }) => qualification === "complete",
  ).length;
  const registrySha256 = digestV1FiveCaseValue(
    matrix.caseSelectors.map(({ id, selectorSha256 }) => [id, selectorSha256]),
  );
  return {
    ...matrix,
    fiveCaseAdmission: {
      schema: V1_FIVE_CASE_ADMISSION_SCHEMA,
      ...identity,
      registrySha256,
      bundleSha256: digestV1FiveCaseValue(
        bundles
          .map(({ id, bundleSha256 }) => [id, bundleSha256])
          .toSorted(([left], [right]) => left.localeCompare(right)),
      ),
    },
    caseSelectors,
    knownGaps,
    releaseReady:
      admittedCaseSelectors === caseSelectors.length &&
      completeRequiredRows === matrix.requiredRows.length &&
      blockingGaps === 0,
    counts: {
      ...matrix.counts,
      admittedCaseSelectors,
      missingCaseSelectors: caseSelectors.filter(({ receiptState }) => receiptState === "missing")
        .length,
      skippedCaseSelectors: caseSelectors.filter(({ receiptState }) => receiptState === "skipped")
        .length,
      staleCaseSelectors: caseSelectors.filter(({ receiptState }) => receiptState === "stale")
        .length,
      unsupportedCaseSelectors: caseSelectors.filter(
        ({ receiptState }) => receiptState === "unsupported",
      ).length,
      blockingGaps,
    },
  };
}
