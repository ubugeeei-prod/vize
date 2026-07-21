import type {
  TestRunEvidence,
  TestRunSuiteExecution,
  TestRunSuiteId,
  TestRunTargetId,
} from "./test-run-model.js";

export {
  TEST_RUN_EVIDENCE_FORMAT,
  TEST_RUN_EVIDENCE_FORMAT_VERSION,
  type TestRunArtifact,
  type TestRunEvidence,
  type TestRunIsolation,
  type TestRunRetainedEvidence,
  type TestRunRunner,
  type TestRunSelection,
  type TestRunSuiteExecution,
  type TestRunSuiteId,
  type TestRunSuiteKind,
  type TestRunSuiteOutcome,
  type TestRunTargetExecution,
  type TestRunTargetId,
  type TestRunTargetKind,
  type TestRunVerification,
  type TestRunVerificationOutcome,
} from "./test-run-model.js";

/**
 * Cross-reference constraints derived from the literals in one record.
 *
 * Keeping this type separate makes editor diagnostics point at the authored
 * reference instead of widening every identifier to `string`.
 */
export type TestRunReferenceConstraints<Evidence extends TestRunEvidence> = {
  readonly selection?: {
    readonly targetIds: readonly TestRunTargetId<Evidence>[];
    readonly suiteIds: readonly TestRunSuiteId<Evidence>[];
  };
  readonly suites?: readonly (TestRunSuiteExecution<string, TestRunTargetId<Evidence>> & {
    readonly targetId: TestRunTargetId<Evidence>;
  })[];
};

/**
 * Defines a test-run evidence record while preserving every authored literal.
 *
 * The candidate selection and every suite's target reference are checked
 * against identifiers recorded in the same object. The function returns its
 * input without allocation or runtime work; use the `/test-run/validate`
 * entry before trusting a record.
 *
 * @example
 * ```ts
 * const evidence = defineTestRunEvidence({
 *   format: "vize.test-run.evidence",
 *   id: "run-1",
 *   application: "shop",
 *   // ...bindings, runner, selection, targets, suites, verification
 * });
 * ```
 */
export function defineTestRunEvidence<const Evidence extends TestRunEvidence>(
  evidence: Evidence & TestRunReferenceConstraints<NoInfer<Evidence>>,
): Evidence {
  return evidence;
}
