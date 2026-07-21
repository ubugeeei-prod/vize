/** Serialized `format` marker for test-run evidence records. */
export const TEST_RUN_EVIDENCE_FORMAT = "vize.test-run.evidence" as const;

/** Current serialized test-run evidence format version. */
export const TEST_RUN_EVIDENCE_FORMAT_VERSION = 1 as const;

/** Isolation level of the runner that executed the tests. */
export type TestRunIsolation = "dedicated" | "ephemeral";

/** Kind of user-visible target a test execution ran against. */
export type TestRunTargetKind = "web" | "native" | "desktop" | "terminal" | "server";

/** Semantic kind of one executed suite. */
export type TestRunSuiteKind =
  | "unit"
  | "integration"
  | "contract"
  | "end-to-end"
  | "accessibility"
  | "visual"
  | "performance"
  | "resilience"
  | "installation"
  | "upgrade"
  | "migration";

/** Final result of one executed suite shard. */
export type TestRunSuiteOutcome = "passed" | "failed" | "cancelled";

/** Final result of the independent verification pass. */
export type TestRunVerificationOutcome = "accepted" | "rejected";

/**
 * Immutable, content-addressed reference to retained evidence.
 *
 * The reference names the retained content by SHA-256 so a green label can
 * never drift away from the bytes that were verified.
 */
export interface TestRunRetainedEvidence {
  /** Content-addressed retrieval reference in `sha256:<64 hex>` form. */
  readonly reference: string;
  /** Lowercase SHA-256 fingerprint of the retained content. */
  readonly fingerprint: string;
}

/** Exact release artifact that the recorded test executions exercised. */
export interface TestRunArtifact {
  /** Stable artifact identifier. */
  readonly id: string;
  /** Lowercase SHA-256 fingerprint of the artifact bytes. */
  readonly fingerprint: string;
  /** Exact artifact size in bytes; must be at least one byte. */
  readonly sizeBytes: number;
}

/** Authenticated runner that executed the recorded test run. */
export interface TestRunRunner {
  /** Stable runner identity. */
  readonly identity: string;
  /** Retained evidence for the runner's authentication. */
  readonly authenticationEvidence: TestRunRetainedEvidence;
  /** Isolation level the runner guaranteed for this invocation. */
  readonly isolation: TestRunIsolation;
  /** Lowercase SHA-256 fingerprint of the exact invocation. */
  readonly invocationFingerprint: string;
  /** Retained evidence describing the execution environment. */
  readonly environmentEvidence: TestRunRetainedEvidence;
  /** Lowercase SHA-256 fingerprint of the execution environment. */
  readonly environmentFingerprint: string;
}

/**
 * Complete candidate-selected target and suite coverage for the run.
 *
 * Recorded executions must cover exactly these identifiers; anything less is
 * an undeclared omission and anything more is an undeclared execution.
 */
export interface TestRunSelection<
  TargetId extends string = string,
  SuiteId extends string = string,
> {
  /** Selected target identifiers, unique within the selection. */
  readonly targetIds: readonly TargetId[];
  /** Selected suite identifiers, unique within the selection. */
  readonly suiteIds: readonly SuiteId[];
}

/** One executed target of the release candidate. */
export interface TestRunTargetExecution<Id extends string = string> {
  /** Stable target identifier from the candidate selection. */
  readonly id: Id;
  /** Kind of target that was exercised. */
  readonly kind: TestRunTargetKind;
  /** Environment identifier the target executed in. */
  readonly environment: string;
}

/** One executed suite shard with its exact recorded results. */
export interface TestRunSuiteExecution<
  Id extends string = string,
  TargetId extends string = string,
> {
  /** Stable suite identifier from the candidate selection. */
  readonly id: Id;
  /** Target this shard executed against. */
  readonly targetId: TargetId;
  /** Semantic suite kind. */
  readonly kind: TestRunSuiteKind;
  /** One-based shard index within `shardCount`. */
  readonly shardIndex: number;
  /** Total number of shards the suite was split into. */
  readonly shardCount: number;
  /** Final shard outcome. */
  readonly outcome: TestRunSuiteOutcome;
  /** Number of passed tests. */
  readonly passed: number;
  /** Number of failed tests. */
  readonly failed: number;
  /** Number of skipped tests. */
  readonly skipped: number;
  /** Number of retried tests; retries must always be declared. */
  readonly retries: number;
  /** Wall-clock shard duration in milliseconds. */
  readonly durationMs: number;
  /** Lowercase SHA-256 fingerprint of the exact shard invocation. */
  readonly invocationFingerprint: string;
  /** Retained machine-readable report evidence. */
  readonly report: TestRunRetainedEvidence;
  /** Retained execution log evidence. */
  readonly log: TestRunRetainedEvidence;
}

/** Independent verification summary over every recorded execution. */
export interface TestRunVerification {
  /** Stable identity of the independent verifier. */
  readonly verifier: string;
  /** Millisecond-precision UTC time the verification completed. */
  readonly completedAt: string;
  /** Final verification outcome. */
  readonly outcome: TestRunVerificationOutcome;
  /** Exact number of recorded target executions. */
  readonly targetCount: number;
  /** Exact number of recorded suite executions. */
  readonly suiteCount: number;
  /** Total passed tests across every recorded suite execution. */
  readonly passed: number;
  /** Total failed tests across every recorded suite execution. */
  readonly failed: number;
  /** Total skipped tests across every recorded suite execution. */
  readonly skipped: number;
  /** Total retried tests across every recorded suite execution. */
  readonly retries: number;
  /** Retained evidence produced by the verification pass. */
  readonly evidence: TestRunRetainedEvidence;
}

/**
 * Complete, versioned test-run evidence for one release candidate.
 *
 * The record binds application, environment, application contract, source
 * revision, release, and exact artifact fingerprint to bounded target and
 * suite executions, so a `tests` deployment check can only be satisfied by
 * retained, immutable facts instead of a mutable label or path.
 */
export interface TestRunEvidence {
  /** Serialized format marker; always {@link TEST_RUN_EVIDENCE_FORMAT}. */
  readonly format: typeof TEST_RUN_EVIDENCE_FORMAT;
  /**
   * Serialized format version.
   *
   * @default 1
   */
  readonly formatVersion?: typeof TEST_RUN_EVIDENCE_FORMAT_VERSION;
  /** Stable record identifier. */
  readonly id: string;
  /** Application the run verified. */
  readonly application: string;
  /** Deployment environment the run verified the candidate for. */
  readonly environment: string;
  /** Lowercase SHA-256 fingerprint of the application contract. */
  readonly contractFingerprint: string;
  /** Exact source revision the candidate was built from. */
  readonly sourceRevision: string;
  /** Release the candidate belongs to. */
  readonly release: string;
  /** Exact artifact the recorded executions exercised. */
  readonly artifact: TestRunArtifact;
  /** Millisecond-precision UTC time the run started. */
  readonly startedAt: string;
  /** Millisecond-precision UTC time the run completed. */
  readonly completedAt: string;
  /** Millisecond-precision UTC time the record expires. */
  readonly validUntil: string;
  /** Authenticated runner that executed the run. */
  readonly runner: TestRunRunner;
  /** Complete candidate-selected target and suite coverage. */
  readonly selection: TestRunSelection;
  /** Recorded target executions. */
  readonly targets: readonly TestRunTargetExecution[];
  /** Recorded suite executions. */
  readonly suites: readonly TestRunSuiteExecution[];
  /** Independent verification summary. */
  readonly verification: TestRunVerification;
}

/** Target identifiers recorded by one test-run evidence record. */
export type TestRunTargetId<Evidence extends TestRunEvidence> = Evidence["targets"][number]["id"];

/** Suite identifiers recorded by one test-run evidence record. */
export type TestRunSuiteId<Evidence extends TestRunEvidence> = Evidence["suites"][number]["id"];
