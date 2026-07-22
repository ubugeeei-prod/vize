import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import type { TestRunEvidence } from "./test-run.js";
import type { TestRunAdmissionDecision, TestRunCandidate } from "./test-run-admission.js";
import { validateTestRunCheck, verifyTestRunCheck, type TestRunCheck } from "./test-run-check.js";
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

async function checkFor(evidence: TestRunEvidence): Promise<TestRunCheck> {
  return {
    format: "vize.test-run.check",
    formatVersion: 1,
    evidence: await testRunAdmissionId(evidence),
    candidate: candidateFor(evidence),
    observer: "release.gate",
    observedAt: "2026-07-21T00:12:00.000Z",
  };
}

const NOW = "2026-07-22T00:00:00.000Z";

void test("a release-bound check verifies", async () => {
  const evidence = readEvidence();
  const check = await checkFor(evidence);
  assert.deepEqual(validateTestRunCheck(check), []);
  const decision = await verifyTestRunCheck(check, candidateFor(evidence), evidence, NOW);
  assert.equal(decision.allowed, true);
  assert.deepEqual(decision.denialCodes, []);
  assert.deepEqual(decision.diagnostics, []);
});

void test("generic test-result references fail closed", async () => {
  const evidence = readEvidence();
  const check = { ...(await checkFor(evidence)), evidence: "reports/junit-summary.xml" };
  const decision = await verifyTestRunCheck(check, candidateFor(evidence), evidence, NOW);
  assert.equal(decision.allowed, false);
  assert.deepEqual(decision.denialCodes, ["admission-id-malformed"]);
});

void test("substituted checks and dependent observers are rejected", async () => {
  const evidence = readEvidence();
  const base = await checkFor(evidence);
  const check = {
    ...base,
    candidate: { ...base.candidate, release: "0.999.0" },
    observer: evidence.runner.identity,
    observedAt: "2026-07-21T00:10:30.000Z",
  };
  const decision = await verifyTestRunCheck(check, candidateFor(evidence), evidence, NOW);
  assert.deepEqual(decision.denialCodes, [
    "check-candidate-mismatch",
    "check-invalid",
    "check-observer-not-independent",
  ]);
});

interface SharedCheckCase {
  readonly name: string;
  readonly family: string;
  readonly evidence: string;
  readonly check: TestRunCheck;
  readonly candidate: TestRunCandidate;
  readonly now: string;
  readonly decision: TestRunAdmissionDecision;
}

void test("reproduces every shared check decision", async () => {
  const document = JSON.parse(readFileSync(fixture("check-decisions.json"), "utf8")) as {
    cases: SharedCheckCase[];
  };
  assert.ok(document.cases.length > 0);

  for (const sharedCase of document.cases) {
    const decision = await verifyTestRunCheck(
      sharedCase.check,
      sharedCase.candidate,
      readEvidence(sharedCase.evidence),
      sharedCase.now,
    );
    assert.deepEqual(
      JSON.parse(JSON.stringify(decision)),
      sharedCase.decision,
      `decision mismatch for case ${sharedCase.name}`,
    );
  }
});
