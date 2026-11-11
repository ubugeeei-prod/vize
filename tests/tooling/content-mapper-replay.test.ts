import assert from "node:assert/strict";
import test from "node:test";

import { frame } from "../../tools/benchmarks/scripts/content-mapper-pair-report.ts";
import {
  type ReplayRequest,
  type ReplayRun,
  type ReplayStream,
  bodyFrames,
  markdown,
  replyDigest,
  summarizeReplay,
} from "../../tools/benchmarks/scripts/content-mapper-replay-report.ts";
import {
  concurrency,
  hostStats,
} from "../../tools/benchmarks/scripts/content-mapper-replay-hosts.ts";
import { parseMapper, replyKey } from "../../tools/benchmarks/scripts/content-mapper-replay.ts";

test("host timelines give in-flight transforms and the first latency", () => {
  const events = [
    { t: 0, dir: "req", id: "1", method: "initialize" },
    { t: 1, dir: "rep", id: "1" },
    { t: 10, dir: "req", id: "2", method: "transform" },
    { t: 10, dir: "req", id: "3", method: "transform" },
    { t: 20, dir: "rep", id: "3" },
    { t: 30, dir: "rep", id: "2" },
  ] as const;
  const flight = concurrency(events, 2);
  assert.equal(flight.transforms, 2);
  assert.equal(flight.peakInFlight, 2);
  assert.equal(flight.windowMs, 20);
  // Two in flight for 10 ms, then one for 10 ms.
  assert.equal(flight.meanInFlight, 1.5);
  assert.equal(flight.atWorkersShare, 0.5);
  // The first transform sent, not the first answered.
  assert.equal(flight.firstLatencyMs, 20);
  assert.equal(flight.idleGaps, 0);
  const gap = concurrency(
    [
      ...events,
      { t: 35, dir: "req", id: "4", method: "transform" },
      { t: 41, dir: "rep", id: "4" },
    ],
    2,
  );
  assert.equal(gap.idleGaps, 1);
  assert.equal(gap.postIdleLatencyMs, 6);
});

test("host statistics come from --extendedDiagnostics", () => {
  const stats = hostStats(
    [
      "Parse time:                         0.140s",
      "Content mapper request wait time:   0.115s",
      "vize@0.474.0 initialization time:   0.031s",
      "vize@0.474.0 transform time:        0.734s",
      "Total time:                         0.277s",
    ].join("\n"),
  );
  assert.deepEqual(stats, {
    parse: 0.14,
    mapperWait: 0.115,
    transformSum: 0.734,
    initialization: 0.031,
    total: 0.277,
  });
});

const messages = [
  { jsonrpc: "2.0", id: 1, method: "initialize", params: { positionEncodings: ["utf-8"] } },
  { jsonrpc: "2.0", id: 2, method: "transform", params: { fileName: "/p/A.vue", content: "雪" } },
  { jsonrpc: "2.0", id: "3", method: "transform", params: { fileName: "/p/B.vue", content: "" } },
];
const bytes = Buffer.concat(messages.map(frame));
const requests: ReplayRequest[] = bodyFrames(bytes).map(({ frame, body }) => {
  const message = JSON.parse(body.toString("utf8")) as {
    id: unknown;
    method: string;
    params: { fileName?: string };
  };
  return {
    key: JSON.stringify(message.id),
    method: message.method,
    fileName: message.params.fileName,
    requestBytes: frame.length,
    frame,
    notification: false,
  };
});
const stream: ReplayStream = { label: "corpus", file: "/s.bin", cwd: "/p", bytes, requests };

const run = (mode: "burst" | "serial", answers: Record<string, string>, ms = 1): ReplayRun => ({
  mode,
  wallMs: 10 * ms,
  cpuSeconds: 0.01 * ms,
  peakMiB: 10,
  outputBytes: 100,
  errors: 0,
  hashes: new Map(Object.entries(answers)),
  replyBytes: new Map(Object.keys(answers).map((key) => [key, 10])),
  latencies: new Map(Object.keys(answers).map((key) => [key, ms])),
});
const same = { "1": "i", "2": "a", '"3"': "b" };

test("stream frames keep their exact bytes, including multi-byte bodies", () => {
  const frames = bodyFrames(bytes);
  assert.equal(frames.length, 3);
  assert.deepEqual(Buffer.concat(frames.map((f) => f.frame)), bytes);
  assert.deepEqual(
    requests.map((r) => [r.key, r.method, r.fileName]),
    [
      ["1", "initialize", undefined],
      ["2", "transform", "/p/A.vue"],
      ['"3"', "transform", "/p/B.vue"],
    ],
  );
  assert.throws(() => bodyFrames(bytes.subarray(0, bytes.length - 1)), /inside a body/);
});

test("the digest follows request order, not arrival order", () => {
  const forward = new Map([
    ["1", "i"],
    ["2", "a"],
    ['"3"', "b"],
  ]);
  const reversed = new Map([...forward].reverse());
  assert.equal(replyDigest(requests, forward), replyDigest(requests, reversed));
  assert.notEqual(replyDigest(requests, forward), replyDigest(requests, new Map([["1", "i"]])));
});

test("identical replies pass and report per-mapper timing", () => {
  const summary = summarizeReplay({
    stream,
    mappers: ["base", "head"],
    results: new Map([
      ["base", [run("burst", same, 2), run("serial", same, 2)]],
      ["head", [run("burst", same, 1), run("serial", same, 1)]],
    ]),
    slowest: 1,
    allowOutputChange: false,
  });
  assert.equal(summary.failure, undefined);
  assert.equal(summary.transforms, 2);
  assert.equal(summary.mappers[0]?.serialTotalMs, 4);
  assert.equal(summary.mappers[1]?.serialTotalMs, 2);
  assert.deepEqual(summary.slowest[0]?.ms, { base: 2, head: 1 });
  assert.match(markdown([summary]), /Replies byte-identical/);
});

test("a changed reply names the first differing request", () => {
  const summary = summarizeReplay({
    stream,
    mappers: ["base", "head"],
    results: new Map([
      ["base", [run("burst", same)]],
      ["head", [run("burst", { ...same, '"3"': "c" })]],
    ]),
    slowest: 0,
    allowOutputChange: false,
  });
  assert.match(summary.failure ?? "", /base vs head at transform \/p\/B\.vue/);
});

test("unstable runs and unanswered requests fail", () => {
  const summary = summarizeReplay({
    stream,
    mappers: ["head"],
    results: new Map([["head", [run("burst", same), run("serial", { "1": "i", "2": "a" })]]]),
    slowest: 0,
    allowOutputChange: true,
  });
  assert.match(summary.failure ?? "", /left 1 requests unanswered/);
  assert.match(summary.failure ?? "", /answered runs differently/);
});

test("reply keys come from the leading id, or from a full parse of an error", () => {
  const key = (text: string) => replyKey(Buffer.from(text));
  assert.deepEqual(key('{"id":7,"jsonrpc":"2.0","result":{"text":"{\\"id\\":1"}}'), {
    key: "7",
    error: false,
  });
  assert.deepEqual(key('{"id":"api\\"3","jsonrpc":"2.0","result":null}'), {
    key: JSON.stringify('api"3'),
    error: false,
  });
  assert.deepEqual(key('{"error":{"code":-32603,"message":"x"},"id":"a","jsonrpc":"2.0"}'), {
    key: '"a"',
    error: true,
  });
  assert.deepEqual(key('{"error":{"code":-32700,"message":"x"},"id":null,"jsonrpc":"2.0"}'), {
    key: "null",
    error: true,
  });
});

test("native mappers can answer in ready order; launchers run under node", () => {
  assert.deepEqual(parseMapper("a=/bin/vize").args, ["content-mapper"]);
  assert.deepEqual(parseMapper("a=/bin/vize", true).args, ["content-mapper", "--reply-when-ready"]);
  const launcher = parseMapper("b=node:/repo/npm/cli/bin/vize", true);
  assert.equal(launcher.command, process.execPath);
  assert.deepEqual(launcher.args, ["/repo/npm/cli/bin/vize", "content-mapper"]);
});
