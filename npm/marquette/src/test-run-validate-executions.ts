import type { MarquetteDiagnostic } from "./validate.js";
import type { TestRunEvidence, TestRunSuiteExecution } from "./test-run-model.js";
import {
  checkDigest,
  checkIdentifier,
  checkRetainedEvidence,
  checkSafeInteger,
  checkTimestamp,
  error,
  evidencePath,
} from "./test-run-validate-rules.js";

/** Maximum recorded target executions and selected target identifiers. */
export const TEST_RUN_MAX_TARGETS = 32;

/** Maximum recorded suite executions and selected suite identifiers. */
export const TEST_RUN_MAX_SUITES = 512;

/** Maximum shard index and shard count for one suite execution. */
export const TEST_RUN_MAX_SHARDS = 1024;

/** Validates recorded target executions against the candidate selection. */
export function validateTargets(
  evidence: TestRunEvidence,
  diagnostics: MarquetteDiagnostic[],
): void {
  if (evidence.targets.length === 0 || evidence.targets.length > TEST_RUN_MAX_TARGETS) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_131",
        "targets",
        "record must include between 1 and 32 target executions",
      ),
    );
  }

  const seen = new Set<string>();
  const selected = new Set(evidence.selection.targetIds);
  for (const target of evidence.targets) {
    const path = evidencePath("targets", target.id);
    checkIdentifier(target.id, path, diagnostics);
    checkIdentifier(target.environment, evidencePath(path, "environment"), diagnostics);
    if (seen.has(target.id)) {
      diagnostics.push(
        error("VIZE_MARQUETTE_117", path, "target execution is recorded more than once"),
      );
    }
    seen.add(target.id);
    if (!selected.has(target.id)) {
      diagnostics.push(
        error("VIZE_MARQUETTE_118", path, "target execution was not selected for this candidate"),
      );
    }
  }
  for (const id of evidence.selection.targetIds) {
    if (!seen.has(id)) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_119",
          evidencePath("selection.targetIds", id),
          "selected target has no recorded execution",
        ),
      );
    }
  }
}

interface ShardGroup {
  readonly shardCount: number;
  readonly kind: TestRunSuiteExecution["kind"];
  readonly targetId: string;
  readonly indexes: Set<number>;
  kindsDisagree: boolean;
  targetsDisagree: boolean;
  countsDisagree: boolean;
}

/** Validates recorded suite executions, shards, and their consistency. */
export function validateSuites(
  evidence: TestRunEvidence,
  diagnostics: MarquetteDiagnostic[],
): void {
  if (evidence.suites.length === 0 || evidence.suites.length > TEST_RUN_MAX_SUITES) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_131",
        "suites",
        "record must include between 1 and 512 suite executions",
      ),
    );
  }

  const shards = new Map<string, ShardGroup>();
  const selected = new Set(evidence.selection.suiteIds);
  const recordedTargets = new Set(evidence.targets.map((target) => target.id));
  for (const suite of evidence.suites) {
    const path = evidencePath("suites", suite.id);
    let group = shards.get(suite.id);
    if (group === undefined) {
      group = {
        shardCount: suite.shardCount,
        kind: suite.kind,
        targetId: suite.targetId,
        indexes: new Set(),
        kindsDisagree: false,
        targetsDisagree: false,
        countsDisagree: false,
      };
      shards.set(suite.id, group);
      // Suite-id-scoped invariants are shard-independent, so evaluate them
      // once per unique suite id. Running them per shard would emit the same
      // code/path/message diagnostic once for every shard of the suite.
      checkIdentifier(suite.id, path, diagnostics);
      if (!selected.has(suite.id)) {
        diagnostics.push(
          error("VIZE_MARQUETTE_121", path, "suite execution was not selected for this candidate"),
        );
      }
      if (!recordedTargets.has(suite.targetId)) {
        diagnostics.push(
          error(
            "VIZE_MARQUETTE_123",
            evidencePath(path, "targetId"),
            "suite target has no recorded target execution",
          ),
        );
      }
    }
    group.kindsDisagree ||= group.kind !== suite.kind;
    group.targetsDisagree ||= group.targetId !== suite.targetId;
    group.countsDisagree ||= group.shardCount !== suite.shardCount;
    if (group.indexes.has(suite.shardIndex)) {
      diagnostics.push(error("VIZE_MARQUETTE_120", path, "suite shard is recorded more than once"));
    }
    group.indexes.add(suite.shardIndex);
    if (
      suite.shardCount === 0 ||
      suite.shardCount > TEST_RUN_MAX_SHARDS ||
      suite.shardIndex === 0 ||
      suite.shardIndex > suite.shardCount
    ) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_124",
          evidencePath(path, "shardIndex"),
          "shard index must fall within a shard count between 1 and 1024",
        ),
      );
    }
    checkSafeInteger(suite.durationMs, evidencePath(path, "durationMs"), diagnostics);
    checkDigest(
      suite.invocationFingerprint,
      evidencePath(path, "invocationFingerprint"),
      diagnostics,
    );
    checkRetainedEvidence(suite.report, evidencePath(path, "report"), diagnostics);
    checkRetainedEvidence(suite.log, evidencePath(path, "log"), diagnostics);

    const executed = suite.passed + suite.failed + suite.skipped;
    const consistent =
      suite.outcome === "passed"
        ? suite.failed === 0 && executed > 0
        : suite.outcome === "failed"
          ? suite.failed > 0
          : true;
    if (!consistent) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_130",
          evidencePath(path, "outcome"),
          "suite outcome does not match its recorded counts",
        ),
      );
    }
  }

  for (const id of evidence.selection.suiteIds) {
    if (!shards.has(id)) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_122",
          evidencePath("selection.suiteIds", id),
          "selected suite has no recorded execution",
        ),
      );
    }
  }

  for (const [id, group] of shards) {
    const path = evidencePath("suites", id);
    if (group.kindsDisagree || group.targetsDisagree || group.countsDisagree) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_126",
          path,
          "every shard of one suite must share its kind, target, and shard count",
        ),
      );
    } else if (
      group.shardCount !== 0 &&
      group.shardCount <= TEST_RUN_MAX_SHARDS &&
      group.indexes.size !== group.shardCount
    ) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_125",
          path,
          `suite must record every shard from 1 to ${group.shardCount}`,
        ),
      );
    }
  }
}

/** Validates the independent verification summary. */
export function validateVerification(
  evidence: TestRunEvidence,
  diagnostics: MarquetteDiagnostic[],
): void {
  const verification = evidence.verification;
  checkIdentifier(verification.verifier, "verification.verifier", diagnostics);
  checkTimestamp(verification.completedAt, "verification.completedAt", diagnostics);
  checkRetainedEvidence(verification.evidence, "verification.evidence", diagnostics);
  if (verification.completedAt < evidence.completedAt) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_114",
        "verification.completedAt",
        "verification must complete after the run completes",
      ),
    );
  }
  if (verification.targetCount !== evidence.targets.length) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_127",
        "verification.targetCount",
        "verified target count must equal the recorded target executions",
      ),
    );
  }
  if (verification.suiteCount !== evidence.suites.length) {
    diagnostics.push(
      error(
        "VIZE_MARQUETTE_127",
        "verification.suiteCount",
        "verified suite count must equal the recorded suite executions",
      ),
    );
  }

  const totals = { passed: 0, failed: 0, skipped: 0, retries: 0 };
  for (const suite of evidence.suites) {
    totals.passed += suite.passed;
    totals.failed += suite.failed;
    totals.skipped += suite.skipped;
    totals.retries += suite.retries;
  }
  const recorded = [
    [totals.passed, verification.passed, "verification.passed", "passed"],
    [totals.failed, verification.failed, "verification.failed", "failed"],
    [totals.skipped, verification.skipped, "verification.skipped", "skipped"],
    [totals.retries, verification.retries, "verification.retries", "retried"],
  ] as const;
  for (const [total, value, path, name] of recorded) {
    if (total !== value) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_128",
          path,
          `verified ${name} total must equal the sum over suite executions`,
        ),
      );
    }
  }

  if (verification.outcome === "accepted") {
    const clean = evidence.suites.every(
      (suite) => suite.outcome === "passed" && suite.failed === 0,
    );
    if (!clean || verification.failed > 0) {
      diagnostics.push(
        error(
          "VIZE_MARQUETTE_129",
          "verification.outcome",
          "verification cannot accept a run with failed or cancelled executions",
        ),
      );
    }
  }
}
