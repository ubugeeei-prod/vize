import { createHash } from "node:crypto";
import { open, opendir, readlink, stat } from "node:fs/promises";
import type { FileIdentity, ImageIdentity, ImageRefusalEvidence, ObservationLimits, ObservationRecord, ObservationSink, PhaseEvent,
  ProcReader, ProcStat, ProcessSample, SamplePacket, ThreadSample } from "./types.ts";
import { OBSERVATION_LIMITS } from "./types.ts";

function integer(text: string | undefined): number {
  if (text === undefined || !/^\d+$/.test(text)) throw new Error("invalid proc integer");
  const value = Number(text); if (!Number.isSafeInteger(value)) throw new Error("unsafe proc integer"); return value;
}
function ticks(text: string | undefined): string { if (text === undefined || !/^\d+$/.test(text)) throw new Error("invalid proc ticks"); return text; }
export function parseProcStat(text: string): ProcStat {
  const match = /^(\d+) \((.*)\) ([A-Za-z]) (.*)$/su.exec(text.trim());
  if (match === null || match[4] === undefined || match[2] === undefined || match[3] === undefined) throw new Error("invalid proc stat");
  const fields = match[4].split(/\s+/u); const pid = integer(match[1]); if (pid <= 0) throw new Error("invalid pid");
  return { pid, command: match[2], state: match[3], parentPid: integer(fields[0]), userTicks: ticks(fields[10]),
    systemTicks: ticks(fields[11]), birthTicks: ticks(fields[18]), processor: integer(fields[35]) };
}
export function sameAnchor(left: ProcStat, right: ProcStat): boolean {
  return left.pid === right.pid && left.parentPid === right.parentPid && left.birthTicks === right.birthTicks;
}
export function parseKiBFields(text: string, names: readonly string[]): Record<string, number> {
  const values: Record<string, number> = {};
  for (const line of text.split("\n")) {
    const match = /^([A-Za-z_]+):\s+(\d+) kB\s*$/.exec(line);
    if (match?.[1] !== undefined && names.includes(match[1])) {
      if (Object.hasOwn(values, match[1])) throw new Error("duplicate proc field"); values[match[1]] = integer(match[2]);
    }
  }
  return values;
}
function identityEqual(left: FileIdentity, right: FileIdentity): boolean {
  return left.dev === right.dev && left.ino === right.ino && left.bytes === right.bytes
    && left.mtimeNs === right.mtimeNs && left.ctimeNs === right.ctimeNs;
}
export function runtimeProcReader(): ProcReader {
  return {
    async read(file, maxBytes, signal) {
      signal.throwIfAborted(); const handle = await open(file, "r");
      try {
        signal.throwIfAborted(); const buffer = Buffer.alloc(maxBytes + 1);
        const result = await handle.read(buffer, 0, buffer.length, 0); signal.throwIfAborted();
        if (result.bytesRead > maxBytes) throw new Error("proc file byte cap"); return buffer.subarray(0, result.bytesRead).toString("utf8");
      } finally { await handle.close(); }
    },
    async link(file, signal) { signal.throwIfAborted(); const value = await readlink(file); signal.throwIfAborted(); return value; },
    async list(file, signal) {
      signal.throwIfAborted(); const values: string[] = []; const directory = await opendir(file);
      for await (const item of directory) {
        signal.throwIfAborted(); values.push(item.name);
        if (values.length > OBSERVATION_LIMITS.threads) throw new Error("thread cap");
      }
      return values;
    },
    async identity(file, signal) {
      signal.throwIfAborted(); const value = await stat(file, { bigint: true }); signal.throwIfAborted();
      const bytes = Number(value.size); if (!Number.isSafeInteger(bytes) || !value.isFile()) throw new Error("invalid executable metadata");
      return { dev: value.dev.toString(), ino: value.ino.toString(), bytes, mtimeNs: value.mtimeNs.toString(), ctimeNs: value.ctimeNs.toString() };
    },
  };
}
class MixedSample extends Error {}
class InvalidCustody extends Error {
  readonly imageRefusal: ImageRefusalEvidence | undefined;
  constructor(message: string, imageRefusal?: ImageRefusalEvidence) { super(message); this.imageRefusal = imageRefusal; }
}
export type SamplerOptions = {
  rootPid: number; images: readonly ImageIdentity[]; emit: (record: ObservationRecord) => void;
  reader?: ProcReader; now?: () => bigint;
  limits?: Partial<Pick<ObservationLimits, "intervalMs" | "durationMs" | "samples" | "pids" | "threads" | "sampleDeadlineMs" | "readBytes">>;
};
/** Diagnostic-only sampler. No process signal, command, RPC, purge or thread retirement. */
export class ProcSampler implements ObservationSink {
  private readonly options: SamplerOptions;
  private readonly reader: ProcReader;
  private readonly now: () => bigint;
  private readonly limits: ObservationLimits;
  private readonly ready: Promise<void>;
  private readonly known = new Map<number, { stat: ProcStat; image: ImageIdentity }>();
  private readonly threadAnchors = new Map<number, { birth: string; owner: number; ownerBirth: string }>();
  private root: ProcStat | undefined;
  private phase = "before-lifecycle";
  private epoch = 0;
  private active = false;
  private busy = false;
  private invalid = false;
  private attempts = 0;
  private samples = 0;
  private intervalTicks = 0;
  private skipped = 0;
  private bytes = 0;
  private reads = 0;
  private started: bigint | undefined;
  private ended: bigint | undefined;
  private interval: NodeJS.Timeout | undefined;
  private deadline: NodeJS.Timeout | undefined;
  private controller: AbortController | undefined;
  private readonly refusals: string[] = [];
  private readonly phases: PhaseEvent[] = [];
  private readonly stops: Array<{ reason: string; atNs: string }> = [];
  constructor(options: SamplerOptions) {
    this.options = options;
    this.reader = options.reader ?? runtimeProcReader(); this.now = options.now ?? (() => process.hrtime.bigint());
    this.limits = { ...OBSERVATION_LIMITS, ...options.limits };
    for (const key of ["durationMs", "samples", "pids", "threads", "sampleDeadlineMs", "readBytes"]) {
      const value: unknown = Reflect.get(this.limits, key); const maximum: unknown = Reflect.get(OBSERVATION_LIMITS, key);
      if (typeof value !== "number" || typeof maximum !== "number" || !Number.isSafeInteger(value) || value <= 0 || value > maximum) throw new Error("invalid bounded limit");
    }
    if (this.limits.intervalMs < 50 || !Number.isSafeInteger(this.limits.intervalMs)) throw new Error("sampling cannot be faster than50ms");
    this.ready = this.bindRoot().catch((error: unknown) => {
      if (error instanceof InvalidCustody && error.imageRefusal !== undefined) this.output({ kind: "notice", atNs: this.now().toString(),
        reason: "bind-root:image-custody-evidence", fatal: true, imageRefusal: error.imageRefusal });
      this.refuse(this.message(error));
    });
  }
  private message(error: unknown): string { return error instanceof Error ? error.message : String(error); }
  private output(record: ObservationRecord): void {
    try { this.options.emit(record); } catch {
      if (this.refusals.length < 32) this.refusals.push("observation output failed");
      this.invalid = true; this.active = false; this.clearTimers(); this.controller?.abort();
    }
  }
  private async read(file: string, signal: AbortSignal): Promise<string> {
    const value = await this.reader.read(file, this.limits.fileBytes, signal); const count = Buffer.byteLength(value);
    if (count > this.limits.fileBytes) throw new InvalidCustody("proc file byte cap");
    this.bytes += count; this.reads += 1; if (this.bytes > this.limits.readBytes) throw new InvalidCustody("total proc byte cap"); return value;
  }
  private async processStat(pid: number, signal: AbortSignal): Promise<ProcStat> {
    const value = parseProcStat(await this.read(`/proc/${pid}/stat`, signal)); if (value.pid !== pid) throw new InvalidCustody("foreign proc PID"); return value;
  }
  private async image(process: ProcStat, signal: AbortSignal, operation: ImageRefusalEvidence["operation"]): Promise<ImageIdentity> {
    const file = `/proc/${process.pid}/exe`; const realpath = await this.reader.link(file, signal); const metadata = await this.reader.identity(file, signal);
    const image = this.options.images.find((value: ImageIdentity) => value.realpath === realpath && identityEqual(value, metadata));
    if (image === undefined) {
      const bytes = Buffer.from(realpath), keys: Array<keyof FileIdentity> = ["dev", "ino", "bytes", "mtimeNs", "ctimeNs"];
      throw new InvalidCustody("foreign or changed executable image", { operation,
        process: { pid: process.pid, parentPid: process.parentPid, birthTicks: process.birthTicks },
        actual: { pathPrefixBase64: bytes.subarray(0, 1024).toString("base64"), pathBytes: bytes.length,
          pathSha256: createHash("sha256").update(bytes).digest("hex"), pathTruncated: bytes.length > 1024, metadata: { ...metadata } },
        expectedCount: this.options.images.length, expected: this.options.images.slice(0, 2).map(value => ({ role: value.role, sha256: value.sha256,
          pathMatches: value.realpath === realpath, metadataMismatches: keys.filter(key => value[key] !== metadata[key]) })),
        semantics: "Sequential proc stat, exe link and metadata; not an atomic image observation." });
    }
    return image;
  }
  private async bindRoot(): Promise<void> {
    const signal = AbortSignal.timeout(this.limits.sampleDeadlineMs);
    const before = await this.processStat(this.options.rootPid, signal); const image = await this.image(before, signal, "bind-root");
    const after = await this.processStat(before.pid, signal);
    if (!sameAnchor(before, after) || image.role !== "server") throw new InvalidCustody("root PID/image custody failed");
    this.root = before; this.known.set(before.pid, { stat: before, image });
  }
  event(event: PhaseEvent): void {
    this.output({ kind: "phase", event });
    if (event.phase !== "uncorrelated-response" && event.phase !== "uncorrelated-publication" && event.phase !== "ephemeral-version1-publication") {
      if (this.phases.length >= 256) { this.refuse("phase event cap"); return; }
      this.phase = event.phase; this.epoch += 1; this.phases.push(event);
    }
    this.requestSample(`boundary:${event.phase}`, /^\d+$/.test(event.atNs) ? BigInt(event.atNs) : undefined);
  }
  refuse(reason: string): void {
    if (this.refusals.length < 32) this.refusals.push(reason.slice(0, 240));
    this.invalid = true; this.output({ kind: "notice", atNs: this.now().toString(), reason: reason.slice(0, 240), fatal: true }); this.stop("observation-refused");
  }
  start(rootPid: number): void {
    if (rootPid !== this.options.rootPid) { this.refuse("foreign root PID"); return; }
    if (this.started !== undefined || this.invalid) return;
    this.started = this.now(); this.active = true;
    this.interval = setInterval(() => {
      this.intervalTicks += 1;
      const due = this.started === undefined ? undefined : this.started + BigInt(this.intervalTicks * this.limits.intervalMs) * 1_000_000n;
      this.requestSample("interval50ms", due);
    }, this.limits.intervalMs); this.interval.unref();
    this.deadline = setTimeout(() => { this.refuse("observation20sdeadline"); }, this.limits.durationMs); this.deadline.unref();
    void this.ready.then(() => this.requestSample("pre-lifecycle-arm"));
  }
  stop(reason: string): void {
    this.active = false;
    if (this.ended === undefined) { this.ended = this.now(); this.epoch += 1; }
    if (this.stops.length < 8) this.stops.push({ reason, atNs: this.now().toString() });
    this.clearTimers(); if (this.invalid) this.controller?.abort(new Error(reason));
  }
  private clearTimers(): void { if (this.interval !== undefined) clearInterval(this.interval); if (this.deadline !== undefined) clearTimeout(this.deadline); }
  requestSample(reason: string, scheduled?: bigint): void {
    if (!this.active || this.invalid || this.root === undefined) return;
    if (this.busy) { this.skipped += 1; return; }
    if (this.attempts >= this.limits.samples) { this.refuse("global sample cap"); return; }
    if (this.started !== undefined && this.now() - this.started > BigInt(this.limits.durationMs) * 1_000_000n) { this.refuse("observation wall deadline"); return; }
    this.busy = true; this.attempts += 1;
    const attempt = this.attempts; const started = this.now(); const phase = this.phase; const epoch = this.epoch;
    const due = scheduled ?? started;
    const firstBytes = this.bytes; const firstReads = this.reads; const cpu = process.cpuUsage();
    const controller = new AbortController(); this.controller = controller;
    const timer = setTimeout(() => controller.abort(new Error("sample deadline")), this.limits.sampleDeadlineMs); timer.unref();
    void this.capture(controller.signal).then((processes: ProcessSample[]) => {
      const ended = this.now(); const used = process.cpuUsage(cpu);
      const packet: SamplePacket = { kind: "sample", sample: attempt, reason, startNs: started.toString(), endNs: ended.toString(),
        startPhase: phase, endPhase: this.phase, phaseRelation: epoch === this.epoch ? "contained" : "overlap", processes,
        scheduledNs: due.toString(), startLagMs: Number(started - due) / 1e6,
        startPhaseEpoch: epoch, endPhaseEpoch: this.epoch,
        bytesRead: this.bytes - firstBytes, reads: this.reads - firstReads, wallMs: Number(ended - started) / 1e6,
        harnessCpuUserUs: used.user, harnessCpuSystemUs: used.system };
      this.samples += 1; this.output(packet);
    }, (error: unknown) => {
      const ended = this.now(); const used = process.cpuUsage(cpu);
      this.output({ kind: "notice", atNs: ended.toString(), reason: `sample-gap:${this.message(error)}`.slice(0, 240), fatal: error instanceof InvalidCustody,
        ...(error instanceof InvalidCustody && error.imageRefusal !== undefined ? { imageRefusal: error.imageRefusal } : {}),
        attempt: { sample: attempt, reason, startNs: started.toString(), endNs: ended.toString(), scheduledNs: due.toString(), startLagMs: Number(started - due) / 1e6,
          startPhase: phase, endPhase: this.phase, startPhaseEpoch: epoch, endPhaseEpoch: this.epoch, phaseRelation: epoch === this.epoch ? "contained" : "overlap",
          bytesRead: this.bytes - firstBytes, reads: this.reads - firstReads, wallMs: Number(ended - started) / 1e6, harnessCpuUserUs: used.user, harnessCpuSystemUs: used.system } });
      if (error instanceof InvalidCustody) this.refuse(error.message);
    }).finally(() => { clearTimeout(timer); this.busy = false; this.controller = undefined; }).catch((error: unknown) => this.refuse(this.message(error)));
  }
  private async tasks(pid: number, signal: AbortSignal): Promise<number[]> {
    const values = await this.reader.list(`/proc/${pid}/task`, signal); if (values.length > this.limits.threads) throw new InvalidCustody("thread cap");
    const ids = values.map((value: string) => integer(value)).sort((a, b) => a - b);
    if (ids.some((tid: number) => tid <= 0) || new Set(ids).size !== ids.length) throw new InvalidCustody("invalid TID cohort"); return ids;
  }
  private async capture(signal: AbortSignal): Promise<ProcessSample[]> {
    const root = this.root; if (root === undefined) throw new InvalidCustody("unanchored root");
    const queue: Array<{ pid: number; parent: number | undefined }> = [{ pid: root.pid, parent: undefined }];
    const result: ProcessSample[] = []; const seen = new Set<number>(); let threads = 0;
    for (let index = 0; index < queue.length; index += 1) {
      signal.throwIfAborted(); const entry = queue[index]; if (entry === undefined) throw new InvalidCustody("missing cohort");
      if (seen.has(entry.pid)) continue; seen.add(entry.pid);
      if (seen.size > this.limits.pids) throw new InvalidCustody("PID cohort cap");
      const before = await this.processStat(entry.pid, signal); const image = await this.image(before, signal, "capture-before");
      if (entry.parent !== undefined && image.role !== "native") throw new InvalidCustody("foreign descendant image role");
      if (entry.parent === undefined ? !sameAnchor(root, before) : before.parentPid !== entry.parent) throw new InvalidCustody("foreign/reused PID ancestry");
      const known = this.known.get(before.pid);
      if (known !== undefined && (!sameAnchor(known.stat, before) || known.image.sha256 !== image.sha256)) throw new InvalidCustody("PID reuse/image change");
      if (this.known.size >= this.limits.pids && known === undefined) throw new InvalidCustody("lifetime PID cap");
      this.known.set(before.pid, { stat: before, image });
      const status = parseKiBFields(await this.read(`/proc/${before.pid}/status`, signal), ["VmRSS", "VmHWM", "RssAnon", "RssFile", "RssShmem", "VmSwap"]);
      const rollup = parseKiBFields(await this.read(`/proc/${before.pid}/smaps_rollup`, signal), ["Rss", "Pss", "Pss_Anon", "Pss_File", "Pss_Shmem", "Anonymous", "Private_Clean", "Private_Dirty", "Shared_Clean", "Shared_Dirty", "Swap"]);
      if (status["VmRSS"] === undefined || status["RssAnon"] === undefined || status["RssFile"] === undefined || rollup["Rss"] === undefined) throw new MixedSample("missing process memory fields");
      const ids = await this.tasks(before.pid, signal); threads += ids.length; if (threads > this.limits.threads) throw new InvalidCustody("total thread cap");
      const threadSamples: ThreadSample[] = [];
      for (const tid of ids) {
        const base = `/proc/${before.pid}/task/${tid}`; const first = parseProcStat(await this.read(`${base}/stat`, signal));
        if (first.pid !== tid) throw new InvalidCustody("foreign TID");
        const previousThread = this.threadAnchors.get(tid);
        if (previousThread !== undefined && (previousThread.birth !== first.birthTicks || previousThread.owner !== before.pid || previousThread.ownerBirth !== before.birthTicks)) throw new InvalidCustody("TID reuse/foreign owner");
        if (previousThread === undefined && this.threadAnchors.size >= this.limits.threads) throw new InvalidCustody("lifetime TID cap");
        this.threadAnchors.set(tid, { birth: first.birthTicks, owner: before.pid, ownerBirth: before.birthTicks });
        const children = (await this.read(`${base}/children`, signal)).trim();
        if (children !== "") for (const child of children.split(/\s+/)) {
          const childPid = integer(child); if (childPid <= 0) throw new InvalidCustody("invalid child PID");
          queue.push({ pid: childPid, parent: before.pid });
        }
        if (new Set(queue.map((item) => item.pid)).size > this.limits.pids) throw new InvalidCustody("PID cohort cap");
        const after = parseProcStat(await this.read(`${base}/stat`, signal)); if (!sameAnchor(first, after)) throw new InvalidCustody("TID reuse during sample");
        threadSamples.push({ tid, birthTicks: first.birthTicks, command: first.command, state: first.state, userTicks: first.userTicks, systemTicks: first.systemTicks, processor: first.processor });
      }
      const after = await this.processStat(before.pid, signal); const finalImage = await this.image(after, signal, "capture-after");
      if (!sameAnchor(before, after) || finalImage.sha256 !== image.sha256) throw new InvalidCustody("PID/image changed during sample");
      if (JSON.stringify(ids) !== JSON.stringify(await this.tasks(before.pid, signal))) throw new MixedSample("TID cohort changed during sample");
      this.known.set(before.pid, { stat: before, image });
      result.push({ pid: before.pid, parentPid: before.parentPid, birthTicks: before.birthTicks, image, statusKiB: status, rollupKiB: rollup, threads: threadSamples });
    }
    // A final ancestry recheck rejects exits/replacements; never splice old/new identities.
    const finalChildren = new Set<number>();
    for (const process of result) {
      const actual = await this.processStat(process.pid, signal);
      if (actual.birthTicks !== process.birthTicks || actual.parentPid !== process.parentPid) throw new InvalidCustody("cohort changed/reused");
      const finalImage = await this.image(actual, signal, "final-cohort");
      if (finalImage.sha256 !== process.image.sha256) throw new InvalidCustody("final cohort image changed");
      const ids = await this.tasks(process.pid, signal);
      if (JSON.stringify(ids) !== JSON.stringify(process.threads.map((thread) => thread.tid))) throw new MixedSample("final TID cohort changed");
      for (const tid of ids) {
        const task = parseProcStat(await this.read(`/proc/${process.pid}/task/${tid}/stat`, signal));
        const recorded = process.threads.find((thread) => thread.tid === tid);
        if (recorded === undefined || task.pid !== tid || task.birthTicks !== recorded.birthTicks) throw new InvalidCustody("final TID reuse");
        const children = (await this.read(`/proc/${process.pid}/task/${tid}/children`, signal)).trim();
        if (children !== "") for (const child of children.split(/\s+/)) {
          const childPid = integer(child); if (childPid <= 0) throw new InvalidCustody("invalid child PID");
          finalChildren.add(childPid);
        }
      }
    }
    const expectedChildren = result.filter((process) => process.pid !== root.pid).map((process) => process.pid).sort((a, b) => a - b);
    if (JSON.stringify([...finalChildren].sort((a, b) => a - b)) !== JSON.stringify(expectedChildren)) throw new MixedSample("descendant cohort changed");
    signal.throwIfAborted(); return result;
  }
  snapshotSummary() {
    return { diagnosticOnly: true, qualificationCredit: 0, invalid: this.invalid, root: this.root ?? null,
      attempts: this.attempts, samples: this.samples, skippedTicks: this.skipped, intervalCallbacks: this.intervalTicks, inFlight: this.busy,
      bytesRead: this.bytes, reads: this.reads, startNs: this.started?.toString() ?? null, endNs: this.ended?.toString() ?? null,
      phaseEvents: this.phases, refusals: this.refusals, stops: this.stops, limits: this.limits,
      limitsNote: "Linux proc reads are asynchronous, not atomic. Abort/deadlines stop new work; an in-flight kernel read may outlive its logical deadline. CPU deltas include the Node harness. Missing short-lived TID is not absence proof. No per-thread RSS." };
  }
}
