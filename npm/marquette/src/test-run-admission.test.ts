import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import type { TestRunEvidence } from "./test-run.js";
import {
  TEST_RUN_DENIAL_CODES,
  admitTestRun,
  decideTestRunAdmission,
  testRunDenialCode,
  type TestRunAdmissionDecision,
  type TestRunCandidate,
} from "./test-run-admission.js";
import { testRunAdmissionId } from "./test-run-canonical.js";

function fixture(name: string): URL {
  return new URL(`../../../tests/fixtures/test-run-evidence/${name}`, import.meta.url);
}

function readEvidence(name = "valid.json"): TestRunEvidence {
  return JSON.parse(readFileSync(fixture(name), "utf8")) as TestRunEvidence;
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

void test("the denial vocabulary is sorted and deduplicates into decisions", async () => {
  assert.deepEqual([...TEST_RUN_DENIAL_CODES], [...TEST_RUN_DENIAL_CODES].sort());

  const evidence = readEvidence();
  const decision = await decideTestRunAdmission(
    {
      ...evidence,
      application: "not-the-candidate",
      contractFingerprint: "tampered",
      artifact: { ...evidence.artifact, fingerprint: "also-tampered" },
    },
    candidateFor(evidence),
    "test-run:junk",
    "yesterday",
  );
  assert.equal(decision.allowed, false);
  assert.deepEqual(decision.denialCodes, [
    "admission-id-malformed",
    "admission-time-malformed",
    "candidate-application-mismatch",
    "candidate-artifact-fingerprint-mismatch",
    "candidate-contract-fingerprint-mismatch",
    "record-invalid",
  ]);
  assert.ok(decision.diagnostics.length > decision.denialCodes.length);
  for (const diagnostic of decision.diagnostics) {
    assert.ok(TEST_RUN_DENIAL_CODES.includes(testRunDenialCode(diagnostic)));
  }
});

interface SharedDecisionCase {
  readonly name: string;
  readonly family: string;
  readonly evidence: string;
  readonly candidate: TestRunCandidate;
  readonly admissionId: string;
  readonly now: string;
  readonly decision: TestRunAdmissionDecision;
}

void test("reproduces every shared admission decision", async () => {
  const document = JSON.parse(readFileSync(fixture("admission-decisions.json"), "utf8")) as {
    cases: SharedDecisionCase[];
  };
  assert.ok(document.cases.length > 0);

  for (const sharedCase of document.cases) {
    const decision = await decideTestRunAdmission(
      readEvidence(sharedCase.evidence),
      sharedCase.candidate,
      sharedCase.admissionId,
      sharedCase.now,
    );
    assert.deepEqual(
      JSON.parse(JSON.stringify(decision)),
      sharedCase.decision,
      `decision mismatch for case ${sharedCase.name}`,
    );
  }
});
