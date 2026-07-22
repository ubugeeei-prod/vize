import type { MarquetteDiagnostic } from "./validate.js";
import {
  testRunDenialCode,
  type TestRunAdmissionDecision,
  type TestRunDenialCode,
} from "./test-run-admission.js";
import { parseTestRunAdmissionId } from "./test-run-canonical.js";
import {
  TEST_RUN_TRANSITION_FORMAT,
  TEST_RUN_TRANSITION_FORMAT_VERSION,
  testRunTransitionFingerprint,
  type TestRunTransition,
} from "./test-run-transition-model.js";
import { validateAccepted, validateDecision } from "./test-run-transition-rules.js";
import {
  checkDigest,
  checkIdentifier,
  checkSafeInteger,
  checkSourceRevision,
  checkTimestamp,
  error,
  isStrictTimestamp,
} from "./test-run-validate-rules.js";

export {
  TEST_RUN_TRANSITION_FORMAT,
  TEST_RUN_TRANSITION_FORMAT_VERSION,
  TEST_RUN_TRANSITION_MAX_ACCEPTED,
  canonicalTestRunTransitionJson,
  testRunTransitionFingerprint,
  type TestRunRetainedDecision,
  type TestRunRetainedDiagnostic,
  type TestRunTransition,
} from "./test-run-transition-model.js";

/**
 * Validates one release transition structurally.
 *
 * Diagnostics use `transition.` paths and are deterministic and sorted by
 * path, code, and message. Validation confirms the record alone is
 * internally coherent — grammar, decision consistency against the published
 * diagnostic mapping, and an allowed decision accepting its own evidence —
 * but only {@link verifyTestRunTransition} can confirm the record extends
 * the durable chain. Codes, paths, messages, and ordering are identical to
 * the native implementation.
 */
export function validateTestRunTransition(transition: TestRunTransition): MarquetteDiagnostic[] {
  const diagnostics: MarquetteDiagnostic[] = [];

  if ((transition.format as string) !== TEST_RUN_TRANSITION_FORMAT) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_101",
        "transition.format",
        "unsupported release-transition format marker",
      ),
    );
  }
  const version = transition.formatVersion ?? TEST_RUN_TRANSITION_FORMAT_VERSION;
  if (version !== TEST_RUN_TRANSITION_FORMAT_VERSION) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_102",
        "transition.formatVersion",
        "unsupported release-transition format version",
      ),
    );
  }

  if (!Number.isInteger(transition.sequence) || transition.sequence < 1) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_152",
        "transition.sequence",
        "transition sequence must be at least one",
      ),
    );
  }
  checkSafeInteger(transition.sequence, "transition.sequence", diagnostics);
  if (transition.previous !== null && transition.sequence === 1) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_153",
        "transition.previous",
        "genesis transition must not name a predecessor",
      ),
    );
  } else if (transition.previous !== null) {
    checkDigest(transition.previous, "transition.previous", diagnostics);
  } else if (transition.sequence > 1) {
    diagnostics.push(
      error("VIZE_MARQUETTE_153", "transition.previous", "transition must name its predecessor"),
    );
  }
  checkTimestamp(transition.decidedAt, "transition.decidedAt", diagnostics);

  const candidate = transition.candidate;
  checkIdentifier(candidate.application, "transition.candidate.application", diagnostics);
  checkIdentifier(candidate.environment, "transition.candidate.environment", diagnostics);
  checkDigest(
    candidate.contractFingerprint,
    "transition.candidate.contractFingerprint",
    diagnostics,
  );
  checkSourceRevision(candidate.sourceRevision, "transition.candidate.sourceRevision", diagnostics);
  if (candidate.release.length === 0 || candidate.release.length > 256) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_106",
        "transition.candidate.release",
        "release must be between 1 and 256 characters",
      ),
    );
  }
  checkDigest(
    candidate.artifactFingerprint,
    "transition.candidate.artifactFingerprint",
    diagnostics,
  );

  if (parseTestRunAdmissionId(transition.evidence) === undefined) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_141",
        "transition.evidence",
        "transition evidence must be test-run: followed by 64 lowercase hexadecimal characters",
      ),
    );
  }
  validateAccepted(transition, diagnostics);
  validateDecision(transition, diagnostics);

  sortDiagnostics(diagnostics);
  return diagnostics;
}

/**
 * Verifies one release transition against the durable chain tip.
 *
 * `previous` is the retained, already-verified predecessor — `null` only
 * when deciding the very first transition of a chain. The transition must
 * validate structurally, extend the predecessor's sequence, fingerprint,
 * scope, and decision time exactly, never re-accept evidence the
 * predecessor already accepted, and carry an accepted state equal to the
 * predecessor's state plus exactly the newly accepted evidence (unchanged
 * for a denial). Any diagnostic rejects the transition: a conforming host
 * must not persist it, and on recovery must discard a tip this function
 * rejects. Diagnostics, denial codes, and ordering are identical to the
 * native implementation, as pinned by the shared transition-decision
 * fixtures.
 */
export async function verifyTestRunTransition(
  transition: TestRunTransition,
  previous: TestRunTransition | null,
): Promise<TestRunAdmissionDecision> {
  const diagnostics = validateTestRunTransition(transition);

  let priorAccepted: readonly string[] = [];
  if (previous === null) {
    if (transition.sequence !== 1) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_157",
          "transition.sequence",
          "transition requires its predecessor to verify",
        ),
      );
    }
  } else {
    priorAccepted = previous.accepted;
    if (transition.sequence !== previous.sequence + 1) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_157",
          "transition.sequence",
          "transition must extend its predecessor's sequence",
        ),
      );
    }
    if (transition.previous !== (await testRunTransitionFingerprint(previous))) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_157",
          "transition.previous",
          "transition must name its predecessor's canonical fingerprint",
        ),
      );
    }
    for (const [recorded, expected, path] of [
      [
        transition.candidate.application,
        previous.candidate.application,
        "transition.candidate.application",
      ],
      [
        transition.candidate.environment,
        previous.candidate.environment,
        "transition.candidate.environment",
      ],
    ] as const) {
      if (recorded !== expected) {
        diagnostics.push(
          error("VIZE_MARQUETTE_157", path, "transition must stay within its predecessor's scope"),
        );
      }
    }
    if (
      isStrictTimestamp(transition.decidedAt) &&
      isStrictTimestamp(previous.decidedAt) &&
      transition.decidedAt < previous.decidedAt
    ) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_157",
          "transition.decidedAt",
          "transition must not predate its predecessor",
        ),
      );
    }
    if (transition.decision.allowed && previous.accepted.includes(transition.evidence)) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_158",
          "transition.evidence",
          "accepted evidence must not be accepted again",
        ),
      );
    }
  }

  const expected = [
    ...new Set(
      transition.decision.allowed ? [...priorAccepted, transition.evidence] : priorAccepted,
    ),
  ].sort();
  const accepted = transition.accepted;
  if (accepted.length !== expected.length || accepted.some((id, index) => id !== expected[index])) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_159",
        "transition.accepted",
        "accepted state must equal its predecessor's state plus exactly the newly accepted evidence",
      ),
    );
  }

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
