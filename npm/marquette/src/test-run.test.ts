import assert from "node:assert/strict";
import test from "node:test";

import {
  TEST_RUN_EVIDENCE_FORMAT,
  TEST_RUN_EVIDENCE_FORMAT_VERSION,
  defineTestRunEvidence,
} from "./test-run.js";

void test("defineTestRunEvidence returns its input unchanged", () => {
  const retained = {
    reference: `sha256:${"3".repeat(64)}`,
    fingerprint: "3".repeat(64),
  } as const;
  const evidence = defineTestRunEvidence({
    format: TEST_RUN_EVIDENCE_FORMAT,
    formatVersion: TEST_RUN_EVIDENCE_FORMAT_VERSION,
    id: "run-1",
    application: "example",
    environment: "production",
    contractFingerprint: "1".repeat(64),
    sourceRevision: "a".repeat(40),
    release: "0.298.0",
    artifact: { id: "web-bundle", fingerprint: "2".repeat(64), sizeBytes: 1024 },
    startedAt: "2026-07-21T00:00:00.000Z",
    completedAt: "2026-07-21T00:10:00.000Z",
    validUntil: "2026-07-28T00:10:00.000Z",
    runner: {
      identity: "ci.runner-1",
      authenticationEvidence: retained,
      isolation: "ephemeral",
      invocationFingerprint: "4".repeat(64),
      environmentEvidence: retained,
      environmentFingerprint: "6".repeat(64),
    },
    selection: { targetIds: ["web"], suiteIds: ["unit"] },
    targets: [{ id: "web", kind: "web", environment: "production" }],
    suites: [
      {
        id: "unit",
        targetId: "web",
        kind: "unit",
        shardIndex: 1,
        shardCount: 1,
        outcome: "passed",
        passed: 12,
        failed: 0,
        skipped: 0,
        retries: 0,
        durationMs: 61000,
        invocationFingerprint: "7".repeat(64),
        report: retained,
        log: retained,
      },
    ],
    verification: {
      verifier: "release.verifier",
      completedAt: "2026-07-21T00:11:00.000Z",
      outcome: "accepted",
      targetCount: 1,
      suiteCount: 1,
      passed: 12,
      failed: 0,
      skipped: 0,
      retries: 0,
      evidence: retained,
    },
  });

  assert.equal(evidence.id, "run-1");
  assert.equal(evidence.suites[0]?.targetId, "web");
  assert.equal(evidence.format, "vize.test-run.evidence");
});
