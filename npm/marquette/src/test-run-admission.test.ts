import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import type { TestRunEvidence } from "./test-run.js";
import { admitTestRun, type TestRunCandidate } from "./test-run-admission.js";
import { testRunAdmissionId } from "./test-run-canonical.js";

function readEvidence(): TestRunEvidence {
  return JSON.parse(
    readFileSync(
      new URL("../../../tests/fixtures/test-run-evidence/valid.json", import.meta.url),
      "utf8",
    ),
  ) as TestRunEvidence;
}

function candidateFor(evidence: TestRunEvidence): TestRunCandidate {
  return {
    application: evidence.application,
    environment: evidence.environment,
    contractFingerprint: evidence.contractFingerprint,
    sourceRevision: evidence.sourceRevision,
    release: evidence.release,
    artifactFingerprint: evidence.artifact.fingerprint,
  };
}

const NOW = "2026-07-22T00:00:00.000Z";

function codes(diagnostics: { code: string }[]): string[] {
  return diagnostics.map((diagnostic) => diagnostic.code);
}

void test("an exact candidate is admitted", async () => {
  const evidence = readEvidence();
  const id = await testRunAdmissionId(evidence);
  assert.deepEqual(await admitTestRun(evidence, candidateFor(evidence), id, NOW), []);
});

void test("malformed admission ids are rejected", async () => {
  const evidence = readEvidence();
  const rejected = await admitTestRun(evidence, candidateFor(evidence), "test-run:junk", NOW);
  assert.deepEqual(codes(rejected), ["VIZE_MARQUETTE_141"]);
});

void test("a different record cannot reuse an admission id", async () => {
  const evidence = readEvidence();
  const id = await testRunAdmissionId(evidence);
  const tampered = {
    ...evidence,
    suites: evidence.suites.map((suite, index) =>
      index === 0 ? { ...suite, passed: suite.passed - 1 } : suite,
    ),
    verification: { ...evidence.verification, passed: evidence.verification.passed - 1 },
  };
  assert.deepEqual(codes(await admitTestRun(tampered, candidateFor(evidence), id, NOW)), [
    "VIZE_MARQUETTE_142",
  ]);
});

void test("every candidate binding must match", async () => {
  const evidence = readEvidence();
  const id = await testRunAdmissionId(evidence);
  const candidate = {
    ...candidateFor(evidence),
    release: "0.999.0",
    environment: "staging",
  };
  assert.deepEqual(codes(await admitTestRun(evidence, candidate, id, NOW)), [
    "VIZE_MARQUETTE_144",
    "VIZE_MARQUETTE_144",
  ]);
});

void test("expired records and malformed instants fail closed", async () => {
  const evidence = readEvidence();
  const id = await testRunAdmissionId(evidence);
  assert.deepEqual(
    codes(await admitTestRun(evidence, candidateFor(evidence), id, evidence.validUntil)),
    ["VIZE_MARQUETTE_145"],
  );
  assert.deepEqual(codes(await admitTestRun(evidence, candidateFor(evidence), id, "yesterday")), [
    "VIZE_MARQUETTE_148",
  ]);
});

void test("skipped tests are not admitted", async () => {
  const evidence = readEvidence();
  const skipped = {
    ...evidence,
    suites: evidence.suites.map((suite, index) =>
      index === 0 ? { ...suite, skipped: 1, passed: suite.passed - 1 } : suite,
    ),
    verification: {
      ...evidence.verification,
      skipped: 1,
      passed: evidence.verification.passed - 1,
    },
  };
  const id = await testRunAdmissionId(skipped);
  assert.deepEqual(codes(await admitTestRun(skipped, candidateFor(skipped), id, NOW)), [
    "VIZE_MARQUETTE_147",
  ]);
});
