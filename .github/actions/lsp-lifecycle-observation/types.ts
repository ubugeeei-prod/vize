export type Primitive = string | number | boolean | null;
export type PhaseEvent = {
  phase: string;
  atNs: string;
  method?: string;
  outcome?: string;
  detail?: Record<string, Primitive>;
};
export interface ObservationSink {
  event(event: PhaseEvent): void;
  refuse(reason: string): void;
  start(rootPid: number): void;
  stop(reason: string): void;
}
export const OBSERVATION_LIMITS = Object.freeze({
  intervalMs: 50, durationMs: 20_000, samples: 400, pids: 8,
  threads: 128, fileBytes: 8192, outputBytes: 8 * 1024 * 1024,
  finalReserveBytes: 8192, sampleDeadlineMs: 500,
  readBytes: 64 * 1024 * 1024,
});
export type ObservationLimits = { [Key in keyof typeof OBSERVATION_LIMITS]: number };
export type FileIdentity = { dev: string; ino: string; bytes: number; mtimeNs: string; ctimeNs: string };
export type ImageIdentity = FileIdentity & { role: "server" | "native"; realpath: string; sha256: string };
export type ObservationConfig = {
  schema: "vize.lsp.lifecycle-observation"; version: 1; repoRoot: string;
  sourceRevision: string; workflowRevision: string; outputRoot: string;
  images: ImageIdentity[]; nodeImage: FileIdentity & { realpath: string; sha256: string };
  sessionSourceSha256: string;
  metricsSourceSha256: string;
};
export type ProcStat = {
  pid: number; parentPid: number; birthTicks: string; command: string;
  state: string; userTicks: string; systemTicks: string; processor: number;
};
export interface ProcReader {
  read(file: string, maxBytes: number, signal: AbortSignal): Promise<string>;
  link(file: string, signal: AbortSignal): Promise<string>;
  list(file: string, signal: AbortSignal): Promise<string[]>;
  identity(file: string, signal: AbortSignal): Promise<FileIdentity>;
}
export type ThreadSample = { tid: number; birthTicks: string; command: string; state: string; userTicks: string; systemTicks: string; processor: number };
export type ProcessSample = {
  pid: number; parentPid: number; birthTicks: string; image: ImageIdentity;
  statusKiB: Record<string, number>; rollupKiB: Record<string, number>; threads: ThreadSample[];
};
export type SamplePacket = {
  kind: "sample";
  sample: number; reason: string; startNs: string; endNs: string; startPhase: string;
  scheduledNs: string; startLagMs: number;
  startPhaseEpoch: number; endPhaseEpoch: number;
  endPhase: string; phaseRelation: "contained" | "overlap"; processes: ProcessSample[];
  bytesRead: number; reads: number; wallMs: number; harnessCpuUserUs: number; harnessCpuSystemUs: number;
};
export type ImageRefusalEvidence = {
  operation: "bind-root" | "capture-before" | "capture-after" | "final-cohort";
  process: Pick<ProcStat, "pid" | "parentPid" | "birthTicks">;
  actual: { pathPrefixBase64: string; pathBytes: number; pathSha256: string; pathTruncated: boolean; metadata: FileIdentity };
  expectedCount: number;
  expected: Array<{ role: ImageIdentity["role"]; sha256: string; pathMatches: boolean; metadataMismatches: Array<keyof FileIdentity> }>;
  semantics: "Sequential proc stat, exe link and metadata; not an atomic image observation.";
};
export type ObservationRecord = SamplePacket | { kind: "phase"; event: PhaseEvent }
  | { kind: "notice"; atNs: string; reason: string; fatal: boolean;
      attempt?: Omit<SamplePacket, "kind" | "processes">; imageRefusal?: ImageRefusalEvidence };
