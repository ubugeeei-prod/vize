import type { MarquetteDiagnostic } from "./validate.js";
import { testRunDenialCode } from "./test-run-admission.js";
import { parseTestRunAdmissionId } from "./test-run-canonical.js";
import {
  TEST_RUN_TRANSITION_MAX_ACCEPTED,
  type TestRunRetainedDiagnostic,
  type TestRunTransition,
} from "./test-run-transition-model.js";
import { error } from "./test-run-validate-rules.js";

/** Validates the accepted anti-replay state carried by one transition. */
export function validateAccepted(
  transition: TestRunTransition,
  diagnostics: MarquetteDiagnostic[],
): void {
  const accepted = transition.accepted;
  if (accepted.length > TEST_RUN_TRANSITION_MAX_ACCEPTED) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_131",
        "transition.accepted",
        "transition must accept at most 4096 admission ids",
      ),
    );
  }
  if (accepted.some((id) => parseTestRunAdmissionId(id) === undefined)) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_141",
        "transition.accepted",
        "accepted admission ids must be test-run: followed by 64 lowercase hexadecimal characters",
      ),
    );
  }
  if (accepted.some((id, index) => index > 0 && (accepted[index - 1] as string) >= id)) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_154",
        "transition.accepted",
        "accepted admission ids must be sorted and unique",
      ),
    );
  }
  if (transition.decision.allowed && !accepted.includes(transition.evidence)) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_156",
        "transition.accepted",
        "an allowed transition must accept its own evidence",
      ),
    );
  }
}

const DIAGNOSTIC_CODE = /^VIZE_MARQUETTE_[0-9]{3}$/;

/** Validates the retained decision carried by one transition. */
export function validateDecision(
  transition: TestRunTransition,
  diagnostics: MarquetteDiagnostic[],
): void {
  const decision = transition.decision;
  if (decision.allowed && (decision.denialCodes.length > 0 || decision.diagnostics.length > 0)) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_155",
        "transition.decision",
        "an allowed decision must carry no diagnostics",
      ),
    );
  }
  if (!decision.allowed && decision.diagnostics.length === 0) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_155",
        "transition.decision",
        "a denied decision must carry its diagnostics",
      ),
    );
  }
  if (decision.diagnostics.some((diagnostic) => !DIAGNOSTIC_CODE.test(diagnostic.code))) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_155",
        "transition.decision",
        "diagnostic codes must be stable VIZE_MARQUETTE codes",
      ),
    );
  }
  const sorted = decision.diagnostics.every((diagnostic, index) => {
    if (index === 0) {
      return true;
    }
    const left = decision.diagnostics[index - 1] as TestRunRetainedDiagnostic;
    return (
      left.path < diagnostic.path ||
      (left.path === diagnostic.path &&
        (left.code < diagnostic.code ||
          (left.code === diagnostic.code && left.message <= diagnostic.message)))
    );
  });
  if (!sorted) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_155",
        "transition.decision",
        "decision diagnostics must be sorted by path, code, and message",
      ),
    );
  }
  const recomputed = [
    ...new Set(
      decision.diagnostics.map((diagnostic) =>
        testRunDenialCode(diagnostic as MarquetteDiagnostic),
      ),
    ),
  ].sort();
  const retained = [...decision.denialCodes];
  if (
    recomputed.length !== retained.length ||
    recomputed.some((code, index) => code !== retained[index])
  ) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_155",
        "transition.decision",
        "denial codes must match the published diagnostic mapping",
      ),
    );
  }
}
