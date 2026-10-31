import assert from "node:assert/strict";
import test from "node:test";
import { instrumentIncrementalMetricsPrototype, VBEN_LANES, VBEN_REQUIRED_PHASES,
  type VbenPhase, type VbenRoot, type VbenSink } from "../vben-observer.ts";

const server = "a".repeat(64), native = "b".repeat(64), argv = "c".repeat(64);
type FakeMember = { pid: number; parentPid: number; role: "root" | "descendant"; rssKiB: number;
  executable: string; identity: { birthTicks: string; executableKind: string; executableSha256: string;
    runtimeRole: string; argvCount: number; argvSha256: string } | null };
type FakeTree = { totalKiB: number; processes: number; members: FakeMember[] };
function member(pid: number, root = false): FakeMember {
  return { pid, parentPid: root ? 800 : 900, role: root ? "root" : "descendant", rssKiB: 10,
    executable: root ? "vize" : "other", identity: { birthTicks: String(pid), executableKind: root ? "vize" : "other",
      executableSha256: root ? server : native, runtimeRole: root ? "native-lsp" : "typescript-api", argvCount: 3, argvSha256: argv } };
}
function tree(lane: string): FakeTree {
  const members = [member(900, true)]; if (lane !== "initialize") members.push(member(901));
  if (VBEN_LANES.indexOf(lane as typeof VBEN_LANES[number]) >= 2) members.push(member(902));
  return { totalKiB: members.length * 10, processes: members.length, members };
}
function census(value: FakeTree): void { value.processes = value.members.length; value.totalKiB = value.members.reduce((sum, item) => sum + item.rssKiB, 0); }
function identity(value: FakeMember | undefined): NonNullable<FakeMember["identity"]> { assert.ok(value?.identity); return value.identity; }
function frozen(value: FakeTree): FakeTree {
  for (const item of value.members) { if (item.identity !== null) Object.freeze(item.identity); Object.freeze(item); }
  Object.freeze(value.members); return Object.freeze(value);
}
class Metrics {
  suite = { id: "vben-lsp-incremental" }; processId = 900; expectedRootBirthTicks = "900";
  processTreeSamples: Record<string, FakeTree> = {}; calls: unknown[][] = []; returned: unknown;
  mutate: (lane: string, sample: FakeTree) => void = () => {};
  original: ((...args: unknown[]) => unknown) | undefined;
  measure(...args: unknown[]): unknown {
    assert.equal(this instanceof Metrics, true); this.calls.push(args);
    if (this.original !== undefined) return this.original(...args);
    const lane = String(args[0]); const sample = tree(lane); this.mutate(lane, sample);
    this.processTreeSamples[lane] = frozen(sample); this.returned = Promise.resolve(args[2]); return this.returned;
  }
}
function install(extra: Partial<VbenSink> = {}): {
  uninstall: () => void; events: VbenPhase[]; refusals: string[]; stops: string[]; roots: VbenRoot[];
} {
  const events: VbenPhase[] = [], refusals: string[] = [], stops: string[] = [], roots: VbenRoot[] = [];
  let clock = 1n;
  const uninstall = instrumentIncrementalMetricsPrototype(Metrics.prototype, { serverImageSha: server, nativeImageSha: native,
    now: () => clock++, sinkFactory(root) { roots.push(root); return {
      phase(value) { events.push(value); extra.phase?.(value); },
      refuse(reason) { refusals.push(reason); extra.refuse?.(reason); },
      stop(reason) { stops.push(reason); extra.stop?.(reason); },
    }; } });
  return { uninstall, events, refusals, stops, roots };
}
async function lanes(metrics: Metrics, end: number = VBEN_LANES.length): Promise<void> {
  for (const lane of VBEN_LANES.slice(0, end)) {
    const operation = () => Promise.resolve("untouched"); const result = { lane };
    const promise = metrics.measure(lane, operation, result); assert.equal(promise, metrics.returned);
    assert.equal(await promise, result); assert.deepEqual(metrics.calls.at(-1), [lane, operation, result]);
  }
}

test("all fourteen lanes preserve receiver, argument identity, exact Promise/result and retained snapshots", async () => {
  const original = Object.getOwnPropertyDescriptor(Metrics.prototype, "measure"); const capture = install();
  try {
    const metrics = new Metrics(); await lanes(metrics);
    assert.equal(metrics.calls.length, 14); assert.equal(capture.roots.length, 1);
    assert.deepEqual(capture.events.map((event) => event.phase), VBEN_REQUIRED_PHASES);
    assert.deepEqual(capture.refusals, []); assert.deepEqual(capture.stops, ["vben-all-lanes-settled"]);
    for (const event of capture.events) {
      assert.equal(event.rootPid, 900); assert.equal(event.rootBirthTicks, "900");
      assert.equal(event.serverImageSha, server); assert.equal(event.nativeImageSha, native);
      assert.equal(event.sourceSnapshot, event.stage === "before" ? null : metrics.processTreeSamples[event.lane]);
      assert.equal(event.outcome, event.stage === "before" ? "pending" : "fulfilled");
    }
    const selected = capture.events.filter((event) => event.stage === "settled" && event.target !== null);
    assert.equal(selected.length, 12);
    for (const event of selected) assert.deepEqual(event.target, { pid: 902, birthTicks: "902", parentPid: 900, nativeImageSha: native });
    const first = capture.events.find((event) => event.phase === "vben-coldOpenSecondApp-settled");
    assert.equal(first?.target?.pid, 902); assert.equal(Object.isFrozen(first?.target), true);
    assert.equal(Object.isFrozen(metrics.processTreeSamples["coldOpenSecondApp"]), true);
  } finally { capture.uninstall(); }
  assert.deepEqual(Object.getOwnPropertyDescriptor(Metrics.prototype, "measure"), original);
});

test("Misskey and unrelated suites are forwarded without factory calls or Vben writes", async () => {
  const capture = install();
  try {
    for (const id of ["misskey-lsp-incremental", "misskey-lsp-churn", "other"]) {
      const metrics = new Metrics(); metrics.suite.id = id; metrics.processId = -1;
      const result = metrics.measure("coldOpen", () => Promise.resolve(), id);
      assert.equal(result, metrics.returned); assert.equal(await result, id);
    }
    assert.deepEqual(capture.roots, []); assert.deepEqual(capture.events, []); assert.deepEqual(capture.refusals, []);
  } finally { capture.uninstall(); }
});

test("synchronous throw and Promise rejection retain the identical original error", async () => {
  for (const mode of ["sync", "async"]) {
    const capture = install(); const metrics = new Metrics(); const error = new Error("original failure");
    metrics.original = () => { if (mode === "sync") throw error; metrics.returned = Promise.reject(error); return metrics.returned; };
    try {
      if (mode === "sync") assert.throws(() => metrics.measure("initialize"), (value: unknown) => value === error);
      else { const result = metrics.measure("initialize"); assert.equal(result, metrics.returned); await assert.rejects(result as Promise<unknown>, (value: unknown) => value === error); }
      assert.equal(metrics.calls.length, 1); assert.ok(capture.refusals.length > 0);
      assert.equal(capture.events.at(-1)?.outcome, mode === "sync" ? "threw" : "rejected");
    } finally { capture.uninstall(); }
  }
});

test("observer callback failures do not replace original successful Promise or result", async () => {
  const capture = install({ phase() { throw new Error("sink unavailable"); }, refuse() { throw new Error("refusal sink unavailable"); }, stop() { throw new Error("stop unavailable"); } });
  try { const metrics = new Metrics(); const promise = metrics.measure("initialize", null, "value"); assert.equal(promise, metrics.returned); assert.equal(await promise, "value"); assert.equal(metrics.calls.length, 1); assert.ok(capture.refusals.length > 0); }
  finally { capture.uninstall(); }
});

test("non-Promise original return and invalid installation remain explicit controls", () => {
  const capture = install();
  try { const metrics = new Metrics(); const value = {}; metrics.original = () => value; assert.equal(metrics.measure("initialize"), value); assert.ok(capture.refusals.includes("measure returned non-Promise")); }
  finally { capture.uninstall(); }
  assert.throws(() => instrumentIncrementalMetricsPrototype({}, { serverImageSha: server, nativeImageSha: native, sinkFactory() { throw new Error("unused"); } }));
  assert.throws(() => instrumentIncrementalMetricsPrototype(Metrics.prototype, { serverImageSha: "bad", nativeImageSha: native, sinkFactory() { throw new Error("unused"); } }));
});

test("empty/multiple/foreign/reused/malformed cohorts refuse selection without changing lane outcomes", async () => {
  const changes: Array<[string, (value: FakeTree) => void]> = [
    ["empty", (value) => { value.members.pop(); }],
    ["multiple", (value) => { value.members.push(member(903)); }],
    ["wrong root", (value) => { value.members[0]!.pid = 999; }],
    ["root birth", (value) => { identity(value.members[0]).birthTicks = "999"; }],
    ["root image", (value) => { identity(value.members[0]).executableSha256 = native; }],
    ["root role", (value) => { identity(value.members[0]).runtimeRole = "native-api"; }],
    ["native hash", (value) => { identity(value.members[2]).executableSha256 = server; }],
    ["native indirect parent", (value) => { value.members[2]!.parentPid = 901; }],
    ["foreign parent", (value) => { value.members[2]!.parentPid = 777; }],
    ["wrong native role", (value) => { identity(value.members[2]).runtimeRole = "other"; }],
    ["native kind", (value) => { identity(value.members[2]).executableKind = "node"; value.members[2]!.executable = "node"; }],
    ["baseline PID reused", (value) => { identity(value.members[1]).birthTicks = "999"; }],
    ["duplicate PID", (value) => { value.members[2]!.pid = 901; }],
    ["missing Linux identity", (value) => { value.members[2]!.identity = null; }],
    ["invalid birth", (value) => { identity(value.members[2]).birthTicks = "-1"; }],
  ];
  for (const [label, change] of changes) {
    const capture = install(); const metrics = new Metrics();
    metrics.mutate = (lane, value) => { if (lane === "coldOpenSecondApp") { change(value); census(value); } };
    try { await lanes(metrics, 3); assert.ok(capture.refusals.length > 0, label); assert.equal(capture.events.at(-1)?.target, null, label); assert.equal(metrics.calls.length, 3); }
    finally { capture.uninstall(); }
  }
});

test("selected identity cannot vanish, change birth/hash/role/argv or gain an ambiguous new peer", async () => {
  const changes: Array<(value: FakeTree) => void> = [
    (value) => { value.members.pop(); }, (value) => { identity(value.members[2]).birthTicks = "1000"; },
    (value) => { identity(value.members[2]).executableSha256 = server; }, (value) => { identity(value.members[2]).runtimeRole = "typescript-lsp"; },
    (value) => { identity(value.members[2]).argvSha256 = server; }, (value) => { value.members[2]!.pid = 903; },
    (value) => { value.members.push(member(903)); },
  ];
  for (const change of changes) {
    const capture = install(); const metrics = new Metrics(); metrics.mutate = (lane, value) => { if (lane === "completion") { change(value); census(value); } };
    try { await lanes(metrics, 4); assert.equal(capture.events.find((event) => event.phase === "vben-coldOpenSecondApp-settled")?.target?.pid, 902); assert.ok(capture.refusals.length > 0); assert.equal(capture.events.at(-1)?.target, null); }
    finally { capture.uninstall(); }
  }
});

test("wrong lane order, duplicate lane and uninstall before completion refuse diagnostic completeness", async () => {
  for (const lane of ["coldOpen", "unknown"]) {
    const capture = install(); try { const metrics = new Metrics(); await metrics.measure(lane); assert.equal(metrics.calls.length, 1); assert.ok(capture.refusals.length > 0); }
    finally { capture.uninstall(); }
  }
  const capture = install(); const metrics = new Metrics(); await metrics.measure("initialize"); await metrics.measure("initialize"); capture.uninstall(); capture.uninstall();
  assert.ok(capture.refusals.length > 0); assert.deepEqual(capture.stops, ["vben-uninstall"]);
});

test("concurrent measures and pending uninstall preserve original deferred Promise identities", async () => {
  let resolve: ((value: unknown) => void) | undefined; const pending = new Promise<unknown>((done) => { resolve = done; });
  const capture = install(); const metrics = new Metrics(); metrics.original = () => pending;
  const first = metrics.measure("initialize"), second = metrics.measure("coldOpen");
  assert.equal(first, pending); assert.equal(second, pending); capture.uninstall(); assert.ok(capture.refusals.length > 0);
  const value = {}; assert.ok(resolve); resolve(value); assert.equal(await first, value); assert.equal(await second, value); assert.equal(metrics.calls.length, 2);
});
