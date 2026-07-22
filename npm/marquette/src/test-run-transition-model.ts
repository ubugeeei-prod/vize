import type { MarquetteDiagnosticSeverity } from "./validate.js";
import type { TestRunCandidate, TestRunDenialCode } from "./test-run-admission.js";

/**
 * Serialized `format` marker for release-transition records.
 *
 * Readers must reject any other value before trusting the record.
 */
export const TEST_RUN_TRANSITION_FORMAT = "vize.test-run.transition";

/**
 * Current serialized release-transition format.
 *
 * Readers must reject a higher value until they explicitly support it.
 */
export const TEST_RUN_TRANSITION_FORMAT_VERSION = 1;

/** Maximum admission ids one transition may carry as accepted state. */
export const TEST_RUN_TRANSITION_MAX_ACCEPTED = 4096;

/**
 * One retained diagnostic inside a durable transition record.
 *
 * The shape and serialization match live diagnostics exactly; the retained
 * form is plain data so persisted records can be read back by any host.
 */
export interface TestRunRetainedDiagnostic {
  /** Stable machine-readable diagnostic code. */
  readonly code: string;
  /** Severity recorded for the diagnostic; any diagnostic denies. */
  readonly severity: MarquetteDiagnosticSeverity;
  /** JSON-style path into the decided input. */
  readonly path: string;
  /** Human-readable explanation recorded with the decision. */
  readonly message: string;
}

/**
 * One retained allow-or-deny decision inside a durable transition record.
 *
 * The shape and serialization match live decisions exactly. Validation
 * rejects a retained decision whose `allowed` flag, denial codes, or
 * diagnostic ordering disagree with the published mapping, so a record
 * cannot claim an outcome its own diagnostics contradict.
 */
export interface TestRunRetainedDecision {
  /** Whether the release decision admitted the candidate. */
  readonly allowed: boolean;
  /** Deduplicated denial causes sorted lexicographically; empty if allowed. */
  readonly denialCodes: readonly TestRunDenialCode[];
  /** Complete diagnostics in the stable path, code, message order. */
  readonly diagnostics: readonly TestRunRetainedDiagnostic[];
}

/**
 * One durable atomic release transition.
 *
 * The record binds the decision, the exact candidate and evidence it
 * decided, and the complete accepted anti-replay state after the decision
 * into one canonical document. `sequence` grows by exactly one per
 * transition and `previous` names the predecessor's canonical SHA-256
 * fingerprint, so a chain tip proves the entire decision history and the
 * accepted set can never drift from the decision that produced it.
 *
 * Host durability contract: write the complete canonical bytes to a
 * temporary location, flush them to durable storage, then atomically rename
 * or commit so exactly one complete chain tip exists at every instant; on
 * recovery, verify the tip against its retained predecessor with
 * {@link verifyTestRunTransition} before deciding anything new, and discard
 * — never repair — a torn or partial record.
 */
export interface TestRunTransition {
  /** Serialized format marker; always {@link TEST_RUN_TRANSITION_FORMAT}. */
  readonly format: typeof TEST_RUN_TRANSITION_FORMAT;
  /**
   * Serialized format version.
   *
   * Defaults to {@link TEST_RUN_TRANSITION_FORMAT_VERSION}.
   */
  readonly formatVersion?: typeof TEST_RUN_TRANSITION_FORMAT_VERSION;
  /** One-based position of this transition in its chain. */
  readonly sequence: number;
  /** Canonical fingerprint of the predecessor; `null` only at genesis. */
  readonly previous: string | null;
  /** Millisecond-precision UTC instant the decision was made. */
  readonly decidedAt: string;
  /** Exact candidate the decision was made for. */
  readonly candidate: TestRunCandidate;
  /** Exact `test-run:<sha256>` admission id the decision evaluated. */
  readonly evidence: string;
  /** Retained decision exactly as it was produced. */
  readonly decision: TestRunRetainedDecision;
  /**
   * Complete anti-replay state after this transition: every admission id
   * ever accepted in this chain, sorted and unique.
   */
  readonly accepted: readonly string[];
}

/**
 * Serializes a release transition canonically.
 *
 * Property order matches the record schema and the accepted state sorts
 * lexicographically after deduplication, so equivalent transitions produce
 * byte-identical JSON in every language. These are the exact bytes a host
 * must write atomically and the exact bytes the chain fingerprint covers.
 * Call validation before trusting the record; canonicalization does not
 * make an invalid record valid.
 */
export function canonicalTestRunTransitionJson(transition: TestRunTransition): string {
  const canonical = {
    format: transition.format,
    formatVersion: transition.formatVersion ?? TEST_RUN_TRANSITION_FORMAT_VERSION,
    sequence: transition.sequence,
    previous: transition.previous,
    decidedAt: transition.decidedAt,
    candidate: {
      application: transition.candidate.application,
      environment: transition.candidate.environment,
      contractFingerprint: transition.candidate.contractFingerprint,
      sourceRevision: transition.candidate.sourceRevision,
      release: transition.candidate.release,
      artifactFingerprint: transition.candidate.artifactFingerprint,
    },
    evidence: transition.evidence,
    decision: {
      allowed: transition.decision.allowed,
      denialCodes: [...new Set(transition.decision.denialCodes)].sort(),
      diagnostics: transition.decision.diagnostics.map((diagnostic) => ({
        code: diagnostic.code,
        severity: diagnostic.severity,
        path: diagnostic.path,
        message: diagnostic.message,
      })),
    },
    accepted: [...new Set(transition.accepted)].sort(),
  };
  return JSON.stringify(canonical);
}

/**
 * Returns the lowercase SHA-256 fingerprint of the canonical transition.
 *
 * The fingerprint is the exact value the successor transition must name as
 * `previous`, forming the durable chain. Uses the Web Crypto API available
 * in every supported runtime.
 */
export async function testRunTransitionFingerprint(transition: TestRunTransition): Promise<string> {
  const bytes = new TextEncoder().encode(canonicalTestRunTransitionJson(transition));
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  let fingerprint = "";
  for (const byte of new Uint8Array(digest)) {
    fingerprint += byte.toString(16).padStart(2, "0");
  }
  return fingerprint;
}
