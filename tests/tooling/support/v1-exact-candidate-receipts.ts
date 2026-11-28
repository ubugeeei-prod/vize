import { createHash } from "node:crypto";

import {
  V1_EXACT_CANDIDATE_BUNDLE_SCHEMA,
  V1_EXACT_CANDIDATE_RECEIPT_SCHEMA,
  type V1ExactCandidateReceipt,
  type V1ExactCandidateReceiptBundle,
} from "../../../tools/support/compat/v1/exact-candidate.ts";
import { buildV1SupportMatrix } from "../../../tools/support/compat/v1/support-matrix.ts";

export const matrix = buildV1SupportMatrix();
export const candidate = "1".repeat(40);
export const runIdentity = { run: 44_000_000_000, runAttempt: 1 } as const;

export function hash(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

export function receipt(
  id: string,
  overrides: Partial<V1ExactCandidateReceipt>,
): V1ExactCandidateReceipt {
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

export function completeBundle(): V1ExactCandidateReceiptBundle {
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
