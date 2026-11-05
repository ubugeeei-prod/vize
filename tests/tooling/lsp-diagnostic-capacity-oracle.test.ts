import assert from "node:assert/strict";
import { test } from "node:test";
import { decodeNativeSockets, nativeIpcGrew } from "./support/lsp/diagnostic-capacity-pipe.ts";
import { CapacityNativeCoverage } from "./support/lsp/diagnostic-capacity-native.ts";
import { CapacityCoverage, capacitySource } from "./support/lsp/diagnostic-capacity.ts";

function scheduledNativeCoverage(count: number) {
  const probes = Array.from({ length: count }, (_, index) =>
    capacitySource(index, `file:///workspace/Coverage${index}.vue`),
  );
  const coverage = new CapacityCoverage(probes);
  let trace = "";
  let now = 0;
  const native = new CapacityNativeCoverage(coverage, () => trace);
  const publish = (index: number, version = 1) => {
    const probe = probes[index]!;
    coverage.record(
      {
        uri: probe.uri,
        version,
        diagnostics: [
          {
            source: "vize/types",
            code: 2339,
            message: probe.token,
            range: {
              start: { line: probe.line, character: probe.character },
              end: { line: probe.line, character: probe.character + probe.token.length },
            },
          },
        ],
      },
      now,
    );
    for (const stage of [
      "starting initial type diagnostics for",
      "sent collected diagnostics for",
      "finished initial type diagnostics for",
    ])
      trace += `${stage} ${probe.uri} version ${version}\n`;
  };
  return {
    coverage,
    native,
    probes,
    publish,
    clock(onSleep: (now: number) => void) {
      return {
        now: () => now,
        async sleep(milliseconds: number) {
          now += milliseconds;
          onSleep(now);
        },
      };
    },
  };
}

test("a native cohort can finish beyond one idle interval while new owned URI completions advance", async () => {
  const run = scheduledNativeCoverage(3);
  const published = new Set<number>();
  const clock = run.clock((now) => {
    for (let index = 0; index < 3; index++)
      if (now >= (index + 1) * 50 && !published.has(index)) {
        run.publish(index);
        published.add(index);
      }
  });
  assert.equal(await run.native.waitForComplete(1, false, 60, clock), true);
  assert.equal(
    clock.now(),
    150,
    "the cohort retains all three independent completed native passes",
  );
  run.native.assertComplete(1, false);
});

test("replayed sends and another version cannot extend a stalled native cohort", async () => {
  const run = scheduledNativeCoverage(2);
  run.publish(0);
  const clock = run.clock(() => {
    run.publish(0);
    run.publish(1, 2);
  });
  assert.equal(await run.native.waitForComplete(1, false, 60, clock), false);
  assert.ok(
    clock.now() <= 110,
    "repeated activity cannot refresh the unique-URI progress deadline",
  );
  assert.throws(() => run.native.assertComplete(1, false));
});

test("new cohort progress cannot turn an overwritten historical packet into current completion", async () => {
  const run = scheduledNativeCoverage(2);
  run.publish(0);
  let second = false;
  const clock = run.clock((now) => {
    run.coverage.record({ uri: run.probes[0]!.uri, version: 1, diagnostics: [] }, now);
    if (!second) {
      run.publish(1);
      second = true;
    }
  });
  assert.equal(await run.native.waitForComplete(1, false, 60, clock), false);
  assert.ok(clock.now() <= 160);
  assert.equal(run.native.rows(1, false)[0]!.historicalMatchingPackets, 1);
  assert.throws(() => run.native.assertComplete(1, false));
});

test("the outer test deadline cancels a progressing native drain", async () => {
  const run = scheduledNativeCoverage(2);
  const controller = new AbortController();
  const clock = run.clock(() => {
    run.publish(0);
    controller.abort();
  });
  assert.equal(await run.native.waitForComplete(1, false, 60, clock, controller.signal), false);
  assert.equal(clock.now(), 50, "the 300-second test bound must not become another idle interval");
});

test("bulk diagnostic coverage preserves multiplicity and requires exact native tokens and UTF16 ranges", () => {
  const probe = capacitySource(2, "file:///workspace/Coverage0002.vue");
  const coverage = new CapacityCoverage([probe]);
  const good = {
    uri: probe.uri,
    version: 1,
    diagnostics: [
      {
        source: "vize/types",
        code: 2339,
        message: "Property '" + probe.token + "' does not exist",
        range: {
          start: { line: probe.line, character: probe.character },
          end: { line: probe.line, character: probe.character + probe.token.length },
        },
      },
    ],
  };
  const diagnostic = good.diagnostics[0];
  assert.ok(diagnostic);
  for (const packet of [
    { ...good, version: 0 },
    { ...good, diagnostics: [] },
    { ...good, diagnostics: [{ ...diagnostic, source: "vize/lint" }] },
    { ...good, diagnostics: [{ ...diagnostic, message: "wrong token" }] },
    {
      ...good,
      diagnostics: [
        {
          ...diagnostic,
          range: {
            ...diagnostic.range,
            start: { line: probe.line, character: probe.character + 1 },
          },
        },
      ],
    },
    { ...good, diagnostics: [diagnostic, diagnostic] },
  ])
    coverage.record(packet, 1);
  assert.throws(() => coverage.assertComplete(1, false));
  coverage.record(good, 2);
  coverage.record(good, 3);
  coverage.assertComplete(1, false);
  const row = coverage.rows(1, false)[0];
  assert.ok(row);
  assert.equal(row.historicalMatchingPackets, 2);
  assert.equal(coverage.packets.length, 8);
  coverage.record({ ...good, diagnostics: [] }, 4);
  assert.throws(() => coverage.assertComplete(1, false), "later empty must replace the old error");
  coverage.record(good, 5);
  coverage.assertComplete(1, false);
  assert.ok(probe.invalid.includes("\r\n"));
  assert.equal(probe.character, "/*🌱日本語*/ value0002.".length);
});

test("repair needs a current complete empty packet for every URI", () => {
  const first = capacitySource(0, "file:///workspace/A.vue");
  const second = capacitySource(1, "file:///workspace/B.vue");
  const coverage = new CapacityCoverage([first, second]);
  coverage.record({ uri: first.uri, version: 3, diagnostics: [] }, 1);
  coverage.record({ uri: second.uri, version: 2, diagnostics: [] }, 1);
  coverage.record({ uri: second.uri, version: 3, diagnostics: [{ code: "typecheck-pending" }] }, 1);
  assert.throws(() => coverage.assertComplete(3, true));
  coverage.record({ uri: second.uri, version: 3, diagnostics: [] }, 2);
  coverage.assertComplete(3, true);
  coverage.record({ uri: second.uri, version: 3, diagnostics: [{ code: 2339 }] }, 3);
  assert.throws(() => coverage.assertComplete(3, true), "later error must replace repaired empty");
});

test("lint-only empties, started passes and foreign URI/version finishes never prove native repair", () => {
  const probe = capacitySource(1, "file:///workspace/A.vue");
  const coverage = new CapacityCoverage([probe]);
  let trace = "";
  const native = new CapacityNativeCoverage(coverage, () => trace);
  const complete = () => native.rows(3, true).every((row) => row.complete);
  coverage.record({ uri: probe.uri, version: 3, diagnostics: [] }, 1);
  assert.equal(complete(), false, "Patina empty is insufficient");
  trace += "starting initial type diagnostics for " + probe.uri + " version 3\n";
  assert.equal(complete(), false, "started is insufficient");
  for (const [uri, version] of [
    [probe.uri, 2],
    ["file:///workspace/B.vue", 3],
  ]) {
    trace += "starting initial type diagnostics for " + uri + " version " + version + "\n";
    trace += "sent collected diagnostics for " + uri + " version " + version + "\n";
    trace += "finished initial type diagnostics for " + uri + " version " + version + "\n";
  }
  assert.equal(complete(), false, "another URI or version cannot satisfy the target");
  trace += "finished initial type diagnostics for " + probe.uri + " version 3\n";
  assert.equal(complete(), false, "finished without an owned send is insufficient");
  assert.throws(() => native.assertComplete(3, true));
  trace += "sent collected diagnostics for " + probe.uri + " version 3";
  assert.equal(complete(), false, "a partial trace line is not yet a source receipt");
  trace += "\n";
  assert.equal(complete(), false, "a send after finish cannot qualify the earlier pass");
  trace += "starting initial type diagnostics for " + probe.uri + " version 3\n";
  trace += "sent collected diagnostics for " + probe.uri + " version 3\n";
  assert.equal(complete(), false, "a send without its matching finish is not complete");
  trace += "finished initial type diagnostics for " + probe.uri + " version 3\n";
  native.assertComplete(3, true);
  assert.equal(native.idle(), true);
  coverage.record({ uri: probe.uri, version: 3, diagnostics: [{ code: 2339 }] }, 2);
  assert.equal(complete(), false, "a later wrong packet invalidates past native success");
});

test("native socket admission joins stopped owner FD and inode and rejects listener or foreign queues", () => {
  const descriptor = { pid: 10, fd: 7, inode: "123", birthTicks: "30", executable: "/native/tsc" };
  const raw = (queue: number) =>
    "u_str ESTAB " +
    queue +
    ' 0 /tmp/tsgo-api-test 123 * 456 users:(("tsc",pid=10,fd=7)) uid:1001 ino:9702886 dev:0/2049 sk:42';
  const before = {
    source: "Linux AF_UNIX SOCK_DIAG" as const,
    processes: [],
    raw: [raw(0)],
    descriptors: [descriptor],
    channels: decodeNativeSockets([raw(0)], [descriptor]),
  };
  const after = { ...before, channels: decodeNativeSockets([raw(48)], [descriptor]) };
  assert.equal(nativeIpcGrew(before, after), true);
  assert.equal(after.channels[0]?.inode, "123");
  assert.equal(after.channels[0]?.filesystemInode, "9702886");
  for (const wrong of [
    raw(48).replace("pid=10", "pid=11"),
    raw(48).replace("fd=7", "fd=8"),
    raw(48).replace(" 123 * 456", " 124 * 456"),
    raw(48).replace("ESTAB", "LISTEN"),
    raw(48).replace("/tmp/tsgo-api-test", "/tmp/other-service"),
  ]) {
    assert.deepEqual(decodeNativeSockets([wrong], [descriptor]), []);
  }
  assert.equal(
    nativeIpcGrew(before, {
      ...after,
      channels: after.channels.map((row) => ({ ...row, birthTicks: "31" })),
    }),
    false,
  );
  assert.equal(nativeIpcGrew(before, before), false);
});
