import assert from "node:assert/strict";
import { setTimeout as sleep } from "node:timers/promises";
import type { LspSession } from "../../../../../tests/tooling/support/lsp/session.ts";
import {
  auditCancellationPublications,
  sourceSha256,
  type CancellationPublication,
  type CancellationPublicationRecorder,
} from "./lsp-resource-cancellation.ts";
import type { ResourceCorsaGate } from "./lsp-resource-corsa-gate.ts";
import { mib, type ProcessEntry, type Sampler, type TreeSample } from "./lsp-resource-sampler.ts";
import { cancelledConcurrentEdits, verifyCancellationRecovery } from "./lsp-resource-witness.ts";
import { componentSource, documentUri } from "./lsp-resource-workspace.ts";

interface ResourceCheckpoint {
  monotonic_ns: string;
  server_rss_bytes: number;
  server_rss_mib: number;
  process_tree_rss_bytes: number;
  process_tree_rss_mib: number;
  processes: ProcessEntry[];
  cohort: Array<{ pid: number; birth_ticks: number; executable: string }>;
}

export interface AllocationProfileLink {
  same_session: boolean;
  expected_path: string | null;
  actual_path?: string | null;
  path_matches?: boolean;
  persistence_error?: string;
  receipt_field: "profile_json";
  scope: string;
}

export interface CancellationPublicationProgress {
  phase:
    | "baseline-recovery"
    | "baseline-quiescence"
    | "cancellation"
    | "final-recovery"
    | "final-quiescence"
    | "complete";
  complete: boolean;
  recovery_boundary_received_order: number;
  clean_fence_received_order: number | null;
  observed_count: number;
  rows: CancellationPublication[];
}

export const createCancellationPublicationProgress = (): CancellationPublicationProgress => ({
  phase: "baseline-recovery",
  complete: false,
  recovery_boundary_received_order: 0,
  clean_fence_received_order: null,
  observed_count: 0,
  rows: [],
});

export function updateCancellationPublicationProgress(
  progress: CancellationPublicationProgress,
  phase: CancellationPublicationProgress["phase"],
  recoveryBoundary: number,
  cleanFence: number | null,
  rows: readonly CancellationPublication[],
): void {
  assert.ok(Number.isSafeInteger(recoveryBoundary) && recoveryBoundary >= 0);
  assert.ok(cleanFence === null || (Number.isSafeInteger(cleanFence) && cleanFence >= 0));
  Object.assign(progress, {
    phase,
    complete: phase === "complete",
    recovery_boundary_received_order: recoveryBoundary,
    clean_fence_received_order: cleanFence,
    observed_count: rows.length,
    rows: [...rows],
  });
}

/** Fold profile linkage into evidence without replacing an earlier failure. */
export function finalizeCancellationProfileLink(
  link: AllocationProfileLink,
  actualPath: string | null,
  primaryFailure: boolean,
  persist: () => void = () => undefined,
): void {
  link.actual_path = actualPath;
  link.path_matches = actualPath === link.expected_path;
  try {
    persist();
  } catch (error) {
    link.persistence_error = String(error);
    if (!primaryFailure) throw error;
  }
  if (!primaryFailure) {
    assert.ok(link.path_matches, "the allocation profile must belong to this exact session");
  }
}

const checkpoint = (sample: TreeSample, serverPid: number): ResourceCheckpoint => {
  const server = sample.processes.find((process) => process.pid === serverPid);
  assert.ok(server, "the measured Maestro server must own each RSS checkpoint");
  assert.ok(
    sample.processes.every((process) => process.executable !== null),
    "every quiescent RSS process must retain an executable identity",
  );
  return {
    monotonic_ns: process.hrtime.bigint().toString(),
    server_rss_bytes: server.rss_bytes,
    server_rss_mib: mib(server.rss_bytes),
    process_tree_rss_bytes: sample.rss_bytes,
    process_tree_rss_mib: mib(sample.rss_bytes),
    processes: sample.processes,
    cohort: sample.processes
      .map((process) => ({
        pid: process.pid,
        birth_ticks: process.birth_ticks,
        executable: process.executable!,
      }))
      .sort((left, right) => left.pid - right.pid || left.birth_ticks - right.birth_ticks),
  };
};

const assertSameCohort = (
  expected: ResourceCheckpoint,
  actual: ResourceCheckpoint,
  label: string,
): void => {
  assert.deepEqual(
    actual.cohort,
    expected.cohort,
    `${label} RSS is invalid unless server and complete descendant PID/birth/exe cohort matches`,
  );
};

/** Round only the raw-byte delta, never the independently displayed endpoints. */
export const rssDeltaMib = (afterBytes: number, beforeBytes: number): number => {
  assert.ok(Number.isSafeInteger(afterBytes) && afterBytes >= 0, "valid after RSS bytes required");
  assert.ok(
    Number.isSafeInteger(beforeBytes) && beforeBytes >= 0,
    "valid before RSS bytes required",
  );
  return Math.round(((afterBytes - beforeBytes) / 1024 / 1024) * 10) / 10;
};

/**
 * Measure only the deterministic cancellation window. The recorder and its
 * boundary must precede body-edit churn so even an already-backlogged final
 * publication has a positive order. Both retained-RSS endpoints run the same
 * recovery and 10-second quiet fence; every process identity survives.
 */
export async function measureCancellationRetention(options: {
  session: LspSession;
  sampler: Pick<Sampler, "sample">;
  workspace: string;
  initialVersion: number;
  gate: ResourceCorsaGate;
  profilePath: string | null;
  publications: CancellationPublicationRecorder;
  baselineRecoveryStart: number;
  publicationProgress: CancellationPublicationProgress;
  persistPublicationProgress: () => void;
}) {
  const publications = options.publications;
  assert.ok(
    Number.isSafeInteger(options.baselineRecoveryStart) &&
      options.baselineRecoveryStart >= 0 &&
      options.baselineRecoveryStart <= publications.count(),
    "the diagnostic recovery boundary must precede body-edit churn publications",
  );
  let progressPhase: CancellationPublicationProgress["phase"] = "baseline-recovery";
  let progressBoundary = options.baselineRecoveryStart;
  let progressFence: number | null = null;
  const snapshotPublicationProgress = (persist = false): void => {
    updateCancellationPublicationProgress(
      options.publicationProgress,
      progressPhase,
      progressBoundary,
      progressFence,
      publications.since(0),
    );
    if (persist) options.persistPublicationProgress();
  };
  snapshotPublicationProgress();
  try {
    const recoveryBefore = await verifyCancellationRecovery(
      options.session,
      options.workspace,
      options.initialVersion,
      publications,
      options.baselineRecoveryStart,
    );
    const baselinePublicationFence = publications.count();
    progressPhase = "baseline-quiescence";
    progressBoundary = options.baselineRecoveryStart;
    progressFence = baselinePublicationFence;
    snapshotPublicationProgress(true);
    await sleep(10_000);
    snapshotPublicationProgress(true);
    assert.equal(
      publications.count(),
      baselinePublicationFence,
      "no opened-document diagnostics may publish after the baseline clean recovery fence",
    );
    const baseline = checkpoint(options.sampler.sample(), options.session.processId);
    const publicationWindowStart = publications.count();
    progressPhase = "cancellation";
    progressBoundary = publicationWindowStart;
    progressFence = null;
    snapshotPublicationProgress();
    const cancellation = await cancelledConcurrentEdits(
      options.session,
      options.workspace,
      options.initialVersion,
      options.gate,
    );
    const immediate = checkpoint(options.sampler.sample(), options.session.processId);
    progressPhase = "final-recovery";
    snapshotPublicationProgress();
    const recoveryAfter = await verifyCancellationRecovery(
      options.session,
      options.workspace,
      cancellation.final_version,
      publications,
      publicationWindowStart,
    );
    const finalPublicationFence = publications.count() - publicationWindowStart;
    progressPhase = "final-quiescence";
    progressFence = publications.count();
    snapshotPublicationProgress(true);
    await sleep(10_000);
    snapshotPublicationProgress(true);
    const windowPublications = publications.since(publicationWindowStart);
    const finalAfterQuiescence = publications.count() - publicationWindowStart;
    const childUri = documentUri(options.workspace, 0);
    const parentUri = documentUri(options.workspace, 1);
    const knownDocuments = new Map<string, ReadonlyMap<number, string>>([
      [
        childUri,
        new Map<number, string>([
          [options.initialVersion, sourceSha256(componentSource(0))],
          ...cancellation.samples.map((sample) => [sample.version, sample.source_sha256] as const),
        ]),
      ],
      [parentUri, new Map([[1, sourceSha256(componentSource(1))]])],
    ]);
    const publicationAudit = auditCancellationPublications(
      windowPublications,
      knownDocuments,
      { uri: childUri, version: cancellation.final_version },
      finalPublicationFence,
    );
    const after = checkpoint(options.sampler.sample(), options.session.processId);
    assertSameCohort(baseline, immediate, "immediate post-cancellation");
    assertSameCohort(baseline, after, "post-recovery quiescent");
    const nativeIdentity = cancellation.samples[0]?.admission.native_process;
    const proxyIdentity = cancellation.samples[0]?.admission.proxy_process;
    assert.ok(nativeIdentity && proxyIdentity, "admitted native and proxy identities are required");
    for (const identity of [nativeIdentity, proxyIdentity]) {
      assert.ok(
        baseline.cohort.some(
          (process) =>
            process.pid === identity.pid &&
            process.birth_ticks === identity.birth_ticks &&
            process.executable === identity.executable,
        ),
        `${identity.role} admission identity must belong to the measured process-tree cohort`,
      );
    }
    const allocationProfile: AllocationProfileLink = {
      same_session: options.profilePath !== null,
      expected_path: options.profilePath,
      receipt_field: "profile_json",
      scope: options.profilePath
        ? "the sibling profile_json receipt is exported after graceful shutdown by this exact gated server session; cumulative requested/released Rust allocation traffic and not cancellation-window-only live allocation"
        : "no allocation profile requested for this ordinary-process run",
    };
    progressPhase = "complete";
    snapshotPublicationProgress();
    return {
      ...cancellation,
      recovery: { before: recoveryBefore, after: recoveryAfter },
      diagnostic_publications: publicationAudit,
      diagnostic_fences: {
        baseline_clean_count: baselinePublicationFence,
        baseline_after_10s_count: baselinePublicationFence,
        final_clean_count: finalPublicationFence,
        final_after_10s_count: finalAfterQuiescence,
        no_publications_after_either_clean_fence: true as const,
      },
      retention: {
        scope:
          "like-for-like Maestro server and complete live descendant PID/birth/exe cohort after identical diagnostics/completion/hover recovery and 10s quiet windows; RSS deltas are process residency, not cancellation-only allocator attribution",
        baseline_after_recovery_10s: baseline,
        immediately_after_cancellation_responses: immediate,
        after_recovery_10s: after,
        cohort_identical: true as const,
        retained_delta_mib: {
          server: rssDeltaMib(after.server_rss_bytes, baseline.server_rss_bytes),
          process_tree: rssDeltaMib(after.process_tree_rss_bytes, baseline.process_tree_rss_bytes),
        },
      },
      allocation_profile: allocationProfile,
    };
  } finally {
    snapshotPublicationProgress();
    publications.stop();
  }
}
