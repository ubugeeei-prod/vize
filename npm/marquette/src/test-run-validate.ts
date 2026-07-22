import type { MarquetteDiagnostic } from "./validate.js";
import type { TestRunEvidence } from "./test-run-model.js";
import { TEST_RUN_EVIDENCE_FORMAT, TEST_RUN_EVIDENCE_FORMAT_VERSION } from "./test-run-model.js";
import {
  checkDigest,
  checkIdentifier,
  checkRetainedEvidence,
  checkSafeInteger,
  checkSourceRevision,
  checkTimestamp,
  error,
  evidencePath,
} from "./test-run-validate-rules.js";
import {
  TEST_RUN_MAX_SUITES,
  TEST_RUN_MAX_TARGETS,
  validateSuites,
  validateTargets,
  validateVerification,
} from "./test-run-validate-executions.js";

export {
  TEST_RUN_MAX_SHARDS,
  TEST_RUN_MAX_SUITES,
  TEST_RUN_MAX_TARGETS,
} from "./test-run-validate-executions.js";
export type { MarquetteDiagnostic, MarquetteDiagnosticSeverity } from "./validate.js";

/**
 * Validates a complete test-run evidence record.
 *
 * Diagnostics are deterministic and sorted by path, code, and message so the
 * same record produces identical CLI, promotion, test, and CI output in
 * every consuming language. A record with any error diagnostic must never
 * satisfy a deployment check.
 */
export function validateTestRunEvidence(evidence: TestRunEvidence): MarquetteDiagnostic[] {
  const diagnostics: MarquetteDiagnostic[] = [];

  if (evidence.format !== TEST_RUN_EVIDENCE_FORMAT) {
    diagnostics.push(
      error("VIZE_MARQUETTE_101", "format", "unsupported test-run evidence format marker"),
    );
  }
  if ((evidence.formatVersion ?? 1) !== TEST_RUN_EVIDENCE_FORMAT_VERSION) {
    diagnostics.push(
      error("VIZE_MARQUETTE_102", "formatVersion", "unsupported test-run evidence format version"),
    );
  }

  checkIdentifier(evidence.id, "id", diagnostics);
  checkIdentifier(evidence.application, "application", diagnostics);
  checkIdentifier(evidence.environment, "environment", diagnostics);
  checkDigest(evidence.contractFingerprint, "contractFingerprint", diagnostics);
  checkSourceRevision(evidence.sourceRevision, "sourceRevision", diagnostics);
  if (evidence.release.length === 0 || evidence.release.length > 256) {
    diagnostics.push(
      error("VIZE_MARQUETTE_106", "release", "release must be between 1 and 256 characters"),
    );
  }

  checkIdentifier(evidence.artifact.id, "artifact.id", diagnostics);
  checkDigest(evidence.artifact.fingerprint, "artifact.fingerprint", diagnostics);
  if (evidence.artifact.sizeBytes === 0) {
    diagnostics.push(
      error("VIZE_MARQUETTE_110", "artifact.sizeBytes", "artifact size must be at least one byte"),
    );
  }
  checkSafeInteger(evidence.artifact.sizeBytes, "artifact.sizeBytes", diagnostics);

  checkTimestamp(evidence.startedAt, "startedAt", diagnostics);
  checkTimestamp(evidence.completedAt, "completedAt", diagnostics);
  checkTimestamp(evidence.validUntil, "validUntil", diagnostics);
  if (evidence.completedAt < evidence.startedAt) {
    diagnostics.push(
      error("VIZE_MARQUETTE_112", "completedAt", "run completion must not precede its start"),
    );
  }
  if (evidence.validUntil <= evidence.completedAt) {
    diagnostics.push(
      error("VIZE_MARQUETTE_113", "validUntil", "record expiry must come after run completion"),
    );
  }

  const runner = evidence.runner;
  checkIdentifier(runner.identity, "runner.identity", diagnostics);
  checkRetainedEvidence(
    runner.authenticationEvidence,
    "runner.authenticationEvidence",
    diagnostics,
  );
  checkDigest(runner.invocationFingerprint, "runner.invocationFingerprint", diagnostics);
  checkRetainedEvidence(runner.environmentEvidence, "runner.environmentEvidence", diagnostics);
  checkDigest(runner.environmentFingerprint, "runner.environmentFingerprint", diagnostics);

  const selection = evidence.selection;
  if (selection.targetIds.length === 0 || selection.targetIds.length > TEST_RUN_MAX_TARGETS) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_115",
        "selection.targetIds",
        "selection must include between 1 and 32 targets",
      ),
    );
  }
  if (selection.suiteIds.length === 0 || selection.suiteIds.length > TEST_RUN_MAX_SUITES) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_115",
        "selection.suiteIds",
        "selection must include between 1 and 512 suites",
      ),
    );
  }
  for (const id of selection.targetIds) {
    checkIdentifier(id, evidencePath("selection.targetIds", id), diagnostics);
  }
  for (const id of selection.suiteIds) {
    checkIdentifier(id, evidencePath("selection.suiteIds", id), diagnostics);
  }

  validateTargets(evidence, diagnostics);
  validateSuites(evidence, diagnostics);
  validateVerification(evidence, diagnostics);

  diagnostics.sort((left, right) =>
    left.path !== right.path
      ? left.path < right.path
        ? -1
        : 1
      : left.code !== right.code
        ? left.code < right.code
          ? -1
          : 1
        : left.message < right.message
          ? -1
          : left.message > right.message
            ? 1
            : 0,
  );
  return diagnostics;
}
