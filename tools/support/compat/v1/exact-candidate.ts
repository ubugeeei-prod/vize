import assert from "node:assert/strict";
import { createHash } from "node:crypto";

import { buildV1SupportMatrix } from "./support-matrix.ts";

const SHA256 = /^[0-9a-f]{64}$/u;
const SOURCE_SHA = /^[0-9a-f]{40}$/u;
const RECEIPT_ID = /^[a-z0-9][a-z0-9._:/@+-]*$/u;

export const V1_EXACT_CANDIDATE_RECEIPT_SCHEMA =
  "vize.v1-exact-candidate-receipt.v1" as const;
export const V1_EXACT_CANDIDATE_BUNDLE_SCHEMA =
  "vize.v1-exact-candidate-receipt-bundle.v1" as const;
export const V1_EXACT_CANDIDATE_HOST_ADMISSION_SCHEMA =
  "vize.v1-exact-candidate-host-admission.v1" as const;

export type V1ReceiptClassification =
  | "automated-positive"
  | "automated-negative"
  | "manual"
  | "qualification-set";
export type V1ReceiptOutcome = "passed" | "skipped" | "unsupported";

export interface V1ExactCandidateReceipt {
  schema: typeof V1_EXACT_CANDIDATE_RECEIPT_SCHEMA;
  id: string;
  classification: V1ReceiptClassification;
  outcome: V1ReceiptOutcome;
  row: string | null;
  qualificationSet: string | null;
  environment: string | null;
  source: string;
  run: number;
  runAttempt: number;
  job: string;
  fixtureSha256: string;
  dependencySha256: string;
  binarySha256: string;
  runtime: {
    name: string;
    version: string;
    target: string;
  };
  manualReason: string | null;
}

export interface V1ExactCandidateReceiptBundle {
  schema: typeof V1_EXACT_CANDIDATE_BUNDLE_SCHEMA;
  receipts: V1ExactCandidateReceipt[];
}

type SupportMatrix = ReturnType<typeof buildV1SupportMatrix>;
type RequiredRow = SupportMatrix["requiredRows"][number];
type AdmittedRow = Omit<
  RequiredRow,
  | "candidateSource"
  | "candidateRun"
  | "evidence"
  | "positive"
  | "negative"
  | "manual"
  | "qualification"
  | "gap"
> & {
  candidateSource: string;
  candidateRun: number | null;
  candidateRuns: number[];
  evidence: string[];
  positive: string[];
  negative: string[];
  manual: string[];
  qualification: "complete" | "gap";
  gap: string | null;
  receiptState: "complete" | "missing" | "skipped" | "stale-source" | "unsupported";
};

export interface V1ExactCandidateHostPlan {
  id: string;
  runtime: string;
  runtimeVersion: string;
  target: string;
  rowIds: string[];
}

export interface V1ExactCandidateHostAdmission {
  schema: typeof V1_EXACT_CANDIDATE_HOST_ADMISSION_SCHEMA;
  candidateSource: string;
  host: V1ExactCandidateHostPlan;
  rows: AdmittedRow[];
}

function hostId(row: RequiredRow): string {
  return `${row.runtime}-${row.runtimeVersion}-${row.host}`;
}

function compareText(left: string, right: string): number {
  return left.localeCompare(right);
}

function assertExactSource(source: string): void {
  assert.match(source, SOURCE_SHA, `invalid exact candidate source: ${source}`);
}

function assertHash(name: string, value: string, receiptId: string): void {
  assert.match(value, SHA256, `${receiptId}: invalid ${name}`);
}

function assertReceipt(receipt: V1ExactCandidateReceipt): void {
  assert.equal(receipt.schema, V1_EXACT_CANDIDATE_RECEIPT_SCHEMA, `${receipt.id}: schema`);
  assert.match(receipt.id, RECEIPT_ID, `invalid receipt ID: ${receipt.id}`);
  assertExactSource(receipt.source);
  assert.ok(Number.isSafeInteger(receipt.run) && receipt.run > 0, `${receipt.id}: run`);
  assert.ok(
    Number.isSafeInteger(receipt.runAttempt) && receipt.runAttempt > 0,
    `${receipt.id}: runAttempt`,
  );
  assert.ok(receipt.job.trim().length > 0, `${receipt.id}: job`);
  assertHash("fixtureSha256", receipt.fixtureSha256, receipt.id);
  assertHash("dependencySha256", receipt.dependencySha256, receipt.id);
  assertHash("binarySha256", receipt.binarySha256, receipt.id);
  assert.ok(receipt.runtime.name.trim().length > 0, `${receipt.id}: runtime name`);
  assert.ok(receipt.runtime.version.trim().length > 0, `${receipt.id}: runtime version`);
  assert.ok(receipt.runtime.target.trim().length > 0, `${receipt.id}: runtime target`);

  if (receipt.classification === "qualification-set") {
    assert.equal(receipt.row, null, `${receipt.id}: qualification receipt cannot own a row`);
    assert.ok(receipt.qualificationSet, `${receipt.id}: qualificationSet`);
    assert.ok(receipt.environment, `${receipt.id}: environment`);
    assert.equal(receipt.manualReason, null, `${receipt.id}: manualReason`);
  } else {
    assert.ok(receipt.row, `${receipt.id}: row`);
    assert.equal(receipt.qualificationSet, null, `${receipt.id}: qualificationSet`);
    assert.equal(receipt.environment, null, `${receipt.id}: environment`);
    if (receipt.classification === "manual")
      assert.ok(receipt.manualReason?.trim(), `${receipt.id}: manual evidence requires a reason`);
    else assert.equal(receipt.manualReason, null, `${receipt.id}: manualReason`);
  }
}

export function validateV1ExactCandidateReceiptBundle(
  matrix: SupportMatrix,
  bundle: V1ExactCandidateReceiptBundle,
): void {
  assert.equal(bundle.schema, V1_EXACT_CANDIDATE_BUNDLE_SCHEMA);
  const rowIds = new Set(matrix.requiredRows.map(({ id }) => id));
  const receiptIds = new Set<string>();
  for (const receipt of bundle.receipts) {
    assertReceipt(receipt);
    assert.ok(!receiptIds.has(receipt.id), `duplicate receipt ID: ${receipt.id}`);
    receiptIds.add(receipt.id);
    if (receipt.row) assert.ok(rowIds.has(receipt.row), `${receipt.id}: unknown row ${receipt.row}`);
  }
}

export function buildV1ExactCandidateHostPlan(
  matrix: SupportMatrix = buildV1SupportMatrix(),
): V1ExactCandidateHostPlan[] {
  const hosts = new Map<string, V1ExactCandidateHostPlan>();
  for (const row of matrix.requiredRows) {
    const id = hostId(row);
    const existing = hosts.get(id);
    if (existing) existing.rowIds.push(row.id);
    else
      hosts.set(id, {
        id,
        runtime: row.runtime,
        runtimeVersion: row.runtimeVersion,
        target: row.host,
        rowIds: [row.id],
      });
  }
  return [...hosts.values()]
    .map((host) => ({ ...host, rowIds: host.rowIds.toSorted(compareText) }))
    .toSorted((left, right) => compareText(left.id, right.id));
}

function requiredQualificationSets(matrix: SupportMatrix, row: RequiredRow): string[] {
  return [
    ...new Set(
      matrix.combinations
        .filter(
          (combination) =>
            combination.product === row.product &&
            combination.vue === row.vue &&
            combination.backend === row.backend,
        )
        .flatMap(({ requiredQualificationSets: sets }) => sets),
    ),
  ].toSorted(compareText);
}

function expectedQualificationEnvironments(matrix: SupportMatrix, set: string): string[] {
  switch (set) {
    case "native-node-bun":
      return [];
    case "fresh-package-managers":
      return matrix.environments.packageManagers.map(({ id }) => id).toSorted(compareText);
    case "shipped-editors":
      return matrix.environments.editors.map(({ id }) => id).toSorted(compareText);
    case "wsl2-node":
      return ["wsl2-ubuntu-24.04-x64"];
    default:
      throw new Error(`unknown required qualification set: ${set}`);
  }
}

function receiptIdentityMatchesRow(receipt: V1ExactCandidateReceipt, row: RequiredRow): boolean {
  return (
    receipt.runtime.name === row.runtime &&
    receipt.runtime.version === row.runtimeVersion &&
    receipt.runtime.target === row.host
  );
}

function uniqueSorted(values: readonly string[]): string[] {
  return [...new Set(values)].toSorted(compareText);
}

function uniqueSortedNumbers(values: readonly number[]): number[] {
  return [...new Set(values)].toSorted((left, right) => left - right);
}

function admitRow(
  matrix: SupportMatrix,
  row: RequiredRow,
  candidateSource: string,
  receipts: readonly V1ExactCandidateReceipt[],
): AdmittedRow {
  const owned = receipts.filter(({ row: receiptRow }) => receiptRow === row.id);
  const exact = owned.filter(
    (receipt) => receipt.source === candidateSource && receiptIdentityMatchesRow(receipt, row),
  );
  const positive = exact.filter(
    ({ classification, outcome }) => classification === "automated-positive" && outcome === "passed",
  );
  const negative = exact.filter(
    ({ classification, outcome }) => classification === "automated-negative" && outcome === "passed",
  );
  const manual = exact.filter(
    ({ classification, outcome }) => classification === "manual" && outcome === "passed",
  );
  const qualification = receipts.filter(
    (receipt) => receipt.classification === "qualification-set" && receipt.source === candidateSource,
  );
  const missingQualification = requiredQualificationSets(matrix, row).flatMap((set) =>
    expectedQualificationEnvironments(matrix, set).filter(
      (environment) =>
        !qualification.some(
          (receipt) =>
            receipt.qualificationSet === set &&
            receipt.environment === environment &&
            receipt.outcome === "passed",
        ),
    ).map((environment) => `${set}/${environment}`),
  );
  const qualificationEvidence = qualification.filter((receipt) =>
    requiredQualificationSets(matrix, row).some(
      (set) =>
        receipt.qualificationSet === set &&
        expectedQualificationEnvironments(matrix, set).includes(receipt.environment ?? ""),
    ),
  );
  const skipped = exact.some(({ outcome }) => outcome === "skipped");
  const unsupported = exact.some(({ outcome }) => outcome === "unsupported");
  const hasStaleSource = owned.some(({ source }) => source !== candidateSource);
  const complete = positive.length > 0 && negative.length > 0 && missingQualification.length === 0;
  const state: AdmittedRow["receiptState"] = complete
    ? "complete"
    : unsupported
      ? "unsupported"
      : skipped
        ? "skipped"
        : exact.length === 0 && hasStaleSource
          ? "stale-source"
          : "missing";
  const admitted = [...positive, ...negative, ...manual, ...qualificationEvidence];
  const gap = complete
    ? null
    : state === "stale-source"
      ? "Only stale-source receipts exist for this exact row."
      : state === "skipped"
        ? "The exact-candidate row was skipped."
        : state === "unsupported"
          ? "The exact-candidate row reported an unsupported environment."
          : missingQualification.length > 0
            ? `Missing qualification receipts: ${missingQualification.join(", ")}.`
            : "Missing distinct passed automated positive and negative receipts.";

  return {
    ...row,
    candidateSource,
    candidateRun: admitted.length === 1 ? admitted[0].run : null,
    candidateRuns: uniqueSortedNumbers(admitted.map(({ run }) => run)),
    evidence: uniqueSorted(admitted.map(({ id }) => id)),
    positive: uniqueSorted(positive.map(({ id }) => id)),
    negative: uniqueSorted(negative.map(({ id }) => id)),
    manual: uniqueSorted(manual.map(({ id }) => id)),
    qualification: complete ? "complete" : "gap",
    receiptState: state,
    gap,
  };
}

export function admitV1ExactCandidateHost(
  matrix: SupportMatrix,
  bundle: V1ExactCandidateReceiptBundle,
  candidateSource: string,
  selectedHostId: string,
): V1ExactCandidateHostAdmission {
  assertExactSource(candidateSource);
  validateV1ExactCandidateReceiptBundle(matrix, bundle);
  const host = buildV1ExactCandidateHostPlan(matrix).find(({ id }) => id === selectedHostId);
  assert.ok(host, `unknown host plan: ${selectedHostId}`);
  const byId = new Map(matrix.requiredRows.map((row) => [row.id, row]));
  return {
    schema: V1_EXACT_CANDIDATE_HOST_ADMISSION_SCHEMA,
    candidateSource,
    host,
    rows: host.rowIds.map((id) => admitRow(matrix, byId.get(id)!, candidateSource, bundle.receipts)),
  };
}

function assertHostAdmission(
  admission: V1ExactCandidateHostAdmission,
  candidateSource: string,
): void {
  assert.equal(admission.schema, V1_EXACT_CANDIDATE_HOST_ADMISSION_SCHEMA);
  assert.equal(admission.candidateSource, candidateSource, `${admission.host.id}: candidate source`);
  assert.deepEqual(
    admission.rows.map(({ id }) => id).toSorted(compareText),
    admission.host.rowIds,
    `${admission.host.id}: exact row inventory`,
  );
  assert.ok(
    admission.rows.every((row) => hostId(row) === admission.host.id),
    `${admission.host.id}: foreign row`,
  );
}

export function admitV1ExactCandidate(
  matrix: SupportMatrix,
  candidateSource: string,
  admissions: readonly V1ExactCandidateHostAdmission[],
) {
  assertExactSource(candidateSource);
  const expectedHosts = buildV1ExactCandidateHostPlan(matrix);
  const hostIds = new Set<string>();
  const rows = new Map<string, AdmittedRow>();
  for (const admission of admissions) {
    assertHostAdmission(admission, candidateSource);
    assert.ok(!hostIds.has(admission.host.id), `duplicate host admission: ${admission.host.id}`);
    hostIds.add(admission.host.id);
    for (const row of admission.rows) {
      assert.ok(!rows.has(row.id), `duplicate admitted row: ${row.id}`);
      rows.set(row.id, row);
    }
  }
  assert.deepEqual([...hostIds].toSorted(compareText), expectedHosts.map(({ id }) => id));
  assert.deepEqual(
    [...rows.keys()].toSorted(compareText),
    matrix.requiredRows.map(({ id }) => id).toSorted(compareText),
  );
  const requiredRows = matrix.requiredRows.map(({ id }) => rows.get(id)!);
  const hostComplete = requiredRows.every(({ qualification }) => qualification === "complete");
  const knownGaps = matrix.knownGaps.map((gap) =>
    gap.id === "host-runtime-join" && hostComplete
      ? {
          ...gap,
          state: "complete",
          summary: `Every required host row is admitted against ${candidateSource}.`,
        }
      : gap,
  );
  const completeRequiredRows = requiredRows.filter(
    ({ qualification }) => qualification === "complete",
  ).length;
  const blockingGaps = knownGaps.filter(({ state }) => state === "blocking").length;
  return {
    ...matrix,
    admission: {
      candidateSource,
      hostPlans: expectedHosts.length,
      receiptDigest: createHash("sha256")
        .update(
          JSON.stringify(
            requiredRows.map(({ id, evidence }) => [id, evidence]),
          ),
        )
        .digest("hex"),
    },
    requiredRows,
    knownGaps,
    releaseReady: hostComplete && blockingGaps === 0,
    counts: {
      ...matrix.counts,
      completeRequiredRows,
      blockingGaps,
    },
  };
}
