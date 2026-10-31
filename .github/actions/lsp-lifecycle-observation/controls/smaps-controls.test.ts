import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, readdirSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { parseMaps, parseSmaps, SMAPS_LIMITS, SmapsSnapshots } from "../smaps-snapshots.ts";
import type { SmapsLimits, SmapsOptions, SmapsReader, SmapsSummary } from "../smaps-snapshots.ts";
import type { FileIdentity, ImageIdentity, PhaseEvent } from "../types.ts";

const image: ImageIdentity = { role: "server", realpath: "/fixture/vize", sha256: "a".repeat(64),
  dev: "1", ino: "2", bytes: 100, mtimeNs: "3", ctimeNs: "4" };
const nativeImage: ImageIdentity = { ...image, role: "native", realpath: "/fixture/tsgo", sha256: "b".repeat(64), ino: "20" };
const nativePhases = ["vben-coldOpenSecondApp-settled", "vben-sharedBrokenSecondApp-settled", "vben-sharedRepaired-settled", "vben-sharedRepairedSecondApp-settled"];
const map = "1000-2000 rw-p 00000000 00:00 0 [heap]\n";
const fields = "Size: 4 kB\nRss: 3 kB\nPss: 2 kB\nShared_Clean: 0 kB\nShared_Dirty: 0 kB\nPrivate_Clean: 1 kB\nPrivate_Dirty: 2 kB\nAnonymous: 3 kB\n";
const smaps = `${map}${fields}LazyFree: 0 kB\nTHPeligible: 0\nUnknownNewKernelField: 7 kB\nVmFlags: rd wr mr mw ac\n`;
const phases = ["ephemeral-did-open-before-write", "workspace-open-scan-request-before-call", "workspace-open-scan-settled", "workspace-deleted-scan-settled"];
function statText(birth: string): string {
  const values = Array.from({ length: 36 }, () => "0"); values[0] = "1"; values[10] = "2"; values[11] = "3"; values[18] = birth;
  return `100 (vize) S ${values.join(" ")}\n`;
}
function deferred() {
  let resolve: () => void = () => { throw new Error("uninitialized deferred"); };
  const promise = new Promise<void>((done) => { resolve = done; }); return { promise, resolve };
}
class FakeReader implements SmapsReader {
  birth = "123"; parent = "1"; executable = image.realpath; metadata: FileIdentity = { ...image };
  namespaceId = "12"; clientNamespaceId = "12"; rootPid = "100"; statusPid = "100";
  maps = map; mapsAfter: string | undefined; smaps = smaps; chunk = 37; nowNs = 1_000_000n; tickNs = 1000n;
  opens: string[] = []; closes: string[] = []; reads = 0; smapsReads = 0; mapsOpens = 0;
  onSmaps: (() => void) | undefined; block: ReturnType<typeof deferred> | undefined;
  failOpen: string | undefined; failRead = false; invalidUtf8 = false; namespaceMissing = false;
  namespaceDuringRead: (() => void) | undefined;
  hasOwner = false; ownerMissing = false; ownerBirth = "900"; ownerParent = "1"; ownerExecutable = image.realpath;
  ownerMetadata: FileIdentity = { ...image }; ownerNamespaceId = "12";
  onTargetStat: (() => void) | undefined;
  private ownerPath(file: string): boolean { return this.hasOwner && file.startsWith("/proc/10/"); }
  async open(file: string, signal: AbortSignal) {
    signal.throwIfAborted(); this.opens.push(file);
    if (this.failOpen !== undefined && file.endsWith(this.failOpen)) throw new Error("synthetic permission failure");
    let text: string;
    if (this.ownerPath(file)) {
      if (this.ownerMissing) throw new Error("synthetic missing owner");
      if (file.endsWith("/stat")) text = statText(this.ownerBirth).replace(/^100 /u, "10 ").replace(" S 1 ", ` S ${this.ownerParent} `);
      else if (file.endsWith("/status")) text = `Tgid:\t10\nPid:\t10\nPPid:\t${this.ownerParent}\nNSpid:\t10 1\n`;
      else throw new Error("forbidden owner maps/smaps read");
    } else if (file.endsWith("/stat")) { this.onTargetStat?.(); text = statText(this.birth).replace(/^100 /u, `${this.rootPid} `).replace(" S 1 ", ` S ${this.parent} `); }
    else if (file.endsWith("/status")) text = `Tgid:\t${this.statusPid}\nPid:\t${this.statusPid}\nPPid:\t${this.parent}\nNSpid:\t${this.statusPid} 1\n`;
    else if (file.endsWith("/boot_id")) text = "00000000-0000-0000-0000-000000000001\n";
    else if (file.endsWith("/maps")) { this.mapsOpens += 1; text = this.mapsOpens % 2 === 0 ? this.mapsAfter ?? this.maps : this.maps; }
    else if (file.endsWith("/smaps")) text = this.smaps;
    else throw new Error(`unexpected fake file: ${file}`);
    const bytes = this.invalidUtf8 && file.endsWith("/smaps") ? Buffer.from([0xff, 0x0a]) : Buffer.from(text); let offset = 0;
    return { read: async (maximum: number, active: AbortSignal): Promise<Uint8Array> => {
      active.throwIfAborted(); this.reads += 1; this.nowNs += this.tickNs;
      if (file.endsWith("/smaps")) {
        this.smapsReads += 1;
        if (offset === 0) { this.onSmaps?.(); this.namespaceDuringRead?.(); if (this.block !== undefined) await this.block.promise; }
        if (this.failRead && this.smapsReads > 1) throw new Error("synthetic partial read failure");
      }
      const result = bytes.subarray(offset, offset + Math.min(maximum, this.chunk)); offset += result.length; return result;
    }, close: async (): Promise<void> => { this.closes.push(file); } };
  }
  async link(file: string, signal: AbortSignal): Promise<string> {
    signal.throwIfAborted(); this.nowNs += this.tickNs;
    if (file.endsWith("/exe")) return this.ownerPath(file) ? this.ownerExecutable : this.executable;
    const name = file.split("/").at(-1); if (name === undefined) throw new Error("fake namespace name");
    if (this.namespaceMissing) throw new Error("synthetic missing namespace");
    return `${name}:[${this.ownerPath(file) ? this.ownerNamespaceId : file.includes("/self/") ? this.clientNamespaceId : this.namespaceId}]`;
  }
  async image(file: string, signal: AbortSignal): Promise<FileIdentity> { signal.throwIfAborted(); return this.ownerPath(file) ? this.ownerMetadata : this.metadata; }
  async namespace(file: string, signal: AbortSignal): Promise<{ dev: string; ino: string }> {
    signal.throwIfAborted(); return { dev: "1", ino: this.ownerPath(file) ? this.ownerNamespaceId : file.includes("/self/") ? this.clientNamespaceId : this.namespaceId };
  }
}
function fixture(limits?: Partial<SmapsLimits>) {
  const parent = realpathSync(mkdtempSync(join(tmpdir(), "vize-smaps-control-"))), outputRoot = join(parent, "vma"); const reader = new FakeReader();
  const collector = new SmapsSnapshots({ rootPid: 100, image, outputRoot, sourceRevision: "b".repeat(40), workflowRevision: "c".repeat(40),
    reader, now: () => reader.nowNs, ...(limits === undefined ? {} : { limits }) });
  return { parent, outputRoot, reader, collector, clean: () => rmSync(parent, { recursive: true, force: true }) };
}
async function idle(collector: SmapsSnapshots): Promise<void> {
  for (let i = 0; i < 1000; i += 1) { await new Promise<void>((done) => setImmediate(done)); if (!collector.snapshotSummary().inFlight) return; }
  throw new Error("fake collector did not settle");
}
async function anchored(value: ReturnType<typeof fixture>): Promise<void> {
  for (let i = 0; i < 1000; i += 1) {
    await new Promise<void>((done) => setImmediate(done)); const summary = value.collector.snapshotSummary();
    const ready = summary.mode === "vben-native" ? summary.samplingTarget?.birthTicks != null : summary.rootBirthTicks !== null;
    if (ready || summary.refusals.length !== 0) return;
  }
  throw new Error("fake anchor did not settle");
}
function edge(value: ReturnType<typeof fixture>, index: number, extra?: Partial<PhaseEvent>): void {
  const phase = phases[index]; if (phase === undefined) throw new Error("invalid fixture slot");
  value.collector.event({ phase, atNs: value.reader.nowNs.toString(), ...(index >= 2 ? { outcome: "fulfilled" } : {}), ...extra });
}
async function complete(value: ReturnType<typeof fixture>): Promise<SmapsSummary> {
  await anchored(value); value.collector.start(100);
  for (let i = 0; i < 4; i += 1) { edge(value, i); await idle(value.collector); }
  return await value.collector.finish();
}
test("parser preserves big addresses, pathname spaces/deleted, optional presence and COW fields", () => {
  const header = "ffff800000001000-ffff800000002000 rw-p 00000000 08:01 123 /file with spaces (deleted)\n";
  const result = parseSmaps(`${header}${fields}VmFlags: rd wr\n`)[0]; assert.ok(result);
  assert.equal(result.start, "ffff800000001000"); assert.equal(result.pathname, "/file with spaces (deleted)");
  assert.equal(result.kiB.Anonymous, 3); assert.equal(Object.hasOwn(result.kiB, "LazyFree"), false);
  assert.equal(parseSmaps(smaps)[0]?.kiB.LazyFree, 0); assert.equal(parseSmaps(smaps)[0]?.thpEligible, 0);
  assert.deepEqual(parseMaps(header)[0], { start: "ffff800000001000", end: "ffff800000002000", permissions: "rw-p", offset: "0", device: "08:01", inode: "123", pathname: "/file with spaces (deleted)" });
});
function nativeFixture(change?: (reader: FakeReader) => void, changeOptions?: (options: SmapsOptions) => void) {
  const parent = realpathSync(mkdtempSync(join(tmpdir(), "vize-native-smaps-"))), outputRoot = join(parent, "vma"), reader = new FakeReader();
  reader.hasOwner = true; reader.parent = "10"; reader.executable = nativeImage.realpath; reader.metadata = { ...nativeImage }; change?.(reader);
  const options: SmapsOptions = { rootPid: 100, image: nativeImage, mode: "vben-native", targetBirthTicks: "123",
    ownerRoot: { pid: 10, birthTicks: "900", image }, outputRoot, sourceRevision: "ccea9b862fb20b3dab9e2f04903a22bb503cab3e",
    workflowRevision: "c".repeat(40), reader, now: () => reader.nowNs }; changeOptions?.(options);
  try { return { parent, outputRoot, reader, options, collector: new SmapsSnapshots(options), clean: () => rmSync(parent, { recursive: true, force: true }) }; }
  catch (error: unknown) { rmSync(parent, { recursive: true, force: true }); throw error; }
}
function nativeEdge(f: ReturnType<typeof nativeFixture>, i: number, outcome = "fulfilled"): void {
  const phase = nativePhases[i]; assert.ok(phase !== undefined); f.collector.event({ phase, atNs: f.reader.nowNs.toString(), outcome });
}
test("native mode retains native role/selected birth, validates Vize owner and reads only target VMAs", async () => {
  const f = nativeFixture(); try {
    await anchored(f); f.collector.start(100); for (let i = 0; i < 4; i += 1) { nativeEdge(f, i); await idle(f.collector); }
    const summary = await f.collector.finish(); assert.equal(summary.allReadsComplete, true); assert.equal(summary.snapshotsValid, 4);
    assert.equal(summary.mode, "vben-native"); assert.equal(summary.rootPid, 10); assert.equal(summary.rootBirthTicks, "900");
    assert.deepEqual(summary.ownerRoot, { pid: 10, birthTicks: "900", image });
    assert.deepEqual(summary.samplingTarget, { pid: 100, birthTicks: "123", selectedBirthTicks: "123", image: nativeImage });
    assert.deepEqual(summary.slots.map((slot) => slot.triggerPhase), nativePhases); assert.equal(summary.samplingTarget?.image.role, "native");
    assert.ok(f.reader.opens.filter((file) => /\/(?:maps|smaps)$/u.test(file)).every((file) => file.startsWith("/proc/100/")));
    assert.ok(f.reader.opens.every((file) => /^\/proc\/(?:100|10|sys)\//u.test(file))); assert.deepEqual(summary.limits, SMAPS_LIMITS);
    assert.deepEqual(f.reader.closes.sort(), f.reader.opens.sort());
    const header: unknown = JSON.parse(readFileSync(join(f.outputRoot, "header.json"), "utf8")); assert.ok(header !== null && typeof header === "object");
    assert.equal(Reflect.get(header, "rootPid"), 10); assert.match(summary.limitations, /one-native-target/u);
  } finally { f.clean(); }
});
test("native constructor refuses missing owner/selection, server relabel/alias and target-owner PID alias", () => {
  for (const mutate of [(o: SmapsOptions) => { delete o.ownerRoot; }, (o: SmapsOptions) => { delete o.targetBirthTicks; },
    (o: SmapsOptions) => { o.image = image; }, (o: SmapsOptions) => { o.image = { ...image, role: "native" }; },
    (o: SmapsOptions) => { if (o.ownerRoot !== undefined) o.ownerRoot.image = nativeImage; },
    (o: SmapsOptions) => { o.rootPid = 10; }]) assert.throws(() => nativeFixture(undefined, mutate), /native/u);
});
test("native selection authority cannot be rewritten through caller options or summary aliases", async () => {
  const f = nativeFixture(); try {
    await anchored(f); const exposed = f.collector.snapshotSummary();
    if (exposed.ownerRoot !== undefined) { exposed.ownerRoot.pid = 999; exposed.ownerRoot.image.ino = "999"; }
    if (exposed.samplingTarget !== undefined) { exposed.samplingTarget.image.role = "server"; exposed.samplingTarget.image.ino = "999"; }
    f.options.rootPid = 999; f.options.targetBirthTicks = "999";
    if (f.options.ownerRoot !== undefined) f.options.ownerRoot.pid = 999;
    f.collector.start(100); nativeEdge(f, 0); await idle(f.collector); const summary = await f.collector.finish();
    assert.equal(summary.snapshotsValid, 1); assert.equal(summary.rootPid, 10); assert.equal(summary.samplingTarget?.pid, 100);
    assert.equal(summary.samplingTarget?.image.role, "native"); assert.equal(summary.samplingTarget?.selectedBirthTicks, "123");
    assert.ok(f.reader.opens.every((file) => /^\/proc\/(?:100|10|sys)\//u.test(file)));
  } finally { f.clean(); }
});
test("native owner/target admission failures occur before any maps or smaps", async () => {
  for (const mutate of [(r: FakeReader) => { r.ownerMissing = true; }, (r: FakeReader) => { r.parent = "11"; },
    (r: FakeReader) => { r.birth = "124"; }, (r: FakeReader) => { r.ownerBirth = "901"; },
    (r: FakeReader) => { r.ownerExecutable = "/foreign/vize"; }, (r: FakeReader) => { r.ownerMetadata = { ...image, ino: "99" }; },
    (r: FakeReader) => { r.ownerNamespaceId = "13"; }, (r: FakeReader) => { r.onTargetStat = () => { r.ownerBirth = "901"; }; }]) {
    const f = nativeFixture(mutate); try {
      await anchored(f); f.collector.start(100); nativeEdge(f, 0); await idle(f.collector); const summary = await f.collector.finish();
      assert.equal(summary.snapshotsValid, 0); assert.equal(summary.incomplete, true);
      assert.equal(f.reader.opens.some((file) => /\/(?:maps|smaps)$/u.test(file)), false);
    } finally { f.clean(); }
  }
});
test("native owner changes after initial anchor are refused before first maps", async () => {
  for (const mutate of [(r: FakeReader) => { r.ownerBirth = "902"; }, (r: FakeReader) => { r.ownerMissing = true; },
    (r: FakeReader) => { r.ownerExecutable = "/foreign/changed"; }, (r: FakeReader) => { r.ownerNamespaceId = "99"; },
    (r: FakeReader) => { r.parent = "11"; }]) {
    const f = nativeFixture(); try {
      await anchored(f); mutate(f.reader); f.collector.start(100); nativeEdge(f, 0); await idle(f.collector); const summary = await f.collector.finish();
      assert.equal(summary.snapshotsValid, 0); assert.equal(f.reader.mapsOpens, 0); assert.equal(summary.incomplete, true);
    } finally { f.clean(); }
  }
});
test("native post-read reparent/ownerbirth/image/namespace changes preserve raw but never admit", async () => {
  for (const mutate of [(r: FakeReader) => { r.parent = "11"; }, (r: FakeReader) => { r.ownerBirth = "999"; },
    (r: FakeReader) => { r.ownerExecutable = "/foreign/new-image"; }, (r: FakeReader) => { r.ownerNamespaceId = "99"; }]) {
    const f = nativeFixture(); try {
      await anchored(f); f.collector.start(100); f.reader.onSmaps = () => mutate(f.reader); nativeEdge(f, 0); await idle(f.collector);
      const summary = await f.collector.finish(); assert.equal(summary.snapshotsValid, 0); assert.equal(summary.incomplete, true);
      assert.equal(readFileSync(join(f.outputRoot, "coldOpenSecondApp-settlement-trigger.smaps.txt"), "utf8"), smaps);
    } finally { f.clean(); }
  }
});
test("native mode ignores old server triggers and never consumes a rejected settlement as a sample", async () => {
  const f = nativeFixture(); try {
    await anchored(f); f.collector.start(100); edge(f, 0); nativeEdge(f, 0, "rejected"); nativeEdge(f, 0); await idle(f.collector);
    const summary = await f.collector.finish(); assert.equal(summary.slots[0]?.state, "refused"); assert.equal(summary.snapshotsValid, 0);
    assert.equal(f.reader.mapsOpens, 0); assert.equal(summary.slots[1]?.state, "missing");
  } finally { f.clean(); }
});
test("native owner identity I/O is charged to the unchanged global read/deadline caps", async () => {
  const f = nativeFixture(undefined, (o) => { o.limits = { readBytes: 12000 }; }); try {
    await anchored(f); f.collector.start(100);
    for (let i = 0; i < 4; i += 1) { nativeEdge(f, i); await idle(f.collector); if (f.collector.snapshotSummary().refusals.length !== 0) break; }
    const summary = f.collector.snapshotSummary(); assert.equal(summary.incomplete, true); assert.ok(summary.snapshotsValid < 4); assert.ok(summary.bytesRead <= 12000);
    assert.match(summary.refusals.join(" "), /byte cap/u); const reads = f.reader.reads, maps = f.reader.mapsOpens;
    nativeEdge(f, 1); await idle(f.collector); assert.equal(f.reader.reads, reads); assert.equal(f.reader.mapsOpens, maps); await f.collector.finish();
  } finally { f.clean(); }
  const deadline = nativeFixture(); try {
    await anchored(deadline); deadline.reader.tickNs = 50_000_000n; deadline.collector.start(100); nativeEdge(deadline, 0); await idle(deadline.collector);
    const summary = await deadline.collector.finish(); assert.equal(summary.incomplete, true); assert.match(summary.slots[0]?.reason ?? "", /deadline/u);
    assert.equal(deadline.reader.mapsOpens, 0); assert.equal(summary.limits.deadlineMs, 500);
  } finally { deadline.clean(); }
});
test("parsers refuse malformed/overlapping/truncated/duplicate/unsafe fields and VMA caps", () => {
  for (const source of [smaps.slice(0, -1), smaps.replace("Anonymous: 3 kB\n", ""), smaps.replace("Rss: 3 kB", "Rss: 3 MB"),
    smaps.replace("Rss: 3 kB", "Rss: 9007199254740992 kB"), smaps.replace("Rss: 3 kB", "Rss: 3 kB\nRss: 2 kB"),
    smaps.replace("VmFlags: rd wr mr mw ac\n", ""), smaps.replace("1000-2000", "2000-1000"), `${smaps}${smaps}`]) assert.throws(() => parseSmaps(source));
  assert.throws(() => parseMaps(`${map}${map}`), /overlapping/u); assert.throws(() => parseMaps(map.slice(0, -1)), /incomplete/u);
  assert.throws(() => parseSmaps(smaps, 0), /cap/u); assert.throws(() => parseMaps(""));
});
test("four fixed slots use same-FD short reads to EOF, preserve raw hashes, namespace/birth and byte accounting", async () => {
  const f = fixture(); try {
    f.reader.chunk = 7; const summary = await complete(f);
    assert.equal(summary.incomplete, false); assert.equal(summary.snapshotsValid, 4); assert.equal(summary.phaseContained, 4);
    assert.equal(summary.rootBirthTicks, "123"); assert.equal(summary.allSlotsAccounted, true); assert.equal(summary.totalVmas, 4);
    assert.equal(f.reader.opens.filter((file) => file.endsWith("/smaps")).length, 4); assert.equal(f.reader.smapsReads > 100, true);
    assert.deepEqual(f.reader.closes.sort(), f.reader.opens.sort());
    assert.equal(readFileSync(join(f.outputRoot, "beforeScan1-trigger.smaps.txt"), "utf8"), smaps);
    const disk: unknown = JSON.parse(readFileSync(join(f.outputRoot, "summary.json"), "utf8")); assert.deepEqual(disk, summary);
    assert.equal(summary.outputBytes, readdirSync(f.outputRoot).reduce((sum, name) => sum + readFileSync(join(f.outputRoot, name)).length, 0));
    assert.equal(summary.qualificationCredit, 0); assert.match(summary.limitations, /not atomic/u);
    assert.equal(await f.collector.finish(), summary);
  } finally { f.clean(); }
});
test("busy targets are consumed once without queue, retry, or another smaps FD", async () => {
  const f = fixture(); try {
    await anchored(f); f.collector.start(100); f.reader.block = deferred(); edge(f, 0);
    await new Promise<void>((done) => setImmediate(done)); edge(f, 1); f.reader.block.resolve(); await idle(f.collector);
    edge(f, 1); await idle(f.collector); const summary = await f.collector.finish();
    assert.equal(summary.slots[1]?.state, "busy-gap"); assert.equal(summary.snapshotsValid, 1);
    assert.equal(f.reader.opens.filter((file) => file.endsWith("/smaps")).length, 1); assert.equal(summary.allReadsComplete, false);
  } finally { f.clean(); }
});
test("ABA phase edges overlap even with identical end phase; informational events do not", async () => {
  const f = fixture(); try {
    await anchored(f); f.collector.start(100);
    f.reader.onSmaps = () => {
      f.collector.event({ phase: "other", atNs: f.reader.nowNs.toString() });
      f.collector.event({ phase: phases[0] ?? "", atNs: f.reader.nowNs.toString() });
    };
    edge(f, 0); await idle(f.collector); assert.equal(f.collector.snapshotSummary().slots[0]?.phaseRelation, "overlap");
    f.reader.onSmaps = () => f.collector.event({ phase: "ephemeral-version1-publication", atNs: f.reader.nowNs.toString() });
    edge(f, 1); await idle(f.collector); assert.equal(f.collector.snapshotSummary().slots[1]?.phaseRelation, "contained");
    await f.collector.finish();
  } finally { f.clean(); }
});
test("normal stop drains an in-flight FD and marks its interval overlap, not a point", async () => {
  const f = fixture(); try {
    await anchored(f); f.collector.start(100); f.reader.block = deferred(); edge(f, 0);
    await new Promise<void>((done) => setImmediate(done)); f.collector.stop("normal-first-cycle-complete");
    const first = f.collector.finish(); assert.equal(f.collector.finish(), first); assert.equal(f.collector.snapshotSummary().inFlight, true);
    f.reader.nowNs += 5000n; f.reader.block.resolve(); const summary = await first;
    assert.equal(summary.slots[0]?.state, "complete"); assert.equal(summary.slots[0]?.phaseRelation, "overlap");
    assert.deepEqual(f.reader.closes.sort(), f.reader.opens.sort());
  } finally { f.clean(); }
});
test("a delayed anchor cannot produce a snapshot after normal window stop", async () => {
  const f = fixture(); try {
    f.collector.start(100); edge(f, 0); f.collector.stop("immediate-stop"); const summary = await f.collector.finish();
    assert.equal(summary.snapshotsValid, 0); assert.match(summary.slots[0]?.reason ?? "", /window stopped/u);
    assert.equal(f.reader.opens.some((file) => file.endsWith("/smaps")), false);
  } finally { f.clean(); }
});
test("late trigger execution is retained distinctly from phase-contained evidence", async () => {
  const f = fixture(); try {
    f.collector.start(100); edge(f, 0); f.collector.event({ phase: "following-edge", atNs: f.reader.nowNs.toString() });
    await idle(f.collector); const summary = await f.collector.finish();
    assert.equal(summary.slots[0]?.state, "complete"); assert.equal(summary.slots[0]?.phaseRelation, "late"); assert.equal(summary.phaseContained, 0);
  } finally { f.clean(); }
});
test("synchronous exit seals pending/incomplete summary once; delayed read cannot add or replace output", async () => {
  const f = fixture(); try {
    await anchored(f); f.collector.start(100); f.reader.block = deferred(); edge(f, 0);
    await new Promise<void>((done) => setImmediate(done)); assert.equal(f.collector.snapshotSummary().inFlight, true);
    f.collector.finishOnExit(); const files = readdirSync(f.outputRoot), bytes = readFileSync(join(f.outputRoot, "summary.json"));
    const summary = f.collector.snapshotSummary(); assert.equal(summary.incomplete, true); assert.equal(summary.inFlight, true);
    assert.equal(summary.finalization, "synchronous-exit"); f.collector.finishOnExit(); f.reader.block.resolve();
    assert.equal(await f.collector.finish(), summary); assert.deepEqual(readdirSync(f.outputRoot), files);
    assert.deepEqual(readFileSync(join(f.outputRoot, "summary.json")), bytes); assert.equal(f.reader.mapsOpens, 1);
    assert.deepEqual(f.reader.closes.sort(), f.reader.opens.sort());
  } finally { f.clean(); }
});
test("PID reuse, changed parent, foreign PID/status and ELF metadata refuse original-root attribution", async () => {
  for (const mutate of [(r: FakeReader) => { r.birth = "124"; }, (r: FakeReader) => { r.parent = "2"; },
    (r: FakeReader) => { r.rootPid = "101"; }, (r: FakeReader) => { r.statusPid = "101"; },
    (r: FakeReader) => { r.executable = "/foreign/image"; }, (r: FakeReader) => { r.metadata = { ...image, ino: "9" }; }]) {
    const f = fixture(); try {
      await anchored(f); f.collector.start(100); mutate(f.reader); edge(f, 0); await idle(f.collector); const summary = await f.collector.finish();
      assert.equal(summary.snapshotsValid, 0); assert.equal(summary.incomplete, true); assert.equal(summary.slots[0]?.state, "refused");
    } finally { f.clean(); }
  }
});
test("root/client namespace swaps and missing required namespace refuse, preserving raw on mid-read change", async () => {
  for (const mutate of [(r: FakeReader) => { r.namespaceId = "13"; }, (r: FakeReader) => { r.clientNamespaceId = "13"; },
    (r: FakeReader) => { r.namespaceMissing = true; }]) {
    const f = fixture(); try {
      await anchored(f); f.collector.start(100); f.reader.namespaceDuringRead = () => mutate(f.reader); edge(f, 0); await idle(f.collector);
      const summary = await f.collector.finish(); assert.equal(summary.snapshotsValid, 0); assert.equal(summary.incomplete, true);
      assert.equal(readFileSync(join(f.outputRoot, "closed-transition-trigger.smaps.txt"), "utf8"), smaps);
    } finally { f.clean(); }
  }
});
test("PID/ELF changes after smaps read refuse despite preserved complete raw bytes", async () => {
  for (const mutate of [(r: FakeReader) => { r.birth = "999"; }, (r: FakeReader) => { r.executable = "/foreign/new-image"; },
    (r: FakeReader) => { r.metadata = { ...image, ctimeNs: "100" }; }]) {
    const f = fixture(); try {
      await anchored(f); f.collector.start(100); f.reader.onSmaps = () => mutate(f.reader); edge(f, 0); await idle(f.collector);
      const summary = await f.collector.finish(); assert.equal(summary.snapshotsValid, 0);
      assert.match(summary.slots[0]?.reason ?? "", /root|image/u); assert.equal(summary.incomplete, true);
      assert.equal(readFileSync(join(f.outputRoot, "closed-transition-trigger.smaps.txt"), "utf8"), smaps);
    } finally { f.clean(); }
  }
});
test("layout mutation is refused with bounded raw pre/post maps retained", async () => {
  const f = fixture(); try {
    await anchored(f); f.collector.start(100); f.reader.mapsAfter = map.replace("rw-p", "r--p"); edge(f, 0); await idle(f.collector);
    const summary = await f.collector.finish(); assert.match(summary.slots[0]?.reason ?? "", /layout-overlap/u);
    assert.equal(readFileSync(join(f.outputRoot, "closed-transition-trigger.maps-after.txt"), "utf8"), f.reader.mapsAfter);
  } finally { f.clean(); }
});
test("smaps EOF overflow preserves bounded prefix and stops new reads; failed reads close descriptors", async () => {
  const f = fixture({ smapsBytes: 64 }); try {
    await anchored(f); f.collector.start(100); edge(f, 0); await idle(f.collector); const summary = await f.collector.finish();
    assert.match(summary.slots[0]?.reason ?? "", /byte cap/u); assert.equal(f.reader.mapsOpens, 1);
    assert.equal(readFileSync(join(f.outputRoot, "closed-transition-trigger.smaps.txt")).length, 64);
    assert.deepEqual(f.reader.closes.sort(), f.reader.opens.sort()); assert.equal(summary.bytesRead <= SMAPS_LIMITS.readBytes, true);
  } finally { f.clean(); }
  const failed = fixture(); try {
    await anchored(failed); failed.collector.start(100); failed.reader.failRead = true; edge(failed, 0); await idle(failed.collector);
    const summary = await failed.collector.finish(); assert.match(summary.slots[0]?.reason ?? "", /partial read failure/u);
    assert.equal(readFileSync(join(failed.outputRoot, "closed-transition-trigger.smaps.txt")).length, failed.reader.chunk);
    assert.deepEqual(failed.reader.closes.sort(), failed.reader.opens.sort());
  } finally { failed.clean(); }
});
test("invalid UTF8, truncated VMA and schema failure never admit a raw prefix", async () => {
  for (const mode of ["utf8", "truncated", "schema"]) {
    const f = fixture(); try {
      await anchored(f); f.collector.start(100);
      if (mode === "utf8") f.reader.invalidUtf8 = true;
      else if (mode === "truncated") f.reader.smaps = smaps.slice(0, -3);
      else f.reader.smaps = smaps.replace("Anonymous: 3 kB\n", "");
      edge(f, 0); await idle(f.collector); const summary = await f.collector.finish();
      assert.equal(summary.snapshotsValid, 0); assert.equal(summary.slots[0]?.state, "refused");
      assert.equal(readdirSync(f.outputRoot).includes("closed-transition-trigger.smaps.txt"), true);
    } finally { f.clean(); }
  }
});
test("logical per-read deadline and total read quota stop future I/O without retry", async () => {
  const f = fixture({ deadlineMs: 10 }); try {
    await anchored(f); f.collector.start(100); f.reader.onSmaps = () => { f.reader.nowNs += 11_000_000n; };
    edge(f, 0); await idle(f.collector); const summary = await f.collector.finish();
    assert.match(summary.slots[0]?.reason ?? "", /deadline/u); assert.equal(f.reader.mapsOpens, 1);
  } finally { f.clean(); }
  const quota = fixture({ readBytes: 15000 }); try {
    await anchored(quota); quota.collector.start(100); quota.reader.smaps = smaps + "Unknown: ".repeat(2000);
    edge(quota, 0); await idle(quota.collector); const summary = await quota.collector.finish();
    assert.equal(summary.bytesRead <= 15000, true); assert.equal(summary.incomplete, true);
  } finally { quota.clean(); }
});
test("VMA counts and output caps refuse without expanding limits; summary remains reserved", async () => {
  const f = fixture({ totalVmas: 1 }); try {
    const summary = await complete(f); assert.equal(summary.snapshotsValid, 1); assert.equal(summary.totalVmas, 1);
    assert.match(summary.slots[1]?.reason ?? "", /VMA cap/u);
  } finally { f.clean(); }
  const capped = fixture({ outputBytes: 20000 }); try {
    await anchored(capped); capped.collector.start(100); capped.reader.smaps = smaps.replace("UnknownNewKernelField: 7 kB", `Unknown: ${"x".repeat(6000)}`);
    edge(capped, 0); await idle(capped.collector); const summary = await capped.collector.finish();
    assert.match(summary.slots[0]?.reason ?? "", /output byte cap/u); assert.equal(summary.outputBytes <= 20000, true);
    assert.ok(readFileSync(join(capped.outputRoot, "summary.json")).length <= 16384);
  } finally { capped.clean(); }
});
test("per-snapshot VMA/maps caps and fixed lifecycle window stop new work without retries", async () => {
  const two = fixture({ vmas: 1 }); try {
    await anchored(two); two.collector.start(100); two.reader.maps = map + map.replace("1000-2000", "3000-4000");
    two.reader.smaps = smaps + smaps.replace("1000-2000", "3000-4000"); edge(two, 0); await idle(two.collector);
    const summary = await two.collector.finish(); assert.equal(summary.snapshotsValid, 0); assert.match(summary.slots[0]?.reason ?? "", /VMA.*cap/u);
  } finally { two.clean(); }
  const mapsCap = fixture({ mapsBytes: 10 }); try {
    await anchored(mapsCap); mapsCap.collector.start(100); edge(mapsCap, 0); await idle(mapsCap.collector);
    const summary = await mapsCap.collector.finish(); assert.match(summary.slots[0]?.reason ?? "", /byte cap/u);
    assert.equal(mapsCap.reader.smapsReads, 0); assert.equal(readFileSync(join(mapsCap.outputRoot, "closed-transition-trigger.maps-before.txt")).length, 10);
  } finally { mapsCap.clean(); }
  const deadline = fixture({ durationMs: 10 }); try {
    await anchored(deadline); deadline.collector.start(100); deadline.reader.nowNs += 11_000_000n; edge(deadline, 0); await idle(deadline.collector);
    const summary = await deadline.collector.finish(); assert.match(summary.slots[0]?.reason ?? "", /window deadline/u); assert.equal(deadline.reader.smapsReads, 0);
  } finally { deadline.clean(); }
});
test("missing/rejected/foreign slots are retained and repeated events never replace evidence", async () => {
  const f = fixture(); try {
    await anchored(f); f.collector.start(100); edge(f, 2, { outcome: "rejected" }); edge(f, 2); edge(f, 3); await idle(f.collector);
    const summary = await f.collector.finish(); assert.equal(summary.slots[0]?.state, "missing"); assert.equal(summary.slots[2]?.state, "refused");
    assert.equal(summary.allSlotsAccounted, false); assert.equal(summary.snapshotsValid, 1);
  } finally { f.clean(); }
  const foreign = fixture(); try {
    await anchored(foreign); foreign.collector.start(101); edge(foreign, 0); const summary = await foreign.collector.finish();
    assert.match(summary.refusals.join(" "), /foreign start PID/u); assert.equal(summary.snapshotsValid, 0);
  } finally { foreign.clean(); }
});
test("terminal metadata compaction forces incomplete, retains four slots/digest and respects UTF8 cap", async () => {
  const f = fixture(); try {
    await anchored(f); for (let i = 0; i < 32; i += 1) f.collector.refuse("雪".repeat(512));
    const summary = await f.collector.finish(); assert.equal(summary.compacted, true); assert.equal(summary.incomplete, true);
    assert.equal(summary.slots.length, 4); assert.match(summary.metadataSha256 ?? "", /^[\da-f]{64}$/u);
    assert.ok(readFileSync(join(f.outputRoot, "summary.json")).length <= 16384);
  } finally { f.clean(); }
});
test("metadata compaction itself prevents completion even when all four reads are otherwise valid", async () => {
  const f = fixture(); try {
    await anchored(f); f.collector.start(100); f.reader.onSmaps = () => f.collector.event({ phase: "雪".repeat(1200), atNs: f.reader.nowNs.toString() });
    for (let i = 0; i < 4; i += 1) { edge(f, i); await idle(f.collector); }
    assert.equal(f.collector.snapshotSummary().allReadsComplete, true); assert.equal(f.collector.snapshotSummary().incomplete, false);
    const summary = await f.collector.finish(); assert.equal(summary.compacted, true); assert.equal(summary.incomplete, true);
    assert.equal(summary.snapshotsValid, 4); assert.equal(summary.allReadsComplete, true); assert.equal(summary.slots.length, 4);
  } finally { f.clean(); }
});
test("exclusive output root refuses preexisting bytes and symlink parents; no overwrite", () => {
  const parent = realpathSync(mkdtempSync(join(tmpdir(), "vize-smaps-wx-"))), outputRoot = join(parent, "existing");
  try {
    writeFileSync(outputRoot, "unchanged"); const options = { rootPid: 100, image, outputRoot, sourceRevision: "b", workflowRevision: "h", reader: new FakeReader() };
    assert.throws(() => new SmapsSnapshots(options)); assert.equal(readFileSync(outputRoot, "utf8"), "unchanged");
    symlinkSync(parent, join(parent, "alias")); assert.throws(() => new SmapsSnapshots({ ...options, outputRoot: join(parent, "alias", "vma") }), /noncanonical/u);
    assert.throws(() => new SmapsSnapshots({ ...options, limits: { smapsBytes: SMAPS_LIMITS.smapsBytes + 1 } }), /lower-only/u);
  } finally { rmSync(parent, { recursive: true, force: true }); }
});
