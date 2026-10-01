import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  loadFormatterHistoryAudit,
  validateFormatterHistoryAudit,
  validateFormatterHistoryExecution,
} from "../differential/formatter-history-audit.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("full formatter denominator rejects missing original fixes, source arms and false native credit", () => {
  const { audit, cases } = loadFormatterHistoryAudit(root);
  assert.equal(cases.size, 300);
  for (const mutate of [
    (copy) => {
      copy.fixes.pop();
    },
    (copy) => {
      copy.fixes[1].commit = copy.fixes[0].commit;
    },
    (copy) => {
      copy.caseIndex.pop();
    },
    (copy) => {
      copy.manifestInventory[0].sha256 = "0".repeat(64);
    },
    (copy) => {
      copy.fixes[0].requirements[0].caseIds = [];
    },
    (copy) => {
      copy.fixes[0].currentWitnesses[0].sourceSha256 = "0".repeat(64);
    },
    (copy) => {
      copy.validation.nativeHandled = 1;
    },
    (copy) => {
      copy.supplementalFixes[0].unresolvedRequirements.push({ id: "missing" });
    },
  ]) {
    const copy = structuredClone(audit);
    mutate(copy);
    assert.throws(() => validateFormatterHistoryAudit(copy, root));
  }
  // Registration alone cannot stand in for the real source-built API reports.
  assert.throws(() => validateFormatterHistoryExecution(root, []));
});
