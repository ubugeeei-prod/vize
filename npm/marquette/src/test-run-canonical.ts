import type {
  TestRunEvidence,
  TestRunRetainedEvidence,
  TestRunSuiteExecution,
  TestRunTargetExecution,
} from "./test-run-model.js";

/** Prefix of every test-run deployment admission id. */
export const TEST_RUN_ADMISSION_PREFIX = "test-run:";

const ADMISSION_FINGERPRINT = /^[a-f0-9]{64}$/;

function retained(evidence: TestRunRetainedEvidence): TestRunRetainedEvidence {
  return { reference: evidence.reference, fingerprint: evidence.fingerprint };
}

function target(execution: TestRunTargetExecution): TestRunTargetExecution {
  return { id: execution.id, kind: execution.kind, environment: execution.environment };
}

function suite(execution: TestRunSuiteExecution): TestRunSuiteExecution {
  return {
    id: execution.id,
    targetId: execution.targetId,
    kind: execution.kind,
    shardIndex: execution.shardIndex,
    shardCount: execution.shardCount,
    outcome: execution.outcome,
    passed: execution.passed,
    failed: execution.failed,
    skipped: execution.skipped,
    retries: execution.retries,
    durationMs: execution.durationMs,
    invocationFingerprint: execution.invocationFingerprint,
    report: retained(execution.report),
    log: retained(execution.log),
  };
}

/**
 * Serializes a test-run evidence record canonically.
 *
 * Property order matches the record schema, targets sort by id, suites sort
 * by id then shard index, and selection identifiers sort lexicographically,
 * so equivalent records produce byte-identical JSON in every language. Call
 * validation before trusting the record; canonicalization does not make an
 * invalid record valid.
 */
export function canonicalTestRunJson(evidence: TestRunEvidence): string {
  const canonical = {
    format: evidence.format,
    formatVersion: evidence.formatVersion ?? 1,
    id: evidence.id,
    application: evidence.application,
    environment: evidence.environment,
    contractFingerprint: evidence.contractFingerprint,
    sourceRevision: evidence.sourceRevision,
    release: evidence.release,
    artifact: {
      id: evidence.artifact.id,
      fingerprint: evidence.artifact.fingerprint,
      sizeBytes: evidence.artifact.sizeBytes,
    },
    startedAt: evidence.startedAt,
    completedAt: evidence.completedAt,
    validUntil: evidence.validUntil,
    runner: {
      identity: evidence.runner.identity,
      authenticationEvidence: retained(evidence.runner.authenticationEvidence),
      isolation: evidence.runner.isolation,
      invocationFingerprint: evidence.runner.invocationFingerprint,
      environmentEvidence: retained(evidence.runner.environmentEvidence),
      environmentFingerprint: evidence.runner.environmentFingerprint,
    },
    selection: {
      targetIds: [...evidence.selection.targetIds].sort(),
      suiteIds: [...evidence.selection.suiteIds].sort(),
    },
    targets: evidence.targets
      .map(target)
      .sort((left, right) => (left.id < right.id ? -1 : left.id > right.id ? 1 : 0)),
    suites: evidence.suites
      .map(suite)
      .sort((left, right) =>
        left.id !== right.id ? (left.id < right.id ? -1 : 1) : left.shardIndex - right.shardIndex,
      ),
    verification: {
      verifier: evidence.verification.verifier,
      completedAt: evidence.verification.completedAt,
      outcome: evidence.verification.outcome,
      targetCount: evidence.verification.targetCount,
      suiteCount: evidence.verification.suiteCount,
      passed: evidence.verification.passed,
      failed: evidence.verification.failed,
      skipped: evidence.verification.skipped,
      retries: evidence.verification.retries,
      evidence: retained(evidence.verification.evidence),
    },
  };
  return JSON.stringify(canonical);
}

/**
 * Returns the lowercase SHA-256 fingerprint of the canonical record.
 *
 * The fingerprint is the exact value admitted as `test-run:<sha256>` by
 * deployment gates. Uses the Web Crypto API available in every supported
 * runtime.
 */
export async function testRunFingerprint(evidence: TestRunEvidence): Promise<string> {
  const bytes = new TextEncoder().encode(canonicalTestRunJson(evidence));
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  let fingerprint = "";
  for (const byte of new Uint8Array(digest)) {
    fingerprint += byte.toString(16).padStart(2, "0");
  }
  return fingerprint;
}

/** Returns the `test-run:<sha256>` admission id for one record. */
export async function testRunAdmissionId(evidence: TestRunEvidence): Promise<string> {
  return `${TEST_RUN_ADMISSION_PREFIX}${await testRunFingerprint(evidence)}`;
}

/**
 * Returns the fingerprint named by a `test-run:<sha256>` admission id.
 *
 * Returns `undefined` unless the prefix, length, and lowercase hexadecimal
 * grammar are all exact.
 */
export function parseTestRunAdmissionId(id: string): string | undefined {
  if (!id.startsWith(TEST_RUN_ADMISSION_PREFIX)) {
    return undefined;
  }
  const fingerprint = id.slice(TEST_RUN_ADMISSION_PREFIX.length);
  return ADMISSION_FINGERPRINT.test(fingerprint) ? fingerprint : undefined;
}
