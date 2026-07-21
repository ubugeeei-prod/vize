import type { TestRunEvidence } from "./test-run-model.js";
import type { MarquetteDiagnostic } from "./validate.js";
import { parseTestRunAdmissionId, testRunFingerprint } from "./test-run-canonical.js";
import { validateTestRunEvidence } from "./test-run-validate.js";
import { error, isStrictTimestamp } from "./test-run-validate-rules.js";

export { TEST_RUN_ADMISSION_PREFIX, parseTestRunAdmissionId } from "./test-run-canonical.js";

/**
 * Exact release candidate a deployment gate wants evidence for.
 *
 * Every field must match the record exactly; admission never falls back to a
 * newer, older, or partially matching record.
 */
export interface TestRunCandidate {
  /** Application the gate is deploying. */
  readonly application: string;
  /** Deployment environment the gate is promoting into. */
  readonly environment: string;
  /** Lowercase SHA-256 fingerprint of the application contract. */
  readonly contractFingerprint: string;
  /** Exact source revision of the candidate. */
  readonly sourceRevision: string;
  /** Release the candidate belongs to. */
  readonly release: string;
  /** Lowercase SHA-256 fingerprint of the exact artifact being promoted. */
  readonly artifactFingerprint: string;
}

/**
 * Decides whether one record admits one exact candidate at one instant.
 *
 * An empty result admits the deployment. Any diagnostic rejects it: the
 * record must validate cleanly, its canonical fingerprint must be the one
 * named by `admissionId`, every candidate binding must match exactly, the
 * record must not be expired at `now`, the independent verification must
 * have accepted the run, and no skipped test may remain unaccounted for.
 *
 * Codes, paths, messages, and ordering are identical to the native
 * implementation, so both host families reach the same decision for the
 * same record.
 *
 * `now` must be a millisecond-precision UTC timestamp such as
 * `2026-01-01T00:00:00.000Z`; the fixed-width format keeps the expiry
 * comparison exact. Callers own retrieval: fetch the canonical bytes from an
 * immutable store within their own deadline, refuse oversized content, and
 * hand the parsed record here.
 */
export async function admitTestRun(
  evidence: TestRunEvidence,
  candidate: TestRunCandidate,
  admissionId: string,
  now: string,
): Promise<MarquetteDiagnostic[]> {
  const diagnostics = validateTestRunEvidence(evidence);

  const nowIsExact = isStrictTimestamp(now);
  if (!nowIsExact) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_148",
        "admission.now",
        "admission time must be a millisecond-precision UTC instant",
      ),
    );
  }

  const expected = parseTestRunAdmissionId(admissionId);
  if (expected === undefined) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_141",
        "admission.id",
        "admission id must be test-run: followed by 64 lowercase hexadecimal characters",
      ),
    );
  } else if ((await testRunFingerprint(evidence)) !== expected) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_142",
        "admission.id",
        "admission id does not name this record's canonical fingerprint",
      ),
    );
  }

  const bindings = [
    [evidence.application, candidate.application, "application"],
    [evidence.environment, candidate.environment, "environment"],
    [evidence.contractFingerprint, candidate.contractFingerprint, "contractFingerprint"],
    [evidence.sourceRevision, candidate.sourceRevision, "sourceRevision"],
    [evidence.release, candidate.release, "release"],
    [evidence.artifact.fingerprint, candidate.artifactFingerprint, "artifact.fingerprint"],
  ] as const;
  for (const [recorded, wanted, field] of bindings) {
    if (recorded !== wanted) {
      diagnostics.push(
        error("VIZE_MARQUETTE_144", field, `record does not bind the candidate ${field}`),
      );
    }
  }

  if (nowIsExact && evidence.validUntil <= now) {
    diagnostics.push(
      error("VIZE_MARQUETTE_145", "validUntil", "record is expired at the admission time"),
    );
  }
  if (evidence.verification.outcome !== "accepted") {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_146",
        "verification.outcome",
        "only an accepted verification can admit a deployment",
      ),
    );
  }
  if (evidence.verification.skipped > 0) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_147",
        "verification.skipped",
        "skipped tests are not approved for deployment admission",
      ),
    );
  }

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
