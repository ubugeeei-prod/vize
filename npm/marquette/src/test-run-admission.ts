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

/**
 * Every denial code, in the stable lexicographic decision order.
 *
 * The vocabulary is shared by every backend family: a JavaScript, Rust, Go,
 * or JVM host must derive the same codes from the same diagnostics, as
 * pinned by the shared `tests/fixtures/test-run-evidence` decision fixtures.
 * Codes are append-only: they are never renamed, renumbered, reused, or
 * removed, and a new rejection cause always ships with a new code.
 */
export const TEST_RUN_DENIAL_CODES = [
  "admission-id-malformed",
  "admission-id-mismatch",
  "admission-time-malformed",
  "candidate-application-mismatch",
  "candidate-artifact-fingerprint-mismatch",
  "candidate-contract-fingerprint-mismatch",
  "candidate-environment-mismatch",
  "candidate-release-mismatch",
  "candidate-source-revision-mismatch",
  "check-candidate-mismatch",
  "check-invalid",
  "check-observer-not-independent",
  "record-expired",
  "record-invalid",
  "skipped-tests-recorded",
  "transition-chain-broken",
  "transition-invalid",
  "transition-replayed",
  "transition-state-mismatch",
  "verification-not-accepted",
] as const;

/** Stable machine-readable cause class of one admission denial. */
export type TestRunDenialCode = (typeof TEST_RUN_DENIAL_CODES)[number];

/**
 * Structured allow-or-deny admission decision for one exact candidate.
 *
 * The decision carries the machine-readable cause classes next to the exact
 * diagnostics, so a deployment gate in any language can act on one bounded
 * vocabulary while operators keep the full explanation. Serialization
 * follows the shared `test-run-admission` schema; decisions are outputs, so
 * a gate must never trust a decision it did not compute itself.
 */
export interface TestRunAdmissionDecision {
  /** Whether the record admits the candidate; true only with no diagnostics. */
  readonly allowed: boolean;
  /** Deduplicated denial causes sorted lexicographically; empty when allowed. */
  readonly denialCodes: readonly TestRunDenialCode[];
  /** Complete diagnostics in the stable path, code, message order. */
  readonly diagnostics: readonly MarquetteDiagnostic[];
}

const CANDIDATE_MISMATCH_CODES: ReadonlyMap<string, TestRunDenialCode> = new Map([
  ["application", "candidate-application-mismatch"],
  ["artifact.fingerprint", "candidate-artifact-fingerprint-mismatch"],
  ["contractFingerprint", "candidate-contract-fingerprint-mismatch"],
  ["environment", "candidate-environment-mismatch"],
  ["release", "candidate-release-mismatch"],
  ["sourceRevision", "candidate-source-revision-mismatch"],
]);

/**
 * Returns the stable denial code one admission diagnostic maps to.
 *
 * The mapping is total and identical in every host family: admission codes
 * `VIZE_MARQUETTE_141` through `VIZE_MARQUETTE_148` map to their exact
 * cause, `VIZE_MARQUETTE_144` distinguishes the mismatched candidate binding
 * by its diagnostic path, `VIZE_MARQUETTE_149` through `VIZE_MARQUETTE_151`
 * map to their tests-check cause, `VIZE_MARQUETTE_156` through
 * `VIZE_MARQUETTE_159` map to their transition cause, every other
 * diagnostic at a `check.` path is a `check-invalid` tests-check validation
 * failure, every other diagnostic at a `transition.` path is a
 * `transition-invalid` transition validation failure, and every remaining
 * diagnostic is a `record-invalid` record-validation failure.
 */
export function testRunDenialCode(diagnostic: MarquetteDiagnostic): TestRunDenialCode {
  switch (diagnostic.code) {
    case "VIZE_MARQUETTE_141":
      return "admission-id-malformed";
    case "VIZE_MARQUETTE_142":
      return "admission-id-mismatch";
    case "VIZE_MARQUETTE_144": {
      const mismatch = CANDIDATE_MISMATCH_CODES.get(diagnostic.path);
      if (mismatch !== undefined) {
        return mismatch;
      }
      break;
    }
    case "VIZE_MARQUETTE_145":
      return "record-expired";
    case "VIZE_MARQUETTE_146":
      return "verification-not-accepted";
    case "VIZE_MARQUETTE_147":
      return "skipped-tests-recorded";
    case "VIZE_MARQUETTE_148":
      return "admission-time-malformed";
    case "VIZE_MARQUETTE_149":
      return "check-candidate-mismatch";
    case "VIZE_MARQUETTE_150":
      return "check-invalid";
    case "VIZE_MARQUETTE_151":
      return "check-observer-not-independent";
    case "VIZE_MARQUETTE_156":
      return "transition-state-mismatch";
    case "VIZE_MARQUETTE_157":
      return "transition-chain-broken";
    case "VIZE_MARQUETTE_158":
      return "transition-replayed";
    case "VIZE_MARQUETTE_159":
      return "transition-state-mismatch";
    default:
      break;
  }
  if (diagnostic.path.startsWith("check.")) {
    return "check-invalid";
  }
  return diagnostic.path.startsWith("transition.") ? "transition-invalid" : "record-invalid";
}

/**
 * Decides one candidate and returns the structured admission decision.
 *
 * The decision wraps {@link admitTestRun}: `diagnostics` is exactly its
 * result, `denialCodes` maps every diagnostic through
 * {@link testRunDenialCode} and then deduplicates and sorts the codes
 * lexicographically, and `allowed` is true only when both are empty. Codes,
 * ordering, and diagnostics are identical to the native implementation, as
 * pinned by the shared decision fixtures. Inputs carry the same obligations
 * as {@link admitTestRun}.
 */
export async function decideTestRunAdmission(
  evidence: TestRunEvidence,
  candidate: TestRunCandidate,
  admissionId: string,
  now: string,
): Promise<TestRunAdmissionDecision> {
  const diagnostics = await admitTestRun(evidence, candidate, admissionId, now);
  const denialCodes = [...new Set(diagnostics.map(testRunDenialCode))].sort();
  return { allowed: diagnostics.length === 0, denialCodes, diagnostics };
}
