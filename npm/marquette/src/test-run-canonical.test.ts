import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import type { TestRunEvidence } from "./test-run.js";
import {
  canonicalTestRunJson,
  parseTestRunAdmissionId,
  testRunAdmissionId,
  testRunFingerprint,
} from "./test-run-canonical.js";

function fixture(name: string): string {
  return readFileSync(
    new URL(`../../../tests/fixtures/test-run-evidence/${name}`, import.meta.url),
    "utf8",
  );
}

const evidence = JSON.parse(fixture("valid.json")) as TestRunEvidence;

void test("canonical serialization matches the shared fixture bytes", () => {
  assert.equal(canonicalTestRunJson(evidence), fixture("valid.canonical").trim());
});

void test("fingerprint and admission id match the shared fixture", async () => {
  const expected = fixture("valid.sha256").trim();
  assert.equal(await testRunFingerprint(evidence), expected);
  const admissionId = await testRunAdmissionId(evidence);
  assert.equal(admissionId, `test-run:${expected}`);
  assert.equal(parseTestRunAdmissionId(admissionId), expected);
});

void test("execution order does not change the canonical bytes", () => {
  const reversed = {
    ...evidence,
    targets: [...evidence.targets].reverse(),
    suites: [...evidence.suites].reverse(),
  };
  assert.equal(canonicalTestRunJson(reversed), canonicalTestRunJson(evidence));
});

void test("admission ids must be exact", () => {
  assert.equal(parseTestRunAdmissionId("test-run:"), undefined);
  assert.equal(parseTestRunAdmissionId(`tests:${"a".repeat(64)}`), undefined);
  assert.equal(parseTestRunAdmissionId(`test-run:${"A".repeat(64)}`), undefined);
  assert.equal(parseTestRunAdmissionId(`test-run:${"a".repeat(63)}`), undefined);
});
