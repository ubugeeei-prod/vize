import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test, type TestContext } from "node:test";

import {
  type Diagnostic,
  diffDiagnostics,
  frameReader,
  median,
  parseGnuTime,
  parseHostRun,
  probeRecord,
  runOrder,
  splitHostOutput,
  statValue,
  summarizeProbe,
} from "../../tools/benchmarks/scripts/typecheck-corpus-pair-parse.ts";
import {
  type LaneResult,
  type PairResults,
  percentDelta,
  renderPairSummary,
  summarizePair,
} from "../../tools/benchmarks/scripts/typecheck-corpus-pair-report.ts";
import { root } from "./support/github-workflows.ts";

function scratch(t: TestContext): string {
  const dir = realpathSync(mkdtempSync(join(tmpdir(), "typecheck-corpus-pair-")));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}

const TSC_STATS = [
  "Files:                                1132",
  "Types:                             1217902",
  "Instantiations:                    8914409",
  "Memory used:                      1231776K",
  "Check time:                         2.549s",
  "vize@0.654.0 transform time:      116.570s",
  "Total time:                         3.142s",
];

void test("host output splits into diagnostics and the extended statistics block", () => {
  const stdout = [
    "src/a.vue(1,2): error TS2322: Type 'string' is not assignable to type 'number'.",
    ...TSC_STATS,
    "Program:   loader rounds:                   9",
    "",
  ].join("\n");
  const { diagnosticText, stats } = splitHostOutput(stdout);
  assert.equal(
    diagnosticText,
    "src/a.vue(1,2): error TS2322: Type 'string' is not assignable to type 'number'.",
  );
  assert.equal(statValue(stats, "Types", "count"), 1217902);
  assert.equal(statValue(stats, "Check time", "seconds"), 2.549);
  assert.equal(statValue(stats, "Memory used", "kilobytes"), 1231776);
  assert.equal(statValue(stats, "Program:   loader rounds", "count"), 9);
  assert.throws(() => statValue(stats, "Check time", "count"), /not in count/u);
  assert.throws(() => statValue(stats, "Emit time", "seconds"), /occurs 0 times/u);
  assert.throws(() => statValue([...stats, ["Types", "1"]], "Types", "count"), /occurs 2 times/u);
  assert.throws(
    () => splitHostOutput("src/a.ts(1,1): error TS1: x\n"),
    /no --extendedDiagnostics/u,
  );
  assert.throws(() => splitHostOutput(`${TSC_STATS.join("\n")}\npanic: boom\n`), /unexpected/u);
});

void test("host diagnostics are normalized, sorted and fingerprinted; stray output fails", (t) => {
  const cwd = scratch(t);
  const run = (lines: string[], status = 2) =>
    parseHostRun({ status, stdout: [...lines, ...TSC_STATS].join("\n"), stderr: "" }, cwd);
  const first = run([
    "src/b.vue(3,4): error TS2345: Argument of type 'X' is not assignable.",
    "  Type 'X' is missing properties.",
    "src/a.ts(1,1): error TS2307: Cannot find module 'm'.",
  ]);
  assert.deepEqual(
    first.diagnostics.map((d: Diagnostic) => [d[0], d[2], d[3], d[4]]),
    [
      ["src/a.ts", "1", "1", "TS2307"],
      ["src/b.vue", "3", "4", "TS2345"],
    ],
  );
  assert.match((first.diagnostics[1] as Diagnostic)[5], /\n {2}Type 'X' is missing properties\.$/u);
  const reordered = run([
    "src/a.ts(1,1): error TS2307: Cannot find module 'm'.",
    "src/b.vue(3,4): error TS2345: Argument of type 'X' is not assignable.",
    "  Type 'X' is missing properties.",
  ]);
  assert.equal(reordered.fingerprint, first.fingerprint);
  const changed = run(["src/a.ts(1,2): error TS2307: Cannot find module 'm'."]);
  assert.notEqual(changed.fingerprint, first.fingerprint);
  assert.throws(() => run(["mapper exploded"]), /unexpected type-check output/u);
  assert.throws(() => run([], 2), /exit status disagrees/u);
});

void test("GNU time records parse with or without the abnormal-exit note", () => {
  const line = "vize-time wall=4.07 user=16.54 sys=1.25 maxrss_kb=1294336 status=2";
  const expected = { wallS: 4.07, userS: 16.54, sysS: 1.25, maxRssKb: 1294336, status: 2 };
  assert.deepEqual(parseGnuTime(`${line}\n`), expected);
  assert.deepEqual(parseGnuTime(`Command exited with non-zero status 2\n${line}\n`), expected);
  assert.throws(() => parseGnuTime(""), /unexpected GNU time output/u);
  assert.throws(() => parseGnuTime(`${line}\n${line}\n`), /unexpected GNU time output/u);
  assert.throws(() => parseGnuTime(`${line}\nreal 4.07\n`), /unexpected GNU time output/u);
});

void test("diagnostic difference is a multiset difference", () => {
  const d = (line: string): Diagnostic => ["src/a.vue", "error", line, "1", "TS2322", "x"];
  const { added, removed } = diffDiagnostics([d("1"), d("1"), d("2")], [d("1"), d("2"), d("3")]);
  assert.deepEqual(added, [d("3")]);
  assert.deepEqual(removed, [d("1")]);
  assert.deepEqual(diffDiagnostics([d("1")], [d("1")]), { added: [], removed: [] });
});

void test("medians and alternating run order", () => {
  assert.equal(median([3, 1, 2]), 2);
  assert.equal(median([4, 1, 3, 2]), 2.5);
  assert.throws(() => median([]), /no values/u);
  assert.deepEqual(runOrder(["a", "b", "c"], 0), ["a", "b", "c"]);
  assert.deepEqual(runOrder(["a", "b", "c"], 1), ["c", "b", "a"]);
});

const frame = (value: unknown) => {
  const body = Buffer.from(JSON.stringify(value));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
};

void test("frames are read across chunk boundaries", () => {
  const read = frameReader();
  const first = frame({ id: 1, result: "é" });
  const bytes = Buffer.concat([first, frame({ id: 2, method: "x" })]);
  assert.deepEqual(read(bytes.subarray(0, 7)), []);
  assert.deepEqual(read(bytes.subarray(7, first.length - 1)), []);
  assert.deepEqual(read(bytes.subarray(first.length - 1, first.length + 3)), [
    { id: 1, result: "é" },
  ]);
  assert.deepEqual(read(bytes.subarray(first.length + 3)), [{ id: 2, method: "x" }]);
  assert.throws(() => frameReader()(Buffer.from("X: 1\r\n\r\n{}")), /without Content-Length/u);
});

void test("generated bytes count the last UTF-8 text per file; errors are kept", () => {
  assert.deepEqual(probeRecord(7, "/p/A.vue", { id: 1, result: { text: "é;" } }), {
    pid: 7,
    fileName: "/p/A.vue",
    bytes: 3,
  });
  assert.match(
    (probeRecord(7, "/p/B.vue", { id: 2, result: null }) as { error: string }).error,
    /no text/u,
  );
  const lines = [
    { pid: 1, fileName: "/p/A.vue", bytes: 10 },
    { pid: 2, fileName: "/p/A.vue", bytes: 12 },
    { pid: 1, fileName: "/p/B.vue", bytes: 5 },
  ];
  assert.deepEqual(summarizeProbe(lines.map((line) => JSON.stringify(line)).join("\n")), {
    files: 2,
    transforms: 3,
    processes: 2,
    bytes: 17,
    errors: [],
  });
  const failed = summarizeProbe(JSON.stringify({ pid: 1, fileName: "/p/C.vue", error: "boom" }));
  assert.deepEqual(failed.errors, ["/p/C.vue: boom"]);
});

const FAKE_MAPPER = `
let buffer = Buffer.alloc(0);
const send = (value) => {
  const body = Buffer.from(JSON.stringify(value));
  process.stdout.write(Buffer.concat([Buffer.from("Content-Length: " + body.length + "\\r\\n\\r\\n"), body]));
};
process.stdin.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  for (;;) {
    const end = buffer.indexOf("\\r\\n\\r\\n");
    if (end < 0) return;
    const length = Number(/Content-Length: (\\d+)/.exec(buffer.subarray(0, end).toString())[1]);
    if (buffer.length < end + 4 + length) return;
    const message = JSON.parse(buffer.subarray(end + 4, end + 4 + length).toString());
    buffer = buffer.subarray(end + 4 + length);
    if (message.method === "transform") {
      send({ jsonrpc: "2.0", id: message.id, result: { text: "// " + process.cwd() + " é " + message.params.content } });
    } else {
      send({ jsonrpc: "2.0", id: message.id, result: { method: message.method } });
    }
  }
});
`;

void test("the byte probe forwards the mapper's bytes unchanged and records each transform", (t) => {
  const dir = scratch(t);
  const mapperDir = join(dir, "mapper");
  mkdirSync(mapperDir);
  writeFileSync(join(mapperDir, "server.mjs"), FAKE_MAPPER);
  const input = Buffer.concat([
    frame({ jsonrpc: "2.0", id: 1, method: "initialize", params: {} }),
    frame({
      jsonrpc: "2.0",
      id: 2,
      method: "transform",
      params: { fileName: "/p/A.vue", content: "a" },
    }),
    frame({
      jsonrpc: "2.0",
      id: 3,
      method: "transform",
      params: { fileName: "/p/B.vue", content: "bb" },
    }),
  ]);
  const direct = spawnSync(process.execPath, ["server.mjs"], { cwd: mapperDir, input });
  const record = join(dir, "probe.jsonl");
  const probe = spawnSync(
    process.execPath,
    [
      join(root, "tools/benchmarks/scripts/content-mapper-byte-probe.ts"),
      mapperDir,
      "--",
      process.execPath,
      "server.mjs",
    ],
    { cwd: dir, input, env: { ...process.env, VIZE_MAPPER_PROBE_RECORD: record } },
  );
  assert.equal(probe.status, 0, probe.stderr.toString());
  assert.deepEqual(probe.stdout, direct.stdout);
  const text = (content: string) => Buffer.byteLength(`// ${mapperDir} é ${content}`);
  const summary = summarizeProbe(readFileSync(record, "utf8"));
  assert.deepEqual(
    { files: summary.files, transforms: summary.transforms, bytes: summary.bytes },
    { files: 2, transforms: 2, bytes: text("a") + text("bb") },
  );
  const missing = spawnSync(
    process.execPath,
    [join(root, "tools/benchmarks/scripts/content-mapper-byte-probe.ts"), mapperDir, "--", "node"],
    { cwd: dir, input, env: { ...process.env, VIZE_MAPPER_PROBE_RECORD: "" } },
  );
  assert.equal(missing.status, 2);
});

function lane(
  host: string,
  variant: LaneResult["variant"],
  cpuMs: number[],
  diagnostics: Diagnostic[],
  fingerprint = `${host}-${diagnostics.length}`,
): LaneResult {
  const stats: [string, string][] = [
    ["Types", "100"],
    ["Instantiations", "1000"],
    ["Check time", "1.500s"],
  ];
  return {
    corpus: "misskey",
    corpusLabel: "misskey frontend",
    sfcs: 583,
    host,
    variant,
    probe: { files: 583, transforms: 583, processes: 1, bytes: 2048, fingerprint },
    diagnostics,
    runs: cpuMs.map((ms) => ({
      wallMs: ms / 4,
      cpuMs: ms,
      time: { wallS: 1, userS: 1, sysS: 0, maxRssKb: 1024 * 512, status: 2 },
      status: 2,
      stats,
      fingerprint,
    })),
  };
}

void test("the summary tables medians per lane and reports diagnostic identity", () => {
  const d = (line: string): Diagnostic => ["src/a.vue", "error", line, "1", "TS2322", "x"];
  const results: PairResults = {
    schemaVersion: 1,
    base: { sha: "a".repeat(40) },
    head: { sha: "b".repeat(40) },
    runs: 3,
    measurement: {
      runner: "blacksmith-32vcpu-ubuntu-2404",
      packages: {
        typescript: "7.1.0-beta",
        tsrs: "0.20.1-ts7.1.0-beta",
        "@vue/content-mapper": "4.0.0-alpha.3",
        vue: "3.5.44",
        "@types/node": "26.6.6",
      },
    },
    lanes: [
      lane("tsc", "vize-base", [10_000, 12_000, 11_000], [d("1")]),
      lane("tsc", "vize-head", [9_000, 9_900, 9_500], [d("1")]),
      lane("tsc", "vue", [20_000, 20_000, 20_000], [d("1"), d("2")]),
      lane("tsrs", "vize-base", [4_000, 4_000, 4_000], [d("1")]),
      lane("tsrs", "vize-head", [4_000, 4_000, 4_000], [d("1"), d("3")]),
      lane("tsrs", "vue", [5_000, 5_000, 5_000], [d("1")]),
    ],
  };
  const summary = summarizePair(results);
  assert.deepEqual(
    summary.identity.map((entry) => [entry.host, entry.status, entry.added.length]),
    [
      ["tsc", "IDENTICAL", 0],
      ["tsrs", "CHANGED", 1],
    ],
  );
  assert.deepEqual(summary.unstable, []);
  const markdown = renderPairSummary(results, summary);
  assert.match(
    markdown,
    /\| misskey frontend \(583 SFC\) \| tsc Vize base \| tsc Vize head \| Δ \| tsc Vue \| tsrs Vize base/u,
  );
  assert.match(
    markdown,
    /\| CPU s \(user\+sys, process tree\) \| 11\.00 \| 9\.50 \| −13\.6% \| 20\.00 \| 4\.00 \| 4\.00 \| 0\.0% \| 5\.00 \|/u,
  );
  // The fastest run shows when a median sits on a different mode than its pair.
  assert.match(
    markdown,
    /\| CPU s, fastest run \| 10\.00 \| 9\.00 \| −10\.0% \| 20\.00 \| 4\.00 \| 4\.00 \| 0\.0% \| 5\.00 \|/u,
  );
  assert.match(markdown, /\| Diagnostics \| 1 \| 1 \| IDENTICAL \| 2 \| 1 \| 2 \| \+1 −0 \| 1 \|/u);
  assert.match(markdown, /misskey \/ tsrs: \*\*CHANGED\*\*, base 1 → head 2 \(\+1 −0\)/u);
  assert.match(markdown, /\+ `src\/a\.vue\(3,1\): error TS2322: x`/u);

  const drifting = lane("tsc", "vize-head", [1, 1], [d("1")]);
  (drifting.runs[1] as { fingerprint: string }).fingerprint = "other";
  assert.deepEqual(
    summarizePair({
      ...results,
      lanes: [...results.lanes.slice(0, 1), drifting, ...results.lanes.slice(2)],
    }).unstable,
    ["misskey-tsc-vize-head"],
  );
  assert.equal(percentDelta(0, 0), "0.0%");
  assert.equal(percentDelta(2, 3), "+50.0%");
});
