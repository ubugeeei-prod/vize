import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";

import {
  admitV1ExactCandidate,
  admitV1ExactCandidateHost,
  buildV1ExactCandidateHostPlan,
  V1_EXACT_CANDIDATE_BUNDLE_SCHEMA,
  V1_EXACT_CANDIDATE_RECEIPT_SCHEMA,
  type V1ExactCandidateReceipt,
  type V1ExactCandidateReceiptBundle,
} from "../../tools/support/compat/v1/exact-candidate.ts";
import { buildV1SupportMatrix } from "../../tools/support/compat/v1/support-matrix.ts";

const matrix = buildV1SupportMatrix();
const candidate = "1".repeat(40);
const runIdentity = { run: 44_000_000_000, runAttempt: 1 } as const;

function hash(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function receipt(id: string, overrides: Partial<V1ExactCandidateReceipt>): V1ExactCandidateReceipt {
  return {
    schema: V1_EXACT_CANDIDATE_RECEIPT_SCHEMA,
    id,
    classification: "qualification-set",
    outcome: "passed",
    row: null,
    qualificationSet: "fresh-package-managers",
    environment: "npm",
    source: candidate,
    run: runIdentity.run,
    runAttempt: runIdentity.runAttempt,
    job: id,
    fixtureSha256: hash(`${id}:fixture`),
    dependencySha256: hash(`${id}:dependency`),
    binarySha256: hash(`${id}:binary`),
    runtime: { name: "node", version: "24", target: "x86_64-unknown-linux-gnu" },
    manualReason: null,
    ...overrides,
  };
}

function completeBundle(): V1ExactCandidateReceiptBundle {
  const receipts: V1ExactCandidateReceipt[] = [];
  for (const row of matrix.requiredRows) {
    for (const classification of ["automated-positive", "automated-negative"] as const) {
      const polarity = classification === "automated-positive" ? "positive" : "negative";
      receipts.push(
        receipt(`${polarity}:${row.id}`, {
          classification,
          row: row.id,
          qualificationSet: null,
          environment: null,
          runtime: { name: row.runtime, version: row.runtimeVersion, target: row.host },
        }),
      );
    }
  }
  for (const manager of matrix.environments.packageManagers)
    receipts.push(
      receipt(`manager:${manager.id}`, {
        qualificationSet: "fresh-package-managers",
        environment: manager.id,
      }),
    );
  for (const editor of matrix.environments.editors)
    receipts.push(
      receipt(`editor:${editor.id}`, {
        qualificationSet: "shipped-editors",
        environment: editor.id,
      }),
    );
  receipts.push(
    receipt("wsl2:node", {
      qualificationSet: "wsl2-node",
      environment: "wsl2-ubuntu-24.04-x64",
      runtime: {
        name: "node",
        version: "24.14.0",
        target: "wsl2-ubuntu-24.04-x64",
      },
    }),
  );
  return { schema: V1_EXACT_CANDIDATE_BUNDLE_SCHEMA, receipts };
}

function admissions(bundle: V1ExactCandidateReceiptBundle) {
  return buildV1ExactCandidateHostPlan(matrix).map(({ id }) =>
    admitV1ExactCandidateHost(matrix, bundle, candidate, id, runIdentity),
  );
}

void test("the host plan is generated once from every required row", () => {
  const plan = buildV1ExactCandidateHostPlan(matrix);
  assert.equal(plan.length, matrix.counts.hostRuntimeCombinations);
  assert.equal(
    plan.reduce((count, host) => count + host.rowIds.length, 0),
    matrix.requiredRows.length,
  );
  assert.deepEqual(
    plan.flatMap(({ rowIds }) => rowIds).toSorted(),
    matrix.requiredRows.map(({ id }) => id).toSorted(),
  );
});

void test("one exact candidate admits all host rows without granting unrelated v1 gaps", () => {
  const admitted = admitV1ExactCandidate(
    matrix,
    candidate,
    admissions(completeBundle()),
    runIdentity,
  );
  assert.equal(admitted.counts.completeRequiredRows, 420);
  assert.equal(
    admitted.requiredRows.every(({ receiptState }) => receiptState === "complete"),
    true,
  );
  assert.equal(admitted.knownGaps.find(({ id }) => id === "host-runtime-join")?.state, "complete");
  assert.equal(admitted.counts.blockingGaps, matrix.counts.blockingGaps - 1);
  assert.equal(admitted.releaseReady, false);
  assert.match(admitted.admission.receiptDigest, /^[0-9a-f]{64}$/u);
});

void test("stale source, skipped, unsupported and missing evidence remain distinct", () => {
  const selected = matrix.requiredRows[0];
  const host = buildV1ExactCandidateHostPlan(matrix).find(({ rowIds }) =>
    rowIds.includes(selected.id),
  )!;
  const cases = [
    {
      expected: "stale-source",
      mutate: (entry: V1ExactCandidateReceipt) => ({ ...entry, source: "2".repeat(40) }),
    },
    {
      expected: "skipped",
      mutate: (entry: V1ExactCandidateReceipt) => ({ ...entry, outcome: "skipped" as const }),
    },
    {
      expected: "unsupported",
      mutate: (entry: V1ExactCandidateReceipt) => ({ ...entry, outcome: "unsupported" as const }),
    },
  ] as const;
  for (const { expected, mutate } of cases) {
    const bundle = completeBundle();
    bundle.receipts = bundle.receipts.map((entry) =>
      entry.row === selected.id ? mutate(entry) : entry,
    );
    const admitted = admitV1ExactCandidateHost(matrix, bundle, candidate, host.id, runIdentity);
    assert.equal(admitted.rows.find(({ id }) => id === selected.id)?.receiptState, expected);
  }
  const missing = completeBundle();
  missing.receipts = missing.receipts.filter(({ row }) => row !== selected.id);
  assert.equal(
    admitV1ExactCandidateHost(matrix, missing, candidate, host.id, runIdentity).rows.find(
      ({ id }) => id === selected.id,
    )?.receiptState,
    "missing",
  );
});

void test("manual evidence is scoped and never substitutes for automated negative evidence", () => {
  const selected = matrix.requiredRows[0];
  const host = buildV1ExactCandidateHostPlan(matrix).find(({ rowIds }) =>
    rowIds.includes(selected.id),
  )!;
  const bundle = completeBundle();
  bundle.receipts = bundle.receipts.filter(
    ({ row, classification }) => row !== selected.id || classification !== "automated-negative",
  );
  bundle.receipts.push(
    receipt(`manual:${selected.id}`, {
      classification: "manual",
      row: selected.id,
      qualificationSet: null,
      environment: null,
      manualReason: "Physical editor host observation; no automation substitute claimed.",
      runtime: {
        name: selected.runtime,
        version: selected.runtimeVersion,
        target: selected.host,
      },
    }),
  );
  const row = admitV1ExactCandidateHost(matrix, bundle, candidate, host.id, runIdentity).rows.find(
    ({ id }) => id === selected.id,
  )!;
  assert.equal(row.qualification, "gap");
  assert.deepEqual(row.manual, [`manual:${selected.id}`]);
  assert.deepEqual(row.negative, []);
});

void test("required package-manager evidence is exact and fail-closed", () => {
  const selected = matrix.requiredRows.find(({ product }) => product === "vite")!;
  const host = buildV1ExactCandidateHostPlan(matrix).find(({ rowIds }) =>
    rowIds.includes(selected.id),
  )!;
  const bundle = completeBundle();
  bundle.receipts = bundle.receipts.filter(({ id }) => id !== "manager:bun");
  const row = admitV1ExactCandidateHost(matrix, bundle, candidate, host.id, runIdentity).rows.find(
    ({ id }) => id === selected.id,
  )!;
  assert.equal(row.qualification, "gap");
  assert.match(row.gap ?? "", /fresh-package-managers\/bun/u);
});

void test("receipt identity rejects duplicate IDs and incomplete hashes", () => {
  const bundle = completeBundle();
  bundle.receipts.push({ ...bundle.receipts[0] });
  const host = buildV1ExactCandidateHostPlan(matrix)[0];
  assert.throws(
    () => admitV1ExactCandidateHost(matrix, bundle, candidate, host.id, runIdentity),
    /duplicate receipt ID/u,
  );
  const invalid = completeBundle();
  invalid.receipts[0] = { ...invalid.receipts[0], binarySha256: "missing" };
  assert.throws(
    () => admitV1ExactCandidateHost(matrix, invalid, candidate, host.id, runIdentity),
    /invalid binarySha256/u,
  );
});

void test("receipt run identity is exact and fail-closed", () => {
  const bundle = completeBundle();
  bundle.receipts[0] = { ...bundle.receipts[0], runAttempt: runIdentity.runAttempt + 1 };
  const host = buildV1ExactCandidateHostPlan(matrix)[0];
  assert.throws(
    () => admitV1ExactCandidateHost(matrix, bundle, candidate, host.id, runIdentity),
    /candidate run attempt/u,
  );
});

void test("WSL2 qualification requires the exact runtime identity", () => {
  const selected = matrix.requiredRows.find(({ product }) => product === "content-mapper")!;
  const host = buildV1ExactCandidateHostPlan(matrix).find(({ rowIds }) =>
    rowIds.includes(selected.id),
  )!;
  const bundle = completeBundle();
  const wsl = bundle.receipts.find(({ id }) => id === "wsl2:node")!;
  wsl.runtime = { ...wsl.runtime, target: "x86_64-unknown-linux-gnu" };
  const row = admitV1ExactCandidateHost(matrix, bundle, candidate, host.id, runIdentity).rows.find(
    ({ id }) => id === selected.id,
  )!;
  assert.equal(row.qualification, "gap");
  assert.match(row.gap ?? "", /wsl2-node\/wsl2-ubuntu-24\.04-x64/u);
});

void test("external admissions cannot change the candidate source", () => {
  const hostAdmissions = admissions(completeBundle());
  hostAdmissions[0].rows[0].candidateSource = "2".repeat(40);
  assert.throws(
    () => admitV1ExactCandidate(matrix, candidate, hostAdmissions, runIdentity),
    /foreign candidate source/u,
  );
});

void test("the receipt digest binds canonical receipt payloads", () => {
  const firstBundle = completeBundle();
  const secondBundle = completeBundle();
  secondBundle.receipts[0] = {
    ...secondBundle.receipts[0],
    binarySha256: hash("changed-binary"),
  };
  const first = admitV1ExactCandidate(matrix, candidate, admissions(firstBundle), runIdentity);
  const second = admitV1ExactCandidate(matrix, candidate, admissions(secondBundle), runIdentity);
  assert.notEqual(first.admission.receiptDigest, second.admission.receiptDigest);
});
