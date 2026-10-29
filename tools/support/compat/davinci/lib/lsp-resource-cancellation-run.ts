import assert from "node:assert/strict";
import { setTimeout as sleep } from "node:timers/promises";
import type { LspSession } from "../../../../../tests/tooling/support/lsp/session.ts";
import { auditCancellationPublications, sourceSha256 } from "./lsp-resource-cancellation.ts";
import type { ResourceCorsaGate } from "./lsp-resource-corsa-gate.ts";
import { mib, type ProcessEntry, type Sampler, type TreeSample } from "./lsp-resource-sampler.ts";
import {
  cancelledConcurrentEdits,
  recordCancellationPublications,
  verifyCancellationRecovery,
} from "./lsp-resource-witness.ts";
import { componentSource } from "./lsp-resource-workspace.ts";

interface ResourceCheckpoint {
  monotonic_ns: string;
  server_rss_mib: number;
  process_tree_rss_mib: number;
  processes: ProcessEntry[];
  cohort: Array<{ pid: number; birth_ticks: number; executable: string }>;
}

export interface AllocationProfileLink {
  same_session: boolean;
  expected_path: string | null;
  actual_path?: string | null;
  path_matches?: boolean;
  receipt_field: "profile_json";
  scope: string;
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
  persist();
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
    server_rss_mib: mib(server.rss_bytes),
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

const deltaMib = (after: number, before: number): number => Math.round((after - before) * 10) / 10;

/**
 * Measure only the deterministic cancellation window. Both retained-RSS
 * endpoints run the same diagnostics/completion/hover recovery and 10-second
 * quiet fence; every process identity must survive unchanged.
 */
export async function measureCancellationRetention(options: {
  session: LspSession;
  sampler: Pick<Sampler, "sample">;
  workspace: string;
  initialVersion: number;
  gate: ResourceCorsaGate;
  profilePath: string | null;
}) {
  const publications = recordCancellationPublications(options.session, options.workspace);
  try {
    const recoveryBefore = await verifyCancellationRecovery(
      options.session,
      options.workspace,
      options.initialVersion,
    );
    const baselinePublicationFence = publications.count();
    await sleep(10_000);
    assert.equal(
      publications.count(),
      baselinePublicationFence,
      "no child diagnostics may publish after the baseline clean recovery fence",
    );
    const baseline = checkpoint(options.sampler.sample(), options.session.processId);
    const publicationWindowStart = publications.count();
    const cancellation = await cancelledConcurrentEdits(
      options.session,
      options.workspace,
      options.initialVersion,
      options.gate,
    );
    const immediate = checkpoint(options.sampler.sample(), options.session.processId);
    const recoveryAfter = await verifyCancellationRecovery(
      options.session,
      options.workspace,
      cancellation.final_version,
    );
    const finalPublicationFence = publications.count() - publicationWindowStart;
    await sleep(10_000);
    const windowPublications = publications.since(publicationWindowStart);
    const finalAfterQuiescence = publications.count() - publicationWindowStart;
    const knownVersions = new Map<number, string>([
      [options.initialVersion, sourceSha256(componentSource(0))],
      ...cancellation.samples.map((sample) => [sample.version, sample.source_sha256] as const),
    ]);
    const publicationAudit = auditCancellationPublications(
      windowPublications,
      knownVersions,
      cancellation.final_version,
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
          server: deltaMib(after.server_rss_mib, baseline.server_rss_mib),
          process_tree: deltaMib(after.process_tree_rss_mib, baseline.process_tree_rss_mib),
        },
      },
      allocation_profile: allocationProfile,
    };
  } finally {
    publications.stop();
  }
}
