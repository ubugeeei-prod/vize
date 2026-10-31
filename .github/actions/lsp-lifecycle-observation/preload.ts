import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createHash } from "node:crypto";
import { instrumentIncrementalMetricsPrototype, VBEN_REQUIRED_PHASES, type VbenNativeTarget } from "./vben-observer.ts";
import { ProcSampler } from "./proc-sampler.ts";
import { SmapsSnapshots } from "./smaps-snapshots.ts";
import { OBSERVATION_LIMITS } from "./types.ts";
import type { FileIdentity, ImageIdentity, ObservationConfig, ObservationRecord } from "./types.ts";

function object(value: unknown): object { if (value === null || (typeof value !== "object" && typeof value !== "function")) throw new Error("invalid object"); return value; }
function field(value: unknown, key: string): unknown { return Reflect.get(object(value), key); }
function text(value: unknown): string { if (typeof value !== "string") throw new Error("invalid string"); return value; }
function number(value: unknown): number { if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) throw new Error("invalid integer"); return value; }
function absolute(value: unknown): string { const result = text(value); if (!path.isAbsolute(result) || result.includes("\0")) throw new Error("invalid absolute path"); return result; }
function digest(value: unknown, length: number): string { const result = text(value); if (!new RegExp(`^[0-9a-f]{${length}}$`).test(result)) throw new Error("invalid digest"); return result; }
function metadata(value: unknown): FileIdentity {
  const result = { dev: text(field(value, "dev")), ino: text(field(value, "ino")), bytes: number(field(value, "bytes")), mtimeNs: text(field(value, "mtimeNs")), ctimeNs: text(field(value, "ctimeNs")) };
  if (result.bytes <= 0 || [result.dev, result.ino, result.mtimeNs, result.ctimeNs].some((item) => !/^\d+$/.test(item))) throw new Error("invalid file identity"); return result;
}
export function parseObservationConfig(value: unknown): ObservationConfig {
  if (field(value, "schema") !== "vize.lsp.lifecycle-observation" || field(value, "version") !== 1) throw new Error("config schema refused");
  const values = field(value, "images"); if (!Array.isArray(values) || values.length !== 2) throw new Error("expected server/native image pair");
  const images = values.map((item: unknown): ImageIdentity => {
    const role = field(item, "role"); if (role !== "server" && role !== "native") throw new Error("image role refused");
    return { ...metadata(item), role, realpath: absolute(field(item, "realpath")), sha256: digest(field(item, "sha256"), 64) };
  });
  if (new Set(images.map((image) => image.role)).size !== 2 || new Set(images.map((image) => image.realpath)).size !== 2) throw new Error("duplicate image role/path");
  const node = field(value, "nodeImage");
  return { schema: "vize.lsp.lifecycle-observation", version: 1, repoRoot: absolute(field(value, "repoRoot")), outputRoot: absolute(field(value, "outputRoot")),
    sourceRevision: digest(field(value, "sourceRevision"), 40), workflowRevision: digest(field(value, "workflowRevision"), 40), images,
    nodeImage: { ...metadata(node), realpath: absolute(field(node, "realpath")), sha256: digest(field(node, "sha256"), 64) }, sessionSourceSha256: digest(field(value, "sessionSourceSha256"), 64), metricsSourceSha256: digest(field(value, "metricsSourceSha256"), 64) };
}
function hash(bytes: Uint8Array): string { return createHash("sha256").update(bytes).digest("hex"); }
function smallFile(file: string): Buffer {
  const descriptor = fs.openSync(file, "r");
  try { const bytes = Buffer.alloc(65537); const count = fs.readSync(descriptor, bytes, 0, bytes.length, 0); if (count > 65536) throw new Error("metadata file cap"); return bytes.subarray(0, count); }
  finally { fs.closeSync(descriptor); }
}
function verifyMetadata(file: string, expected: FileIdentity): void {
  if (fs.realpathSync(file) !== file) throw new Error("image path is not canonical");
  const actual = fs.statSync(file, { bigint: true });
  if (!actual.isFile() || actual.dev.toString() !== expected.dev || actual.ino.toString() !== expected.ino || Number(actual.size) !== expected.bytes
    || actual.mtimeNs.toString() !== expected.mtimeNs || actual.ctimeNs.toString() !== expected.ctimeNs) throw new Error("image metadata changed");
}
const REQUIRED_PHASES = VBEN_REQUIRED_PHASES;
export type VbenCohort = {
  state: "complete" | "incomplete";
  target: { pid: number; birthTicks: string; parentPid: number; executableSha256: string } | null;
  refusals: string[]; refusalCount: number;
  selection: "first-newly-observed-native-direct-child-vs-coldOpen;not-semantic-app-owner";
};
function emptyCohort(): VbenCohort {
  return { state: "incomplete", target: null, refusals: [], refusalCount: 0,
    selection: "first-newly-observed-native-direct-child-vs-coldOpen;not-semantic-app-owner" };
}
export class ObservationWriter {
  private readonly descriptor: number;
  private readonly summaryPath: string;
  private bytes = 0;
  private closed = false;
  private truncated = false;
  private samples = 0;
  private readonly phaseCounts: Record<string, number> = {};
  private readonly contained: Record<string, number> = {};
  constructor(directory: string, basename: string, header: unknown) {
    if (!/^[a-z0-9-]+$/.test(basename)) throw new Error("invalid output basename");
    if (fs.realpathSync(directory) !== directory || !fs.lstatSync(directory).isDirectory()) throw new Error("output root must be an owned canonical directory");
    this.summaryPath = path.join(directory, `${basename}.summary.json`);
    if (fs.existsSync(this.summaryPath)) throw new Error("summary already exists");
    this.descriptor = fs.openSync(path.join(directory, `${basename}.jsonl`), "wx", 0o600);
    try { this.append(header); } catch (error: unknown) { fs.closeSync(this.descriptor); this.closed = true; throw error; }
  }
  private append(value: unknown): void {
    if (this.closed) throw new Error("observation writer closed");
    const encoded = JSON.stringify(value); if (encoded === undefined) throw new Error("non-JSON observation"); const bytes = Buffer.from(encoded + "\n");
    if (this.bytes + bytes.length > OBSERVATION_LIMITS.outputBytes - OBSERVATION_LIMITS.finalReserveBytes) { this.truncated = true; throw new Error("observation output byte cap"); }
    let offset = 0;
    while (offset < bytes.length) { const count = fs.writeSync(this.descriptor, bytes, offset, bytes.length - offset); if (count <= 0) throw new Error("observation write failed"); offset += count; this.bytes += count; }
  }
  record(record: ObservationRecord): void {
    this.append(record);
    if (record.kind === "phase") this.phaseCounts[record.event.phase] = (this.phaseCounts[record.event.phase] ?? 0) + 1;
    if (record.kind === "sample") this.samples += 1;
    if (record.kind === "sample" && record.phaseRelation === "contained") this.contained[record.startPhase] = (this.contained[record.startPhase] ?? 0) + 1;
  }
  finish(config: ObservationConfig, rootPid: number, snapshot: ReturnType<ProcSampler["snapshotSummary"]>, preloadErrors: string[], cohort: VbenCohort = emptyCohort()): void {
    if (this.closed) return;
    const missingPhases = REQUIRED_PHASES.filter((phase) => (this.phaseCounts[phase] ?? 0) === 0);
    const missingContainedWindows: string[] = [];
    const invalidPhaseCounts = REQUIRED_PHASES.filter((phase) => (this.phaseCounts[phase] ?? 0) !== 1);
    const selected = cohort.target;
    const native = config.images.find((image) => image.role === "native");
    const cohortComplete = cohort.state === "complete" && cohort.refusalCount === 0 && cohort.refusals.length === 0
      && selected !== null && Number.isSafeInteger(selected.pid) && selected.pid > 0 && selected.pid !== rootPid
      && selected.parentPid === rootPid && /^\d+$/.test(selected.birthTicks) && selected.executableSha256 === native?.sha256;
    const complete = !snapshot.invalid && !snapshot.inFlight && !this.truncated && preloadErrors.length === 0 && snapshot.root !== null
      && snapshot.samples > 0 && this.samples === snapshot.samples && invalidPhaseCounts.length === 0 && cohortComplete;
    const summary = { schema: "vize.lsp.lifecycle-observation.summary", version: 1, sourceRevision: config.sourceRevision, workflowRevision: config.workflowRevision,
      clientPid: process.pid, rootPid, qualificationCredit: 0, complete, invalid: snapshot.invalid, inFlight: snapshot.inFlight, truncated: this.truncated,
      missingPhases, invalidPhaseCounts, observedSampleRecords: this.samples, phaseCounts: this.phaseCounts, containedSamplesByPhase: this.contained, missingContainedWindows,
      cohort: { ...cohort, target: cohort.target === null ? null : { ...cohort.target }, refusals: cohort.refusals.slice(0, 8).map((reason) => reason.slice(0, 128)) },
      containedCoveragePolicy: "Actual sample counts only; no contained sample required for every fast lane.",
      attempts: snapshot.attempts, samples: snapshot.samples, skippedTicks: snapshot.skippedTicks, intervalCallbacks: snapshot.intervalCallbacks, bytesRead: snapshot.bytesRead, reads: snapshot.reads,
      startNs: snapshot.startNs, endNs: snapshot.endNs, rootBirthTicks: snapshot.root?.birthTicks ?? null,
      refusals: snapshot.refusals.slice(0, 8).map((reason) => reason.slice(0, 128)), jsonlBytes: this.bytes, outputBytes: this.bytes,
      preloadErrors: preloadErrors.slice(0, 4).map((reason) => reason.slice(0, 128)),
      limits: snapshot.limits, stopReasons: snapshot.stops.slice(0, 4), metadataCompacted: false, uncompactedSummarySha256: "",
      refusalCount: snapshot.refusals.length, preloadErrorCount: preloadErrors.length, stopReasonCount: snapshot.stops.length,
      semantics: "Diagnostic only. Original measure settlement follows its retained RSS sample, not worker quiescence. First newly observed native cohort is not semantic app ownership. Fast lanes need not contain a 50ms sample. ThreadCPU/state/birth only, no per-threadRSS. Logical read deadlines cannot cancel blocked kernel reads." };
    let summaryDescriptor: number | undefined;
    try {
      const encode = (): Buffer => {
        let result = Buffer.from(JSON.stringify(summary) + "\n");
        for (let pass = 0; pass < 4; pass += 1) {
          const total = this.bytes + result.length; if (summary.outputBytes === total) break;
          summary.outputBytes = total; result = Buffer.from(JSON.stringify(summary) + "\n");
        }
        return result;
      };
      let encoded = encode();
      if (encoded.length > OBSERVATION_LIMITS.finalReserveBytes) {
        summary.uncompactedSummarySha256 = hash(encoded); summary.metadataCompacted = true; summary.complete = false; summary.truncated = true;
        summary.refusals = []; summary.preloadErrors = []; summary.stopReasons = []; summary.cohort.refusals = [];
        summary.phaseCounts = Object.fromEntries(REQUIRED_PHASES.map((phase) => [phase, this.phaseCounts[phase] ?? 0]));
        summary.containedSamplesByPhase = Object.fromEntries(REQUIRED_PHASES.map((phase) => [phase, this.contained[phase] ?? 0]));
        encoded = encode();
      }
      if (encoded.length > OBSERVATION_LIMITS.finalReserveBytes || this.bytes + encoded.length > OBSERVATION_LIMITS.outputBytes) throw new Error("reserved summary byte cap");
      summaryDescriptor = fs.openSync(this.summaryPath, "wx", 0o600); fs.writeFileSync(summaryDescriptor, encoded);
    }
    finally { if (summaryDescriptor !== undefined) fs.closeSync(summaryDescriptor); fs.closeSync(this.descriptor); this.closed = true; }
  }
}

async function attach(config: ObservationConfig): Promise<void> {
  if (process.platform !== "linux") throw new Error("Linux observation only");
  const sessionSource = path.join(config.repoRoot, "tests/tooling/support/lsp/session.ts");
  const metricsSource = path.join(config.repoRoot, "tests/performance/support/incremental-metrics.ts");
  if (hash(smallFile(sessionSource)) !== config.sessionSourceSha256 || hash(smallFile(metricsSource)) !== config.metricsSourceSha256) throw new Error("session/metrics source hash refused");
  for (const image of config.images) verifyMetadata(image.realpath, image);
  const foundServer = config.images.find((image) => image.role === "server"), foundNative = config.images.find((image) => image.role === "native");
  if (foundServer === undefined || foundNative === undefined) throw new Error("image pair absent");
  const server: ImageIdentity = foundServer, native: ImageIdentity = foundNative;
  const helperRoot = path.dirname(fileURLToPath(import.meta.url));
  const helperHashes = Object.fromEntries(["types.ts", "vben-observer.ts", "proc-sampler.ts", "preload.ts", "smaps-snapshots.ts"].map((file) => [file, hash(smallFile(path.join(helperRoot, file)))]));
  const imported: unknown = await import(pathToFileURL(metricsSource).href);
  const prototype = object(field(field(imported, "IncrementalMetrics"), "prototype"));
  const sessions: Array<{ pid: number; sampler: ProcSampler; writer: ObservationWriter; errors: string[]; cohort: VbenCohort; smaps?: SmapsSnapshots }> = [];
  let finalized = false;
  const uninstall = instrumentIncrementalMetricsPrototype(prototype, { serverImageSha: server.sha256, nativeImageSha: native.sha256,
    sinkFactory(root) {
      if (sessions.length !== 0) {
        const previous = sessions[0]; previous?.sampler.refuse("multiple Vben metrics sessions");
        if (previous !== undefined) { previous.cohort.state = "incomplete"; previous.cohort.refusalCount += 1; previous.cohort.refusals.push("multiple Vben metrics sessions"); }
        throw new Error("only one original Vben metrics session is expected");
      }
      const errors: string[] = [], cohort = emptyCohort();
      try { if (process.version !== "v24.14.0" || fs.realpathSync(process.execPath) !== config.nodeImage.realpath) throw new Error("unexpected task Node identity"); verifyMetadata(config.nodeImage.realpath, config.nodeImage); }
      catch { errors.push("task Node metadata refused"); }
      const writer = new ObservationWriter(config.outputRoot, `client-${process.pid}-server-${root.rootPid}`, { kind: "header", diagnosticOnly: true, qualificationCredit: 0,
        clientPid: process.pid, rootPid: root.rootPid, rootBirthTicks: root.rootBirthTicks, sourceRevision: config.sourceRevision, workflowRevision: config.workflowRevision,
        sessionSourceSha256: config.sessionSourceSha256, metricsSourceSha256: config.metricsSourceSha256, suite: "vben-lsp-incremental",
        images: config.images, nodeImage: config.nodeImage, helperHashes, limits: OBSERVATION_LIMITS,
        hashScope: "driver hashes images before/after; original metric samples retain physical native hashes; observers revalidate metadata", preloadErrors: errors });
      const sampler = new ProcSampler({ rootPid: root.rootPid, images: config.images, emit: (record) => writer.record(record) });
      const session: typeof sessions[number] = { pid: root.rootPid, sampler, writer, errors, cohort }; sessions.push(session);
      function refuse(reason: string): void {
        cohort.state = "incomplete"; cohort.refusalCount += 1;
        if (cohort.refusals.length < 8) cohort.refusals.push(reason.slice(0, 128));
        sampler.refuse(reason); session.smaps?.refuse(reason);
      }
      function select(target: VbenNativeTarget): void {
        if (cohort.target !== null) return;
        cohort.target = { pid: target.pid, birthTicks: target.birthTicks, parentPid: target.parentPid, executableSha256: target.nativeImageSha };
        try {
          session.smaps = new SmapsSnapshots({ rootPid: target.pid, targetBirthTicks: target.birthTicks, image: native, mode: "vben-native",
            ownerRoot: { pid: root.rootPid, birthTicks: root.rootBirthTicks, image: server },
            outputRoot: path.join(config.outputRoot, `vma-client-${process.pid}-server-${root.rootPid}`),
            sourceRevision: config.sourceRevision, workflowRevision: config.workflowRevision });
          session.smaps.start(target.pid);
        } catch { errors.push("native smaps setup incomplete"); refuse("native smaps setup incomplete"); }
      }
      sampler.start(root.rootPid);
      if (errors.length > 0) refuse("preload Node identity failed");
      return {
        phase(event) {
          sampler.event(event);
          if (event.target !== null && event.outcome === "fulfilled" && event.phase === "vben-coldOpenSecondApp-settled") select(event.target);
          session.smaps?.event(event);
        },
        stop(reason) {
          if (reason === "vben-all-lanes-settled" && cohort.target !== null && cohort.refusalCount === 0 && errors.length === 0) cohort.state = "complete";
          sampler.stop(reason); session.smaps?.stop(reason);
        },
        refuse,
      };
    },
  });
  function finalize(): void {
    if (finalized) return; finalized = true;
    try { uninstall(); } catch { /* Missing summary remains diagnostic-incomplete. */ }
    for (const session of sessions) {
      try { session.sampler.stop("node-exit-summary"); session.writer.finish(config, session.pid, session.sampler.snapshotSummary(), session.errors, session.cohort); }
      catch { process.stderr.write("lifecycle observation summary incomplete\n"); }
      if (session.smaps !== undefined) {
        session.smaps.stop("node-exit-summary");
        const rootBirth = session.sampler.snapshotSummary().root?.birthTicks;
        const smapsBirth = session.smaps.snapshotSummary().rootBirthTicks;
        if (rootBirth !== undefined && smapsBirth !== null && rootBirth !== smapsBirth) session.smaps.refuse("proc/smaps owner birth mismatch");
        void session.smaps.finish().catch(() => process.stderr.write("smaps observation summary incomplete\n"));
      }
    }
  }
  process.once("beforeExit", finalize);
  process.once("exit", () => {
    finalize();
    for (const session of sessions) {
      try { session.smaps?.finishOnExit(); }
      catch { process.stderr.write("smaps exit summary incomplete\n"); }
    }
  });
}
if (process.env["VIZE_LSP_LIFECYCLE_OBSERVATION"] === "diagnostic-only-v1") {
  try {
    const file = process.env["VIZE_LSP_LIFECYCLE_CONFIG"]; if (file === undefined) throw new Error("config path absent");
    const parsed: unknown = JSON.parse(smallFile(file).toString("utf8")); await attach(parseObservationConfig(parsed));
  } catch { process.stderr.write("lifecycle observation preload incomplete\n"); }
}
