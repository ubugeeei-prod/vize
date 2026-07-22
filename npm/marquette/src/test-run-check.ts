import type { TestRunEvidence } from "./test-run-model.js";
import type { MarquetteDiagnostic } from "./validate.js";
import {
  admitTestRun,
  testRunDenialCode,
  type TestRunAdmissionDecision,
  type TestRunCandidate,
  type TestRunDenialCode,
} from "./test-run-admission.js";
import { parseTestRunAdmissionId } from "./test-run-canonical.js";
import {
  checkDigest,
  checkIdentifier,
  checkSourceRevision,
  checkTimestamp,
  error,
  isStrictTimestamp,
} from "./test-run-validate-rules.js";

/**
 * Serialized `format` marker for retained tests-check records.
 *
 * Readers must reject any other value before trusting the record.
 */
export const TEST_RUN_CHECK_FORMAT = "vize.test-run.check";

/**
 * Current serialized tests-check format.
 *
 * Readers must reject a higher value until they explicitly support it.
 */
export const TEST_RUN_CHECK_FORMAT_VERSION = 1;

/**
 * Retained, release-bound `tests` check for one deployment decision.
 *
 * The record replaces every generic test-result reference — a summary blob,
 * a report path, or a green workflow label — with the exact
 * `test-run:<sha256>` admission id of an independently verified run, the six
 * candidate facts the run was admitted for, and the identity and instant of
 * the independent observer that recorded the admission. A release decision
 * retaining anything else as its tests evidence cannot pass
 * {@link verifyTestRunCheck}.
 */
export interface TestRunCheck {
  /** Serialized format marker; always {@link TEST_RUN_CHECK_FORMAT}. */
  readonly format: typeof TEST_RUN_CHECK_FORMAT;
  /**
   * Serialized format version.
   *
   * Defaults to {@link TEST_RUN_CHECK_FORMAT_VERSION}.
   */
  readonly formatVersion?: typeof TEST_RUN_CHECK_FORMAT_VERSION;
  /** Exact `test-run:<sha256>` admission id of the observed run. */
  readonly evidence: string;
  /** Exact candidate facts the run was admitted for. */
  readonly candidate: TestRunCandidate;
  /**
   * Identity of the independent observer that recorded the admission.
   *
   * The observer is the trusted promotion boundary, never the runner that
   * executed the tests.
   */
  readonly observer: string;
  /** Millisecond-precision UTC instant the admission was observed. */
  readonly observedAt: string;
}

/**
 * Validates a retained tests-check record structurally.
 *
 * Diagnostics use `check.` paths and are deterministic and sorted by path,
 * code, and message. A generic evidence reference fails here with
 * `VIZE_MARQUETTE_141`: only an exact `test-run:<sha256>` admission id can
 * name retained test evidence. Structural validity never admits anything by
 * itself; {@link verifyTestRunCheck} must confirm the record against the
 * caller's candidate and the retained run. Codes, paths, messages, and
 * ordering are identical to the native implementation.
 */
export function validateTestRunCheck(check: TestRunCheck): MarquetteDiagnostic[] {
  const diagnostics: MarquetteDiagnostic[] = [];

  if ((check.format as string) !== TEST_RUN_CHECK_FORMAT) {
    diagnostics.push(
      error("VIZE_MARQUETTE_101", "check.format", "unsupported tests-check format marker"),
    );
  }
  if ((check.formatVersion ?? TEST_RUN_CHECK_FORMAT_VERSION) !== TEST_RUN_CHECK_FORMAT_VERSION) {
    diagnostics.push(
      error("VIZE_MARQUETTE_102", "check.formatVersion", "unsupported tests-check format version"),
    );
  }

  if (parseTestRunAdmissionId(check.evidence) === undefined) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_141",
        "check.evidence",
        "check evidence must be test-run: followed by 64 lowercase hexadecimal characters",
      ),
    );
  }

  const candidate = check.candidate;
  checkIdentifier(candidate.application, "check.candidate.application", diagnostics);
  checkIdentifier(candidate.environment, "check.candidate.environment", diagnostics);
  checkDigest(candidate.contractFingerprint, "check.candidate.contractFingerprint", diagnostics);
  checkSourceRevision(candidate.sourceRevision, "check.candidate.sourceRevision", diagnostics);
  if (candidate.release.length === 0 || candidate.release.length > 256) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_106",
        "check.candidate.release",
        "release must be between 1 and 256 characters",
      ),
    );
  }
  checkDigest(candidate.artifactFingerprint, "check.candidate.artifactFingerprint", diagnostics);

  checkIdentifier(check.observer, "check.observer", diagnostics);
  checkTimestamp(check.observedAt, "check.observedAt", diagnostics);

  sortDiagnostics(diagnostics);
  return diagnostics;
}

/**
 * Verifies one retained tests check against the caller's own facts.
 *
 * The caller supplies the candidate it is deciding from its own trusted
 * facts; the retained check must validate structurally, bind that candidate
 * exactly, name an observer independent from the run's runner, and be
 * observed no earlier than the run's completed verification. The referenced
 * record is then admitted exactly like {@link admitTestRun}: canonical
 * fingerprint, candidate bindings, expiry at `now`, verification outcome,
 * and skipped-test accounting all fail closed. Diagnostics, denial codes,
 * and ordering are identical to the native implementation, as pinned by the
 * shared check-decision fixtures.
 */
export async function verifyTestRunCheck(
  check: TestRunCheck,
  candidate: TestRunCandidate,
  evidence: TestRunEvidence,
  now: string,
): Promise<TestRunAdmissionDecision> {
  const diagnostics = validateTestRunCheck(check);

  const bindings = [
    [check.candidate.application, candidate.application, "application", "application"],
    [check.candidate.environment, candidate.environment, "environment", "environment"],
    [
      check.candidate.contractFingerprint,
      candidate.contractFingerprint,
      "contractFingerprint",
      "contract fingerprint",
    ],
    [check.candidate.sourceRevision, candidate.sourceRevision, "sourceRevision", "source revision"],
    [check.candidate.release, candidate.release, "release", "release"],
    [
      check.candidate.artifactFingerprint,
      candidate.artifactFingerprint,
      "artifactFingerprint",
      "artifact fingerprint",
    ],
  ] as const;
  for (const [recorded, expected, property, field] of bindings) {
    if (recorded !== expected) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_149",
          `check.candidate.${property}`,
          `check does not bind the candidate ${field}`,
        ),
      );
    }
  }

  if (check.observer === evidence.runner.identity) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_151",
        "check.observer",
        "check observer must be independent from the run's runner",
      ),
    );
  }
  if (isStrictTimestamp(check.observedAt) && check.observedAt < evidence.verification.completedAt) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_150",
        "check.observedAt",
        "observation must not precede the completed verification",
      ),
    );
  }

  diagnostics.push(...(await admitTestRun(evidence, candidate, check.evidence, now)));
  sortDiagnostics(diagnostics);
  const denialCodes: TestRunDenialCode[] = [...new Set(diagnostics.map(testRunDenialCode))].sort();
  return { allowed: diagnostics.length === 0, denialCodes, diagnostics };
}

function sortDiagnostics(diagnostics: MarquetteDiagnostic[]): void {
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
}
