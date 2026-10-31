import { createHash } from "node:crypto";
import { mkdirSync, realpathSync, writeFileSync } from "node:fs";
import { open, readlink, stat } from "node:fs/promises";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { parseProcStat, sameAnchor } from "./proc-sampler.ts";
import type { FileIdentity, ImageIdentity, ObservationSink, PhaseEvent, ProcStat } from "./types.ts";

export const SMAPS_LIMITS = Object.freeze({ slots: 4, vmas: 512, totalVmas: 2048, smapsBytes: 512 * 1024,
  mapsBytes: 128 * 1024, smallBytes: 8192, linkBytes: 4096, readBytes: 4 * 1024 * 1024,
  outputBytes: 4 * 1024 * 1024, finalReserveBytes: 16384, deadlineMs: 500, durationMs: 20_000 });
export type SmapsLimits = { [Key in keyof typeof SMAPS_LIMITS]: number };
export interface SmapsHandle { read(maxBytes: number, signal: AbortSignal): Promise<Uint8Array>; close(): Promise<void> }
export interface SmapsReader {
  open(file: string, signal: AbortSignal): Promise<SmapsHandle>;
  link(file: string, signal: AbortSignal): Promise<string>;
  image(file: string, signal: AbortSignal): Promise<FileIdentity>;
  namespace(file: string, signal: AbortSignal): Promise<{ dev: string; ino: string }>;
}
export type VmaHeader = { start: string; end: string; permissions: string; offset: string; device: string; inode: string; pathname: string };
export type Vma = VmaHeader & { kiB: Record<string, number>; thpEligible?: number; vmFlags: string[] };
export type Slot = { label: string; triggerPhase: string; state: "missing" | "busy-gap" | "refused" | "complete";
  triggerNs?: string; startNs?: string; endNs?: string; startPhase?: string; endPhase?: string;
  startEpoch?: number; endEpoch?: number; triggerStartOffsetMs?: number; phaseRelation?: "contained" | "overlap" | "late";
  crossings?: Array<{ phase: string; atNs: string; epoch: number }>; files?: string[]; vmas?: number;
  bytesRead?: number; reads?: number; wallMs?: number; harnessCpuUserUs?: number; harnessCpuSystemUs?: number; reason?: string };
export type SmapsSummary = { schema: "vize.lsp.smaps-snapshots"; version: 1; diagnosticOnly: true; qualificationCredit: 0;
  sourceRevision: string; workflowRevision: string; rootPid: number; rootBirthTicks: string | null;
  mode?: "vben-native"; samplingTarget?: { pid: number; birthTicks: string | null; selectedBirthTicks: string; image: ImageIdentity }; ownerRoot?: SmapsOwner;
  slots: Slot[]; snapshotsValid: number; phaseContained: number; allSlotsAccounted: boolean; allReadsComplete: boolean;
  incomplete: boolean; inFlight: boolean; bytesRead: number; reads: number; outputBytes: number; totalVmas: number;
  bootIdSha256: string | null; startNs: string | null; endNs: string | null; refusals: string[]; limits: SmapsLimits;
  compacted?: boolean; metadataSha256?: string; finalization?: "normal-drain" | "synchronous-exit"; limitations: string };
export type SmapsOwner = { pid: number; birthTicks: string; image: ImageIdentity };
/** rootPid is the sampling target. Native mode separately requires its original Vize owner. */
export type SmapsOptions = { rootPid: number; image: ImageIdentity; outputRoot: string; sourceRevision: string;
  mode?: "server" | "vben-native"; ownerRoot?: SmapsOwner; targetBirthTicks?: string;
  workflowRevision: string; reader?: SmapsReader; now?: () => bigint; limits?: Partial<SmapsLimits> };
const required = ["Size", "Rss", "Pss", "Shared_Clean", "Shared_Dirty", "Private_Clean", "Private_Dirty", "Anonymous"];
const optional = ["LazyFree", "Pss_Dirty", "Referenced", "Swap", "SwapPss", "AnonHugePages", "ShmemPmdMapped",
  "FilePmdMapped", "Shared_Hugetlb", "Private_Hugetlb", "Locked", "KernelPageSize", "MMUPageSize"];
const names = ["pid", "mnt", "user", "time"];
const info = new Set(["uncorrelated-response", "uncorrelated-publication", "ephemeral-version1-publication"]);
const triggers = ["ephemeral-did-open-before-write", "workspace-open-scan-request-before-call", "workspace-open-scan-settled", "workspace-deleted-scan-settled"];
const labels = ["closed-transition-trigger", "beforeScan1-trigger", "afterScan1-trigger", "afterScan2-trigger"];
const nativeTriggers = ["vben-coldOpenSecondApp-settled", "vben-sharedBrokenSecondApp-settled", "vben-sharedRepaired-settled", "vben-sharedRepairedSecondApp-settled"];
const nativeLabels = ["coldOpenSecondApp-settlement-trigger", "sharedBrokenSecondApp-settlement-trigger", "sharedRepaired-settlement-trigger", "sharedRepairedSecondApp-settlement-trigger"];
const message = (error: unknown): string => error instanceof Error ? error.message : String(error);
const digest = (value: Uint8Array | string): string => createHash("sha256").update(value).digest("hex");
function integer(value: string | undefined): number {
  if (value === undefined || !/^\d+$/u.test(value) || !Number.isSafeInteger(Number(value))) throw new Error("unsafe numeric proc field");
  return Number(value);
}
function header(line: string): VmaHeader {
  const m = /^([\da-f]+)-([\da-f]+) ([r-][w-][x-][ps]) ([\da-f]+) ([\da-f]+:[\da-f]+) (\d+)(?:\s+(.*))?$/iu.exec(line);
  if (!m?.[1] || !m[2] || !m[3] || !m[4] || !m[5] || !m[6]) throw new Error("invalid VMA header");
  const start = BigInt(`0x${m[1]}`), end = BigInt(`0x${m[2]}`);
  if (start >= end) throw new Error("invalid VMA address range");
  return { start: start.toString(16), end: end.toString(16), permissions: m[3], offset: BigInt(`0x${m[4]}`).toString(16),
    device: m[5].toLowerCase(), inode: BigInt(m[6]).toString(), pathname: m[7] ?? "" };
}
function ordered(values: VmaHeader[], maximum: number): void {
  if (values.length === 0 || values.length > maximum) throw new Error("VMA count cap/empty layout");
  for (let i = 1; i < values.length; i += 1) {
    const before = values[i - 1], after = values[i];
    if (before === undefined || after === undefined || BigInt(`0x${before.end}`) > BigInt(`0x${after.start}`)) throw new Error("overlapping/unordered VMAs");
  }
}
export function parseMaps(text: string, maximum: number = SMAPS_LIMITS.vmas): VmaHeader[] {
  if (!text.endsWith("\n")) throw new Error("incomplete maps line");
  const values = text.trimEnd().split("\n").map(header); ordered(values, maximum); return values;
}
export function parseSmaps(text: string, maximum: number = SMAPS_LIMITS.vmas): Vma[] {
  if (!text.endsWith("\n")) throw new Error("incomplete smaps line");
  const values: Vma[] = []; let current: Vma | undefined; let flags = false;
  const finish = (): void => {
    const value = current;
    if (value !== undefined && (!flags || required.some((key) => !Object.hasOwn(value.kiB, key)))) throw new Error("incomplete VMA fields");
  };
  for (const line of text.trimEnd().split("\n")) {
    if (/^[\da-f]+-/iu.test(line)) {
      finish(); current = { ...header(line), kiB: {}, vmFlags: [] }; values.push(current); flags = false;
      if (values.length > maximum) throw new Error("VMA count cap"); continue;
    }
    if (current === undefined) throw new Error("smaps field without VMA");
    const key = /^([A-Za-z_]+):/u.exec(line)?.[1];
    if (key !== undefined && [...required, ...optional].includes(key)) {
      const m = /^[A-Za-z_]+:\s+(\d+) kB\s*$/u.exec(line);
      if (Object.hasOwn(current.kiB, key)) throw new Error("duplicate KiB field"); current.kiB[key] = integer(m?.[1]);
    } else if (key === "THPeligible") {
      if (current.thpEligible !== undefined) throw new Error("duplicate THPeligible");
      current.thpEligible = integer(/^THPeligible:\s+(\d+)\s*$/u.exec(line)?.[1]);
    } else if (key === "VmFlags") {
      if (flags) throw new Error("duplicate VmFlags"); flags = true; current.vmFlags = line.slice(line.indexOf(":") + 1).trim().split(/\s+/u).filter(Boolean);
    } else if (!/^[A-Za-z_]+:\s*.*$/u.test(line)) throw new Error("malformed smaps field");
  }
  finish(); ordered(values, maximum); return values;
}
function sameFile(a: FileIdentity, b: FileIdentity): boolean {
  return a.dev === b.dev && a.ino === b.ino && a.bytes === b.bytes && a.mtimeNs === b.mtimeNs && a.ctimeNs === b.ctimeNs;
}
export function runtimeSmapsReader(): SmapsReader {
  return {
    async open(file, signal) {
      signal.throwIfAborted(); const fd = await open(file, "r");
      return { async read(maxBytes, active) {
        active.throwIfAborted(); const buffer = Buffer.alloc(maxBytes); const result = await fd.read(buffer, 0, maxBytes, null);
        return buffer.subarray(0, result.bytesRead);
      }, async close() { await fd.close(); } };
    },
    async link(file, signal) { signal.throwIfAborted(); return await readlink(file); },
    async image(file, signal) {
      signal.throwIfAborted(); const s = await stat(file, { bigint: true }); const bytes = Number(s.size);
      if (!s.isFile() || !Number.isSafeInteger(bytes) || bytes < 0) throw new Error("invalid executable metadata");
      return { dev: s.dev.toString(), ino: s.ino.toString(), bytes, mtimeNs: s.mtimeNs.toString(), ctimeNs: s.ctimeNs.toString() };
    },
    async namespace(file, signal) { signal.throwIfAborted(); const s = await stat(file, { bigint: true }); return { dev: s.dev.toString(), ino: s.ino.toString() }; },
  };
}
type Anchor = { stat: ProcStat; image: ImageIdentity; namespaces: Record<string, { link: string; dev: string; ino: string }>;
  clientNamespaces: Record<string, { link: string; dev: string; ino: string }>; status: Record<string, string>;
  ownerChecks?: { before: Anchor; after: Anchor } };
type Raw = { bytes: Buffer; complete: boolean; error?: string };
/** Four passive intervals, never atomic points. No RPC, process signal or allocator operation. */
export class SmapsSnapshots implements ObservationSink {
  private readonly options: SmapsOptions; private readonly reader: SmapsReader; private readonly now: () => bigint;
  private readonly limits: SmapsLimits; private readonly ready: Promise<void>; private root: Anchor | undefined;
  private owner: Anchor | undefined;
  private readonly slots: Slot[];
  private readonly edges: Array<{ phase: string; atNs: string; epoch: number }> = []; private readonly refusals: string[] = [];
  private phase = "before-lifecycle"; private epoch = 0; private active = false; private invalid = false; private stopped = false;
  private pending: Promise<void> | undefined; private finishing: Promise<SmapsSummary> | undefined; private controller: AbortController | undefined;
  private sealed: SmapsSummary | undefined;
  private bytes = 0; private reads = 0; private outputBytes = 0; private vmas = 0; private boot: string | null = null;
  private started: bigint | undefined; private ended: bigint | undefined; private timer: NodeJS.Timeout | undefined;
  constructor(options: SmapsOptions) {
    if (options.mode !== undefined && options.mode !== "server" && options.mode !== "vben-native") throw new Error("invalid smaps mode");
    const native = options.mode === "vben-native", owner = options.ownerRoot;
    if (native) {
      if (options.image.role !== "native" || owner === undefined || owner.image.role !== "server" || !/^[1-9]\d*$/u.test(options.targetBirthTicks ?? "")
        || !Number.isSafeInteger(owner.pid) || owner.pid <= 0 || owner.pid === options.rootPid || !/^[1-9]\d*$/u.test(owner.birthTicks)
        || owner.image.realpath === options.image.realpath || owner.image.sha256 === options.image.sha256
        || (owner.image.dev === options.image.dev && owner.image.ino === options.image.ino)) throw new Error("invalid native target/owner admission");
    } else if (owner !== undefined || options.image.role !== "server" || options.targetBirthTicks !== undefined) throw new Error("native target requires explicit owner mode");
    this.options = { ...options, image: { ...options.image }, ...(owner === undefined ? {} : { ownerRoot: { ...owner, image: { ...owner.image } } }) };
    this.slots = (native ? nativeTriggers : triggers).map((triggerPhase, i) => ({ label: (native ? nativeLabels : labels)[i] ?? "invalid", triggerPhase, state: "missing" }));
    if (native) this.phase = "before-vben-native-observation";
    this.reader = options.reader ?? runtimeSmapsReader(); this.now = options.now ?? (() => process.hrtime.bigint());
    this.limits = { ...SMAPS_LIMITS, ...options.limits };
    for (const key of Object.keys(SMAPS_LIMITS)) {
      const value: unknown = Reflect.get(this.limits, key), cap: unknown = Reflect.get(SMAPS_LIMITS, key);
      if (typeof value !== "number" || typeof cap !== "number" || !Number.isSafeInteger(value) || value <= 0 || value > cap) throw new Error("invalid lower-only smaps cap");
    }
    if (this.limits.slots !== 4 || this.limits.finalReserveBytes !== 16384 || this.limits.outputBytes <= this.limits.finalReserveBytes) throw new Error("invalid fixed slot/summary caps");
    if (!Number.isSafeInteger(options.rootPid) || options.rootPid <= 0) throw new Error("invalid root image/PID");
    if (!isAbsolute(options.outputRoot) || resolve(options.outputRoot) !== options.outputRoot || realpathSync(dirname(options.outputRoot)) !== dirname(options.outputRoot)) throw new Error("noncanonical smaps output parent");
    mkdirSync(options.outputRoot, { mode: 0o700 });
    this.write("header.json", JSON.stringify({ schema: "vize.lsp.smaps-snapshots", version: 1, diagnosticOnly: true, qualificationCredit: 0,
      sourceRevision: options.sourceRevision, workflowRevision: options.workflowRevision, rootPid: owner?.pid ?? options.rootPid, image: owner?.image ?? options.image,
      ...this.targetMetadata(),
      limits: this.limits, triggers: this.slots.map(({ label, triggerPhase }) => ({ label, triggerPhase })), requestedPointGuaranteed: false }));
    this.ready = this.bind().catch((error: unknown) => this.refuse(`anchor: ${message(error)}`));
  }
  private write(name: string, data: string | Uint8Array, terminal = false): void {
    if (this.sealed !== undefined) throw new Error("smaps output already sealed");
    const count = typeof data === "string" ? Buffer.byteLength(data) : data.byteLength;
    if (this.outputBytes + count > this.limits.outputBytes - (terminal ? 0 : this.limits.finalReserveBytes)) throw new Error("smaps output byte cap");
    writeFileSync(join(this.options.outputRoot, name), data, { flag: "wx", mode: 0o600 }); this.outputBytes += count;
  }
  private check(signal: AbortSignal, deadline: bigint): void {
    if (this.sealed !== undefined) throw new Error("smaps observation already sealed");
    signal.throwIfAborted(); if (this.now() > deadline) throw new Error("smaps logical deadline");
    if (this.started !== undefined && this.now() - this.started > BigInt(this.limits.durationMs) * 1_000_000n) throw new Error("smaps window deadline");
  }
  private async raw(file: string, cap: number, signal: AbortSignal, deadline: bigint): Promise<Raw> {
    const chunks: Buffer[] = []; let length = 0; let fd: SmapsHandle | undefined; let error: string | undefined; let complete = false;
    try {
      this.check(signal, deadline); fd = await this.reader.open(file, signal);
      while (true) {
        this.check(signal, deadline); const remaining = this.limits.readBytes - this.bytes;
        if (remaining <= 0) throw new Error("total smaps read byte cap");
        const maximum = Math.min(65536, cap + 1 - length, remaining); this.reads += 1;
        const chunk = await fd.read(maximum, signal); this.bytes += chunk.byteLength;
        if (chunk.byteLength > maximum) throw new Error("reader exceeded requested cap");
        if (chunk.byteLength !== 0) { chunks.push(Buffer.from(chunk)); length += chunk.byteLength; }
        this.check(signal, deadline); if (length > cap) throw new Error("proc file byte cap");
        if (chunk.byteLength === 0) { complete = true; break; }
      }
    } catch (caught: unknown) { error = message(caught); }
    finally { try { await fd?.close(); } catch (caught: unknown) { complete = false; error = `close: ${message(caught)}`; } }
    return { bytes: Buffer.concat(chunks).subarray(0, cap), complete, ...(error === undefined ? {} : { error }) };
  }
  private text(raw: Raw): string {
    if (!raw.complete) throw new Error(raw.error ?? "incomplete proc read");
    return new TextDecoder("utf-8", { fatal: true }).decode(raw.bytes);
  }
  private async small(file: string, signal: AbortSignal, deadline: bigint): Promise<string> { return this.text(await this.raw(file, this.limits.smallBytes, signal, deadline)); }
  private async link(file: string, signal: AbortSignal, deadline: bigint): Promise<string> {
    this.check(signal, deadline); if (this.limits.readBytes - this.bytes < this.limits.linkBytes) throw new Error("total smaps read byte cap");
    this.reads += 1; const value = await this.reader.link(file, signal); this.bytes += Buffer.byteLength(value);
    if (Buffer.byteLength(value) > this.limits.linkBytes || this.bytes > this.limits.readBytes) throw new Error("proc link byte cap");
    this.check(signal, deadline); return value;
  }
  private async namespaces(pid: number | "self", signal: AbortSignal, deadline: bigint): Promise<Anchor["namespaces"]> {
    const result: Anchor["namespaces"] = {};
    for (const name of names) {
      const file = `/proc/${pid}/ns/${name}`; const link = await this.link(file, signal, deadline);
      if (!new RegExp(`^${name}:\\[\\d+\\]$`, "u").test(link)) throw new Error("invalid namespace membership link");
      this.reads += 1; const identity = await this.reader.namespace(file, signal); this.check(signal, deadline);
      if (!/^\d+$/u.test(identity.dev) || !/^\d+$/u.test(identity.ino)) throw new Error("invalid namespace identity");
      result[name] = { link, ...identity };
    }
    return result;
  }
  private async anchor(signal: AbortSignal, deadline: bigint, pid = this.options.rootPid, expected = this.options.image): Promise<Anchor> {
    const base = `/proc/${pid}`; const value = parseProcStat(await this.small(`${base}/stat`, signal, deadline));
    if (value.pid !== pid) throw new Error("foreign root PID");
    const path = await this.link(`${base}/exe`, signal, deadline); this.reads += 1;
    const image = await this.reader.image(`${base}/exe`, signal); this.check(signal, deadline);
    if (path !== expected.realpath || !sameFile(image, expected)) throw new Error("foreign/changed executable image");
    const status: Record<string, string> = {};
    for (const line of (await this.small(`${base}/status`, signal, deadline)).split("\n")) {
      const m = /^(Tgid|Pid|PPid|NSpid):\s+(\d+(?:\s+\d+)*)\s*$/u.exec(line);
      if (m?.[1] && m[2]) { if (Object.hasOwn(status, m[1])) throw new Error("duplicate status identity"); status[m[1]] = m[2].split(/\s+/u).join(" "); }
    }
    if (status.Pid !== String(value.pid) || status.Tgid !== String(value.pid) || status.PPid !== String(value.parentPid) || status.NSpid === undefined) throw new Error("invalid root status identity");
    const result = { stat: value, image: expected, status, namespaces: await this.namespaces(value.pid, signal, deadline),
      clientNamespaces: await this.namespaces("self", signal, deadline) };
    const final = parseProcStat(await this.small(`${base}/stat`, signal, deadline));
    if (!sameAnchor(value, final)) throw new Error("root changed during anchor"); return result;
  }
  private equal(a: Anchor, b: Anchor): boolean {
    return sameAnchor(a.stat, b.stat) && sameFile(a.image, b.image) && JSON.stringify(a.namespaces) === JSON.stringify(b.namespaces)
      && JSON.stringify(a.clientNamespaces) === JSON.stringify(b.clientNamespaces) && JSON.stringify(a.status) === JSON.stringify(b.status);
  }
  private async checkedAnchor(signal: AbortSignal, deadline: bigint): Promise<Anchor> {
    const owner = this.options.ownerRoot;
    if (owner === undefined) return await this.anchor(signal, deadline);
    const before = await this.anchor(signal, deadline, owner.pid, owner.image);
    if (before.stat.birthTicks !== owner.birthTicks || (this.owner !== undefined && !this.equal(this.owner, before))) throw new Error("owner birth/image/namespace changed");
    const target = await this.anchor(signal, deadline);
    if (target.stat.birthTicks !== this.options.targetBirthTicks || target.stat.parentPid !== owner.pid
      || JSON.stringify(target.namespaces) !== JSON.stringify(before.namespaces)) throw new Error("native target birth/parent/namespace differs from owner selection");
    const after = await this.anchor(signal, deadline, owner.pid, owner.image);
    if (!this.equal(before, after) || after.stat.birthTicks !== owner.birthTicks) throw new Error("owner changed across target identity proof");
    this.owner ??= before; return { ...target, ownerChecks: { before, after } };
  }
  private targetMetadata(): Pick<SmapsSummary, "mode" | "samplingTarget" | "ownerRoot"> {
    const owner = this.options.ownerRoot;
    return owner === undefined ? {} : { mode: "vben-native", ownerRoot: { ...owner, image: { ...owner.image } },
      samplingTarget: { pid: this.options.rootPid, birthTicks: this.root?.stat.birthTicks ?? null, selectedBirthTicks: this.options.targetBirthTicks ?? "", image: { ...this.options.image } } };
  }
  private async bind(): Promise<void> {
    const signal = AbortSignal.timeout(this.limits.deadlineMs), deadline = this.now() + BigInt(this.limits.deadlineMs) * 1_000_000n;
    const before = await this.checkedAnchor(signal, deadline); const boot = await this.small("/proc/sys/kernel/random/boot_id", signal, deadline);
    if (!/^[\da-f]{8}(?:-[\da-f]{4}){3}-[\da-f]{12}\n?$/iu.test(boot)) throw new Error("invalid boot ID");
    const after = await this.checkedAnchor(signal, deadline); if (!this.equal(before, after)) throw new Error("initial root/namespace changed");
    this.root = before; this.boot = digest(boot.trim()); this.write("anchor.json", JSON.stringify({ ...before, bootIdSha256: this.boot }));
  }
  start(rootPid: number): void {
    if (rootPid !== this.options.rootPid) { this.refuse("foreign start PID"); return; }
    if (this.started !== undefined || this.stopped || this.invalid) return;
    this.started = this.now(); this.active = true;
    this.timer = setTimeout(() => this.refuse("smaps window deadline"), this.limits.durationMs); this.timer.unref();
  }
  event(event: PhaseEvent): void {
    try {
      if (this.sealed !== undefined) return;
      if (!/^\d+$/u.test(event.atNs)) throw new Error("invalid event timestamp");
      if (!info.has(event.phase)) {
        this.phase = event.phase; this.epoch += 1;
        if (this.edges.length >= 256) throw new Error("smaps phase edge cap"); this.edges.push({ phase: event.phase, atNs: event.atNs, epoch: this.epoch });
      }
      const slot = this.slots.find((value) => value.triggerPhase === event.phase);
      if (slot === undefined || slot.state !== "missing" || slot.triggerNs !== undefined) return;
      slot.triggerNs = event.atNs;
      if (event.phase.endsWith("settled") && event.outcome !== "fulfilled") { slot.state = "refused"; slot.reason = "original request not fulfilled"; return; }
      if (!this.active || this.invalid || this.stopped) { slot.state = "refused"; slot.reason = "outside active observation window"; return; }
      if (this.pending !== undefined) { slot.state = "busy-gap"; slot.reason = "one attempt already in flight; no retry"; return; }
      const triggerEpoch = this.epoch;
      this.pending = this.capture(slot, triggerEpoch).catch((error: unknown) => { slot.state = "refused"; slot.reason = message(error); })
        .finally(() => { this.pending = undefined; });
    } catch (error: unknown) { this.refuse(message(error)); }
  }
  refuse(reason: string): void {
    this.invalid = true; this.active = false; if (this.refusals.length < 32) this.refusals.push(reason.slice(0, 512));
    this.controller?.abort(); this.stop("refused");
  }
  stop(reason: string): void {
    if (this.stopped) return; this.stopped = true; this.active = false; this.ended = this.now(); this.epoch += 1;
    if (this.timer !== undefined) clearTimeout(this.timer);
    if (this.edges.length < 256) this.edges.push({ phase: `window-stop:${reason.slice(0, 128)}`, atNs: this.ended.toString(), epoch: this.epoch });
  }
  private async capture(slot: Slot, triggerEpoch: number): Promise<void> {
    await this.ready;
    if (this.root === undefined || this.invalid || this.stopped) throw new Error("initial root anchor unavailable/refused or window stopped");
    const start = this.now(), cpu = process.cpuUsage(), startEpoch = this.epoch, startBytes = this.bytes, startReads = this.reads;
    slot.startNs = start.toString(); slot.startPhase = this.phase; slot.startEpoch = startEpoch;
    slot.triggerStartOffsetMs = Number(start - BigInt(slot.triggerNs ?? start.toString())) / 1e6;
    const deadline = start + BigInt(this.limits.deadlineMs) * 1_000_000n; this.controller = new AbortController();
    const timer = setTimeout(() => this.controller?.abort(), this.limits.deadlineMs); timer.unref();
    const signal = this.controller.signal, files: string[] = []; slot.files = files;
    const save = async (kind: string, cap: number): Promise<Raw> => {
      const value = await this.raw(`/proc/${this.options.rootPid}/${kind.startsWith("maps") ? "maps" : "smaps"}`, cap, signal, deadline);
      const name = `${slot.label}.${kind}.txt`; this.write(name, value.bytes); files.push(name);
      if (!value.complete) throw new Error(value.error ?? "incomplete proc read"); return value;
    };
    try {
      this.check(signal, deadline); const before = await this.checkedAnchor(signal, deadline);
      if (!this.equal(this.root, before)) throw new Error("root PID/image/namespace differs from original anchor");
      const mapsBefore = await save("maps-before", this.limits.mapsBytes), smaps = await save("smaps", this.limits.smapsBytes);
      const mapsAfter = await save("maps-after", this.limits.mapsBytes); const after = await this.checkedAnchor(signal, deadline);
      const identityValid = this.equal(this.root, after) && this.equal(before, after);
      const evidence = `${slot.label}.identity.json`; this.write(evidence, JSON.stringify({ before, after, identityValid })); files.push(evidence);
      if (!identityValid) throw new Error("root PID/image/namespace changed across read");
      const first = parseMaps(this.text(mapsBefore), this.limits.vmas), values = parseSmaps(this.text(smaps), this.limits.vmas);
      const last = parseMaps(this.text(mapsAfter), this.limits.vmas), headers = values.map(({ kiB: _kiB, thpEligible: _thp, vmFlags: _flags, ...value }) => value);
      if (JSON.stringify(first) !== JSON.stringify(last) || JSON.stringify(first) !== JSON.stringify(headers)) throw new Error("layout-overlap: maps/smaps header sequences differ");
      if (this.vmas + values.length > this.limits.totalVmas) throw new Error("total VMA cap"); this.vmas += values.length; slot.vmas = values.length;
      const parsed = `${slot.label}.parsed.json`; this.write(parsed, JSON.stringify({ vmas: values,
        raw: [mapsBefore, smaps, mapsAfter].map((value) => ({ bytes: value.bytes.length, sha256: digest(value.bytes), eof: value.complete })),
        atomic: false, requestedPointGuaranteed: false })); files.push(parsed); this.check(signal, deadline); slot.state = "complete";
    } catch (error: unknown) {
      slot.state = "refused"; slot.reason = message(error);
      if (/PID|image|namespace|owner|parent|deadline|byte cap|VMA.*cap/u.test(slot.reason)) this.refuse(slot.reason);
    } finally {
      clearTimeout(timer); this.controller = undefined; const end = this.now(), usage = process.cpuUsage(cpu);
      slot.endNs = end.toString(); slot.endPhase = this.phase; slot.endEpoch = this.epoch;
      slot.crossings = this.edges.filter((edge) => edge.epoch > startEpoch);
      slot.phaseRelation = startEpoch !== triggerEpoch ? "late" : this.epoch !== startEpoch || (this.ended !== undefined && end > this.ended) ? "overlap" : "contained";
      slot.bytesRead = this.bytes - startBytes; slot.reads = this.reads - startReads; slot.wallMs = Number(end - start) / 1e6;
      slot.harnessCpuUserUs = usage.user; slot.harnessCpuSystemUs = usage.system;
    }
  }
  snapshotSummary(): SmapsSummary {
    if (this.sealed !== undefined) return this.sealed;
    const allReadsComplete = this.slots.every((slot) => slot.state === "complete");
    return { schema: "vize.lsp.smaps-snapshots", version: 1, diagnosticOnly: true, qualificationCredit: 0,
      sourceRevision: this.options.sourceRevision, workflowRevision: this.options.workflowRevision, rootPid: this.options.ownerRoot?.pid ?? this.options.rootPid,
      rootBirthTicks: (this.options.ownerRoot === undefined ? this.root : this.owner)?.stat.birthTicks ?? null,
      ...this.targetMetadata(), slots: this.slots.map((slot) => ({ ...slot })),
      snapshotsValid: this.slots.filter((slot) => slot.state === "complete").length,
      phaseContained: this.slots.filter((slot) => slot.state === "complete" && slot.phaseRelation === "contained").length,
      allSlotsAccounted: this.slots.every((slot) => slot.state !== "missing"), allReadsComplete,
      incomplete: this.invalid || this.pending !== undefined || !allReadsComplete, inFlight: this.pending !== undefined,
      bytesRead: this.bytes, reads: this.reads, outputBytes: this.outputBytes, totalVmas: this.vmas, bootIdSha256: this.boot,
      startNs: this.started?.toString() ?? null, endNs: this.ended?.toString() ?? null, refusals: [...this.refusals], limits: this.limits,
      limitations: this.options.ownerRoot === undefined
        ? "Passive root-only intervals, not atomic points or Rust fences. Closed trigger may overlap didOpen. Stable maps cannot exclude ABA. File mappings may contain anonymous COW; LazyFree is not returned RSS/live heap. No allocator ownership or per-thread RSS. Node CPU includes concurrent harness work. Logical deadlines cannot cancel blocked kernel reads."
        : "Passive one-native-target intervals; Vize owner identity reads only, not atomic points or Rust fences. Settlement trigger is not worker quiescence. Stable maps cannot exclude ABA. File mappings may contain anonymous COW; LazyFree is not returned RSS/live heap. No allocator ownership or per-thread RSS. Node CPU includes concurrent harness work. Logical deadlines cannot cancel blocked kernel reads." };
  }
  finish(): Promise<SmapsSummary> {
    if (this.finishing !== undefined) return this.finishing;
    this.stop("finalize"); this.finishing = (async () => {
      await this.ready; await this.pending; return this.seal("normal-drain");
    })(); return this.finishing;
  }
  finishOnExit(): void {
    if (this.sealed !== undefined) return; this.stop("process-exit");
    try { this.seal("synchronous-exit"); } catch (error: unknown) { this.refuse(`exit summary: ${message(error)}`); }
  }
  private seal(finalization: "normal-drain" | "synchronous-exit"): SmapsSummary {
    if (this.sealed !== undefined) return this.sealed;
    let summary = { ...this.snapshotSummary(), finalization }; let text = JSON.stringify(summary);
    if (Buffer.byteLength(text) > this.limits.finalReserveBytes - 128) {
      summary = { ...summary, incomplete: true, compacted: true, metadataSha256: digest(text), refusals: ["terminal metadata compacted"],
        slots: summary.slots.map((slot) => ({ label: slot.label, triggerPhase: slot.triggerPhase, state: slot.state, ...(slot.phaseRelation === undefined ? {} : { phaseRelation: slot.phaseRelation }) })) };
    }
    for (let i = 0; i < 3; i += 1) { text = JSON.stringify(summary); summary.outputBytes = this.outputBytes + Buffer.byteLength(text); }
    text = JSON.stringify(summary);
    if (Buffer.byteLength(text) > this.limits.finalReserveBytes) throw new Error("reserved terminal smaps summary cap");
    this.write("summary.json", text, true); this.sealed = summary; return summary;
  }
}
