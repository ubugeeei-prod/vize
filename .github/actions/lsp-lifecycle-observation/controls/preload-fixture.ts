import type { ProcSampler } from "../proc-sampler.ts";
import type { ObservationConfig, SamplePacket } from "../types.ts";
import { OBSERVATION_LIMITS } from "../types.ts";
import { native, procStat, server } from "./proc-fixture.ts";
import { parseProcStat } from "../proc-sampler.ts";
import { ObservationWriter } from "../preload.ts";

export const phases = [
  "pre-lifecycle-source-close", "ephemeral-did-open-before-write",
  "workspace-open-scan-request-before-call", "workspace-open-scan-settled",
  "ephemeral-did-close-before-write", "ephemeral-close-clear",
  "workspace-deleted-scan-request-before-call", "workspace-deleted-scan-settled",
  "final-document-symbol-request-before-call", "final-document-symbol-settled",
  "next-ephemeral-v2-before-write",
];
export const windows = [
  "ephemeral-did-open-before-write", "workspace-open-scan-request-before-call",
  "workspace-deleted-scan-request-before-call",
];
export type Snapshot = ReturnType<ProcSampler["snapshotSummary"]>;

export function config(outputRoot: string): ObservationConfig {
  return {
    schema: "vize.lsp.lifecycle-observation", version: 1,
    repoRoot: "/synthetic/repository", outputRoot,
    sourceRevision: "a".repeat(40), workflowRevision: "b".repeat(40),
    images: [{ ...server }, { ...native }],
    nodeImage: {
      dev: server.dev, ino: server.ino, bytes: server.bytes,
      mtimeNs: server.mtimeNs, ctimeNs: server.ctimeNs,
      realpath: "/synthetic/node", sha256: "c".repeat(64),
    },
    sessionSourceSha256: "d".repeat(64),
  };
}

export function snapshot(overrides: Partial<Snapshot> = {}): Snapshot {
  return {
    diagnosticOnly: true, qualificationCredit: 0, invalid: false,
    root: parseProcStat(procStat(900, 42, "9007199254740993")),
    attempts: 3, samples: 3, skippedTicks: 0, intervalCallbacks: 7, inFlight: false,
    bytesRead: 1000, reads: 50, startNs: "0", endNs: "90",
    phaseEvents: [], refusals: [], stops: [{ reason: "first-cycle-methods-complete", atNs: "90" }],
    limits: { ...OBSERVATION_LIMITS }, limitsNote: "Synthetic data only; zero native credit.",
    ...overrides,
  };
}

export function sample(phase: string, index: number): SamplePacket {
  return {
    kind: "sample", sample: index + 1, reason: "synthetic-contained-window",
    scheduledNs: String(index * 10 + 1), startLagMs: 0,
    startNs: String(index * 10 + 1), endNs: String(index * 10 + 2),
    startPhase: phase, endPhase: phase, startPhaseEpoch: index + 1, endPhaseEpoch: index + 1,
    phaseRelation: "contained", bytesRead: 100, reads: 10, wallMs: 0.000001,
    harnessCpuUserUs: 0, harnessCpuSystemUs: 0, processes: [],
  };
}

export function fill(writer: ObservationWriter, missingPhase?: string, missingWindow?: string): void {
  for (const [index, phase] of phases.entries()) {
    if (phase !== missingPhase) writer.record({ kind: "phase", event: { phase, atNs: String(index * 10) } });
    if (windows.includes(phase) && phase !== missingWindow) writer.record(sample(phase, index));
  }
}
