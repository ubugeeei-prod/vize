import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test, type TestContext } from "node:test";

import {
  type Diagnostic,
  assertExactDiagnosticFingerprints,
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
  diffBatchMapper,
  renderBatchDiff,
} from "../../tools/benchmarks/scripts/typecheck-corpus-pair-batch.ts";
import {
  hostArgs,
  PAIR_HOST_ARGS,
} from "../../tools/benchmarks/scripts/typecheck-corpus-pair-host.ts";
import { MAPPER_HOSTS } from "../../tools/benchmarks/scripts/typecheck-mapper-hosts.ts";
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

void test("base and head diagnostic fingerprints must match exactly", () => {
  assert.doesNotThrow(() =>
    assertExactDiagnosticFingerprints([{ label: "misskey-tsc", base: "same", head: "same" }]),
  );
  assert.throws(
    () =>
      assertExactDiagnosticFingerprints([
        { label: "misskey-tsc", base: "base", head: "head" },
        { label: "n8n-tsrs", base: "base", head: "head" },
      ]),
    /misskey-tsc, n8n-tsrs/u,
  );
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
  assert.deepEqual(
    probeRecord(
      7,
      "/p/A.vue",
      { sourceBytes: 20, templateBytes: 2 },
      { id: 1, result: { text: "é;" } },
    ),
    {
      pid: 7,
      fileName: "/p/A.vue",
      bytes: 3,
      sourceBytes: 20,
      templateBytes: 2,
      replyBytes: Buffer.byteLength(JSON.stringify({ text: "é;" })),
      spans: 0,
      directives: 0,
      ignoreGaps: 0,
    },
  );
  const shaped = {
    text: "a;\n",
    mappings: [
      [0, 1, 0, 1, 0],
      [1, 1, 1, 1, 0, 0],
    ],
    diagnosticDirectives: {
      directives: [
        [0, 0, 2, 3, 0],
        [4, 2, 0, 1, 1, 0],
      ],
      unusedExpectDirectiveDiagnostics: [],
    },
  };
  assert.deepEqual(
    probeRecord(7, "/p/S.vue", { sourceBytes: 30, templateBytes: 8 }, { id: 3, result: shaped }),
    {
      pid: 7,
      fileName: "/p/S.vue",
      bytes: 3,
      sourceBytes: 30,
      templateBytes: 8,
      replyBytes: Buffer.byteLength(JSON.stringify(shaped)),
      spans: 2,
      directives: 2,
      ignoreGaps: 1,
    },
  );
  assert.match(
    (
      probeRecord(7, "/p/B.vue", { sourceBytes: 1, templateBytes: 0 }, { id: 2, result: null }) as {
        error: string;
      }
    ).error,
    /no text/u,
  );
  const lines = [
    {
      pid: 1,
      fileName: "/p/A.vue",
      bytes: 10,
      sourceBytes: 20,
      templateBytes: 2,
      replyBytes: 20,
      spans: 4,
      directives: 2,
      ignoreGaps: 1,
    },
    {
      pid: 2,
      fileName: "/p/A.vue",
      bytes: 12,
      sourceBytes: 20,
      templateBytes: 2,
      replyBytes: 24,
      spans: 5,
      directives: 3,
      ignoreGaps: 2,
    },
    { pid: 1, fileName: "/p/B.vue", bytes: 5, sourceBytes: 7, templateBytes: 0 },
  ];
  assert.deepEqual(summarizeProbe(lines.map((line) => JSON.stringify(line)).join("\n")), {
    files: 2,
    transforms: 3,
    processes: 2,
    bytes: 17,
    sourceBytes: 27,
    templateBytes: 2,
    replyBytes: 24,
    spans: 5,
    directives: 3,
    ignoreGaps: 2,
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
      params: { fileName: "/p/A.vue", content: "<template>é</template>" },
    }),
    frame({
      jsonrpc: "2.0",
      id: 3,
      method: "transform",
      params: {
        fileName: "/p/B.vue",
        content: "<script setup>const n=1</script><template>bb</template>",
      },
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
    {
      cwd: dir,
      input,
      env: {
        ...process.env,
        VIZE_MAPPER_PROBE_RECORD: record,
        VIZE_MAPPER_PROBE_DUMP: join(dir, "dump"),
        VIZE_MAPPER_PROBE_ROOT: "/p",
      },
    },
  );
  assert.equal(probe.status, 0, probe.stderr.toString());
  const authored = {
    a: "<template>é</template>",
    b: "<script setup>const n=1</script><template>bb</template>",
  };
  assert.equal(
    readFileSync(join(dir, "dump", "B.vue.tsx"), "utf8"),
    `// ${mapperDir} é ${authored.b}`,
  );
  assert.deepEqual(JSON.parse(readFileSync(join(dir, "dump", "B.vue.tsx.shape.json"), "utf8")), {
    spans: [],
    ignoreGaps: [],
    directives: [],
  });
  assert.deepEqual(probe.stdout, direct.stdout);
  const text = (content: string) => Buffer.byteLength(`// ${mapperDir} é ${content}`);
  const summary = summarizeProbe(readFileSync(record, "utf8"));
  assert.deepEqual(
    {
      files: summary.files,
      transforms: summary.transforms,
      bytes: summary.bytes,
      sourceBytes: summary.sourceBytes,
      templateBytes: summary.templateBytes,
    },
    {
      files: 2,
      transforms: 2,
      bytes: text(authored.a) + text(authored.b),
      sourceBytes: Buffer.byteLength(authored.a) + Buffer.byteLength(authored.b),
      templateBytes: Buffer.byteLength("é") + Buffer.byteLength("bb"),
    },
  );
  const missing = spawnSync(
    process.execPath,
    [join(root, "tools/benchmarks/scripts/content-mapper-byte-probe.ts"), mapperDir, "--", "node"],
    { cwd: dir, input, env: { ...process.env, VIZE_MAPPER_PROBE_RECORD: "" } },
  );
  assert.equal(missing.status, 2);
});

void test("the pair runs tsc at its default and tsrs at --fast, each a published host setting", () => {
  assert.deepEqual(PAIR_HOST_ARGS, { tsc: [], tsrs: ["--fast"] });
  for (const host of MAPPER_HOSTS) {
    const published = host.settings.map((setting) => setting.args(32));
    assert.ok(
      published.some((args) => args.join(" ") === PAIR_HOST_ARGS[host.id].join(" ")),
      host.id,
    );
  }
});

void test("mapper commands preserve the pair settings and projection trace arguments", () => {
  for (const host of MAPPER_HOSTS) {
    const extraArgs = host.id === "tsc" ? ["--generateTrace", "/work/traces"] : [];
    assert.deepEqual(hostArgs(host, "/work/project", "tsconfig.mapper.json", extraArgs), [
      "--noEmit",
      ...PAIR_HOST_ARGS[host.id],
      "--runExternalCode",
      "--extendedDiagnostics",
      "--pretty",
      "false",
      ...extraArgs,
      "-p",
      "/work/project/tsconfig.mapper.json",
    ]);
  }
});

void test("the batch diff keeps every diagnostic only one path reports, matched by position and code", () => {
  const at = (file: string, line: string, code: string, message = "m"): Diagnostic => [
    file,
    "error",
    line,
    "3",
    code,
    message,
  ];
  const diff = diffBatchMapper(
    "demo",
    [
      at("A.vue", "1", "TS2345"),
      at("A.vue", "1", "TS2345"),
      at("B.vue", "4", "TS2322", "batch text"),
    ],
    [
      at("A.vue", "1", "TS2345"),
      at("B.vue", "4", "TS2322", "mapper text"),
      at("C.vue", "9", "TS7006"),
    ],
  );
  assert.deepEqual(diff.batchOnly, [at("A.vue", "1", "TS2345")]);
  assert.deepEqual(diff.mapperOnly, [at("C.vue", "9", "TS7006")]);
  const markdown = renderBatchDiff([diff]);
  assert.match(
    markdown,
    /3 diagnostics, tsc with the Vize mapper 3; 1 only in batch, 1 only through the mapper/u,
  );
  assert.match(markdown, /\| `C\.vue` \| 9:3 \| TS7006 \| m \|/u);
});

void test("the batch report refuses hidden tsrs fast-mode and width drift", () => {
  const at = (file: string, code: string): Diagnostic => [file, "error", "1", "1", code, "m"];
  const markdown = renderBatchDiff([
    {
      corpus: "demo",
      batch: 2,
      mapper: 2,
      batchOnly: [],
      mapperOnly: [],
      tsrs: {
        diagnostics: 2,
        only: [at("Tsrs.vue", "TS2322")],
        missing: [at("Tsgo.vue", "TS2345")],
        wide: {
          checkers: 8,
          added: [at("Wide.vue", "TS7006")],
          removed: [at("Narrow.vue", "TS2339")],
        },
      },
    },
  ]);
  assert.match(markdown, /2 diagnostics; 1 only under tsrs and 1 only under tsgo/u);
  assert.match(markdown, /At 8 checkers, tsrs adds 1 and drops 1/u);
  assert.match(markdown, /Only under tsrs batch/u);
  assert.match(markdown, /Only under tsgo batch/u);
});
