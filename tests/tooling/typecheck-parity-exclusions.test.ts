import assert from "node:assert/strict";
import { test } from "node:test";

import {
  readTypecheckParityExclusions,
  validateTypecheckParityExclusions,
} from "../../tools/fixtures/typecheck-parity-exclusions.mjs";

const ledger = readTypecheckParityExclusions();

test("typecheck parity ledger exactly partitions enabled and excluded fixtures", () => {
  assert.deepEqual(validateTypecheckParityExclusions(ledger), {
    enabledCount: 12,
    excludedCount: 122,
    totalCount: 134,
  });
});

test("typecheck parity exclusion mutations fail closed", () => {
  for (const [name, mutate, message] of mutations()) {
    const value = structuredClone(ledger);
    mutate(value);
    assert.throws(() => validateTypecheckParityExclusions(value), message, name);
  }
});

function mutations(): Array<[string, (value: typeof ledger) => void, RegExp]> {
  return [
    ["schema", (value) => (value.schema = "other"), /unsupported exclusion schema/],
    ["version", (value) => (value.version = 2), /unsupported exclusion version/],
    ["unknown root field", (value) => (value.extra = true), /shape is not closed/],
    ["missing project", (value) => value.exclusions.pop(), /exactly partition/],
    [
      "duplicate project",
      (value) => value.exclusions.push(structuredClone(value.exclusions[0])),
      /duplicates/,
    ],
    ["unsorted projects", (value) => value.exclusions.reverse(), /codepoint sorted/],
    ["unknown project", (value) => (value.exclusions[0].project = "unknown"), /unknown excluded/],
    [
      "enabled project",
      (value) => (value.exclusions[0].project = "create-vue"),
      /enabled parity project is excluded/,
    ],
    ["wrong policy", (value) => (value.exclusions[0].policy = "no-tsconfig"), /policy drifted/],
    ["owner", (value) => (value.policies[0].ownerIssue = 1), /ownership or expiry drifted/],
    ["reason", (value) => (value.policies[0].reason = "trust me"), /ownership or expiry drifted/],
    ["expiry", (value) => (value.policies[0].expiresWhen = "never"), /ownership or expiry drifted/],
    ["unknown exclusion field", (value) => (value.exclusions[0].waived = true), /not closed/],
  ];
}
