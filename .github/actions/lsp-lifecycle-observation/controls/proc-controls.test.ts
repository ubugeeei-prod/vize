import assert from "node:assert/strict";
import { setTimeout as sleep } from "node:timers/promises";
import test from "node:test";

import { parseKiBFields, parseProcStat, ProcSampler, sameAnchor, type SamplerOptions } from "../proc-sampler.ts";
import { OBSERVATION_LIMITS } from "../types.ts";
import { close, eventually, native, ProcFixture, procStat, samples, server, setup } from "./proc-fixture.ts";

function gate() {
  let release!: () => void;
  const promise = new Promise<void>((resolve) => { release = resolve; });
  return { promise, release };
}

test("proc stat preserves large birth ticks and names with spaces/parentheses; malformed identities refuse", () => {
  const value = parseProcStat(procStat(900, 42, "9007199254740993", "worker ) name"));
  assert.equal(value.birthTicks, "9007199254740993");
  assert.equal(value.command, "worker ) name");
  assert.equal(value.userTicks, "11");
  assert.equal(value.systemTicks, "7");
  assert.equal(value.processor, 3);
  assert.ok(sameAnchor(value, { ...value, userTicks: "99" }));
  assert.equal(sameAnchor(value, { ...value, birthTicks: "9007199254740994" }), false);
  assert.throws(() => parseProcStat("broken"));
  assert.throws(() => parseProcStat(procStat(0, 42, "1")));
  assert.throws(() => parseProcStat(procStat(900, 42, "not-a-tick")));
});

test("memory parser retains KiB fields independently and refuses duplicates or unsafe integers", () => {
  assert.deepEqual(parseKiBFields("VmRSS: 100 kB\nRssAnon: 60 kB\nRssFile: 35 kB\nThreads: 3\n", ["VmRSS", "RssAnon", "RssFile"]),
    { VmRSS: 100, RssAnon: 60, RssFile: 35 });
  assert.deepEqual(parseKiBFields("VmRSS: -1 kB\nRssFile: 10 MB\n", ["VmRSS", "RssFile"]), {});
  assert.throws(() => parseKiBFields("VmRSS: 1 kB\nVmRSS: 2 kB\n", ["VmRSS"]));
  assert.throws(() => parseKiBFields("VmRSS: 9007199254740993 kB\n", ["VmRSS"]));
});

test("same-name navigation threads retain separate TID births and never claim per-thread RSS", async () => {
  const { sampler, records } = await setup();
  try {
    sampler.start(900);
    await eventually(() => samples(records).length === 1 && !sampler.snapshotSummary().inFlight);
    const packet = samples(records)[0]!;
    const root = packet.processes.find((value) => value.pid === 900)!;
    const workers = root.threads.filter((value) => value.command === "vize-project-navigation");
    assert.deepEqual(workers.map(({ tid, birthTicks }) => [tid, birthTicks]), [[901, "401"], [902, "402"]]);
    assert.equal(root.statusKiB["VmRSS"], 100);
    assert.equal(root.statusKiB["RssAnon"], 60);
    assert.equal(root.statusKiB["RssFile"], 35);
    assert.ok(root.threads.every((value) => !Object.keys(value).some((key) => /rss/i.test(key))));
    assert.equal(packet.processes.length, 2);
    assert.match(packet.scheduledNs, /^\d+$/u);
    assert.ok(Number.isFinite(packet.startLagMs) && packet.startLagMs >= 0);
    assert.equal(sampler.snapshotSummary().qualificationCredit, 0);
  } finally { await close(sampler); }
});

test("foreign ancestry, executable path and changed backing-image metadata refuse attribution", async () => {
  for (const mutate of [
    (fixture: ProcFixture) => { fixture.processes.get(910)!.parent = 42; },
    (fixture: ProcFixture) => { fixture.processes.get(910)!.image = { ...native, realpath: "/foreign/tsc" }; },
    (fixture: ProcFixture) => { fixture.processes.get(910)!.image = { ...native, ctimeNs: "999" }; },
  ]) {
    const fixture = new ProcFixture();
    mutate(fixture);
    const { sampler, records } = await setup(fixture);
    try {
      sampler.start(900);
      await eventually(() => sampler.snapshotSummary().invalid && !sampler.snapshotSummary().inFlight);
      assert.equal(samples(records).length, 0);
      assert.ok(sampler.snapshotSummary().refusals.length > 0);
    } finally { await close(sampler); }
  }
});

test("new native owner PID is a new admitted generation; reused PID and TID births are refused", async () => {
  const { fixture, sampler, records } = await setup();
  try {
    sampler.start(900);
    await eventually(() => samples(records).length === 1 && !sampler.snapshotSummary().inFlight);
    fixture.processes.delete(910);
    fixture.addNative(920, "520");
    fixture.processes.get(900)!.tasks.get(900)!.children = [920];
    sampler.requestSample("synthetic-new-owner");
    await eventually(() => samples(records).length === 2 && !sampler.snapshotSummary().inFlight);
    assert.deepEqual(samples(records)[1]!.processes.map(({ pid }) => pid), [900, 920]);
    fixture.processes.get(920)!.birth = "521";
    fixture.processes.get(920)!.tasks.get(920)!.birth = "521";
    sampler.requestSample("synthetic-reused-owner");
    await eventually(() => sampler.snapshotSummary().invalid && !sampler.snapshotSummary().inFlight);
    assert.equal(samples(records).length, 2);
  } finally { await close(sampler); }
  const next = await setup();
  try {
    next.sampler.start(900);
    await eventually(() => samples(next.records).length === 1 && !next.sampler.snapshotSummary().inFlight);
    next.fixture.processes.get(900)!.tasks.get(901)!.birth = "499";
    next.sampler.requestSample("synthetic-reused-tid");
    await eventually(() => next.sampler.snapshotSummary().invalid && !next.sampler.snapshotSummary().inFlight);
    assert.equal(samples(next.records).length, 1);
  } finally { await close(next.sampler); }
});

test("root PID replacement is refused against the constructor-time anchor", async () => {
  const { fixture, sampler, records } = await setup();
  try {
    fixture.processes.get(900)!.birth = "9007199254740994";
    fixture.processes.get(900)!.tasks.get(900)!.birth = "9007199254740994";
    sampler.start(900);
    await eventually(() => sampler.snapshotSummary().invalid && !sampler.snapshotSummary().inFlight);
    assert.equal(samples(records).length, 0);
  } finally { await close(sampler); }
});

test("late child creation is a reported cohort gap, then a fresh complete cohort can be sampled", async () => {
  const fixture = new ProcFixture();
  fixture.hook = (file, count) => {
    if (file === "/proc/900/task/900/children" && count === 2) {
      fixture.addNative(920, "520");
      fixture.processes.get(900)!.tasks.get(900)!.children = [910, 920];
    }
  };
  const { sampler, records } = await setup(fixture);
  try {
    sampler.start(900);
    await eventually(() => sampler.snapshotSummary().attempts === 1 && !sampler.snapshotSummary().inFlight);
    assert.equal(samples(records).length, 0);
    assert.ok(records.some((record) => record.kind === "notice" && record.reason.includes("cohort")));
    assert.equal(sampler.snapshotSummary().invalid, false);
    sampler.requestSample("synthetic-fresh-cohort");
    await eventually(() => samples(records).length === 1 && !sampler.snapshotSummary().inFlight);
    assert.deepEqual(samples(records)[0]!.processes.map(({ pid }) => pid), [900, 910, 920]);
  } finally { await close(sampler); }
});

test("final custody check rejects an executable swap or unchanged-number TID reuse late in the capture", async () => {
  for (const change of ["image", "tid"] as const) {
    const fixture = new ProcFixture();
    fixture.hook = (file, count) => {
      if (file === "/proc/900/stat" && count === 5) {
        if (change === "image") fixture.processes.get(900)!.image = { ...server, ctimeNs: "999" };
        else fixture.processes.get(900)!.tasks.get(901)!.birth = "499";
      }
    };
    const { sampler, records } = await setup(fixture);
    try {
      sampler.start(900);
      await eventually(() => sampler.snapshotSummary().invalid && !sampler.snapshotSummary().inFlight);
      assert.equal(samples(records).length, 0);
    } finally { await close(sampler); }
  }
});

test("phase A to B to A inside one sample is overlap even though endpoint labels agree", async () => {
  const fixture = new ProcFixture();
  let now = 0n;
  const { sampler, records } = await setup(fixture, { now: () => now });
  fixture.hook = (file, count) => {
    if (file === "/proc/900/status" && count === 1) {
      now = 1n; sampler.event({ phase: "B", atNs: "1" });
      now = 2n; sampler.event({ phase: "A", atNs: "2" });
    }
  };
  try {
    sampler.event({ phase: "A", atNs: "0" });
    sampler.start(900);
    await eventually(() => samples(records).length === 1 && !sampler.snapshotSummary().inFlight);
    const packet = samples(records)[0]!;
    assert.equal(packet.startPhase, "A");
    assert.equal(packet.endPhase, "A");
    assert.equal(packet.phaseRelation, "overlap");
    assert.notEqual(packet.startPhaseEpoch, packet.endPhaseEpoch);
  } finally { await close(sampler); }
});

test("generic empty/wrong diagnostic publications and unmatched responses do not advance the causal phase", async () => {
  const fixture = new ProcFixture();
  let now = 0n;
  const { sampler, records } = await setup(fixture, { now: () => now });
  fixture.hook = (file, count) => {
    if (file === "/proc/900/status" && count === 1) {
      now = 1n; sampler.event({ phase: "ephemeral-version1-publication", atNs: "1", detail: { diagnostics: 0 } });
      now = 2n; sampler.event({ phase: "uncorrelated-response", atNs: "2" });
    }
  };
  try {
    sampler.event({ phase: "ephemeral-did-open-before-write", atNs: "0" });
    sampler.start(900);
    await eventually(() => samples(records).length === 1 && !sampler.snapshotSummary().inFlight);
    const packet = samples(records)[0]!;
    assert.equal(packet.phaseRelation, "contained");
    assert.equal(packet.startPhaseEpoch, packet.endPhaseEpoch);
    assert.equal(sampler.snapshotSummary().phaseEvents.length, 1);
  } finally { await close(sampler); }
});

test("normal stop drains a crossing sample and later stop cannot overwrite the first window end", async () => {
  const fixture = new ProcFixture();
  const wait = gate(); let entered = false; let now = 0n;
  fixture.hook = async (file) => {
    if (file === "/proc/900/status") { entered = true; await wait.promise; }
  };
  const { sampler, records } = await setup(fixture, { now: () => now });
  try {
    sampler.event({ phase: "final-document-symbol-settled", atNs: "0" });
    sampler.start(900);
    await eventually(() => entered);
    now = 1n; sampler.stop("first-cycle-methods-complete");
    now = 2n; wait.release();
    await eventually(() => !sampler.snapshotSummary().inFlight);
    assert.equal(samples(records).length, 1, "normal stop must not abort the in-flight observation");
    assert.equal(samples(records)[0]!.phaseRelation, "overlap");
    assert.equal(sampler.snapshotSummary().endNs, "1");
    now = 50_000_000_000n;
    sampler.stop("synthetic-beforeExit-fifty-seconds-later");
    assert.equal(sampler.snapshotSummary().endNs, "1", "shutdown cannot extend the 20-second observation window");
  } finally { wait.release(); await close(sampler); }
});

test("forced boundaries do not overlap reads and all captures share one hard sample budget", async () => {
  const fixture = new ProcFixture(); const wait = gate(); let entered = false;
  fixture.hook = async (file) => {
    if (file === "/proc/900/status") { entered = true; await wait.promise; }
  };
  const { sampler, records } = await setup(fixture, { limits: { samples: 1 } });
  try {
    sampler.start(900);
    await eventually(() => entered);
    for (let index = 0; index < 20; index += 1) sampler.requestSample("synthetic-forced-boundary");
    assert.equal(sampler.snapshotSummary().attempts, 1);
    assert.ok(sampler.snapshotSummary().skippedTicks >= 20);
    wait.release();
    await eventually(() => samples(records).length === 1 && !sampler.snapshotSummary().inFlight);
    sampler.requestSample("synthetic-over-budget");
    assert.equal(sampler.snapshotSummary().attempts, 1);
    assert.equal(sampler.snapshotSummary().invalid, true);
  } finally { wait.release(); await close(sampler); }
});

test("wall deadline and overlarge PID/TID/file cohorts refuse without admitting a packet", async () => {
  for (const kind of ["wall", "pids", "threads", "file"] as const) {
    const fixture = new ProcFixture(); let now = 0n;
    const { sampler, records } = await setup(fixture, {
      now: () => now,
      ...(kind === "pids" ? { limits: { pids: 1 } } : {}),
      ...(kind === "threads" ? { limits: { threads: 2 } } : {}),
    });
    try {
      if (kind === "file") fixture.status = "x".repeat(OBSERVATION_LIMITS.fileBytes + 1);
      sampler.start(900);
      if (kind === "wall") now = BigInt(OBSERVATION_LIMITS.durationMs + 1) * 1_000_000n;
      await eventually(() => sampler.snapshotSummary().invalid && !sampler.snapshotSummary().inFlight);
      assert.equal(samples(records).length, 0);
    } finally { await close(sampler); }
  }
});

test("a deadline abort is an explicit gap/refusal and leaves no overlapping sample active", async () => {
  const fixture = new ProcFixture();
  fixture.hook = async (file, _count, signal) => {
    if (file !== "/proc/900/status") return;
    await new Promise<void>((_resolve, reject) => {
      if (signal.aborted) reject(signal.reason);
      else signal.addEventListener("abort", () => reject(signal.reason), { once: true });
    });
  };
  const { sampler, records } = await setup(fixture, { limits: { durationMs: 50, sampleDeadlineMs: 100 } });
  try {
    sampler.start(900);
    await sleep(80);
    await eventually(() => sampler.snapshotSummary().invalid && !sampler.snapshotSummary().inFlight);
    assert.equal(samples(records).length, 0);
    assert.ok(records.some((record) => record.kind === "notice"));
    const failure = records.find((record) => record.kind === "notice" && record.attempt !== undefined);
    assert.ok(failure?.kind === "notice" && failure.attempt !== undefined);
    assert.match(failure.attempt.scheduledNs, /^\d+$/u);
    assert.ok(failure.attempt.startLagMs >= 0 && failure.attempt.wallMs >= 0);
    assert.ok(failure.attempt.bytesRead >= 0 && failure.attempt.reads >= 0);
    assert.ok(BigInt(failure.attempt.endNs) >= BigInt(failure.attempt.startNs));
    assert.equal("processes" in failure.attempt, false, "failed custody never emits untrusted process rows");
    assert.ok(Number.isSafeInteger(sampler.snapshotSummary().intervalCallbacks));
  } finally { await close(sampler); }
});

test("test injection cannot raise hard production bounds or sample faster than 50 ms", () => {
  const keys = ["durationMs", "samples", "pids", "threads", "sampleDeadlineMs", "readBytes"] as const;
  for (const key of keys) {
    const limits: NonNullable<SamplerOptions["limits"]> = { [key]: OBSERVATION_LIMITS[key] + 1 };
    assert.throws(() => new ProcSampler({ rootPid: 900, images: [server, native], reader: new ProcFixture(), emit: () => {}, limits }));
  }
  for (const limits of [{ intervalMs: 49 }, { samples: 0 }, { durationMs: -1 }]) {
    assert.throws(() => new ProcSampler({ rootPid: 900, images: [server, native], reader: new ProcFixture(), emit: () => {}, limits }));
  }
});
