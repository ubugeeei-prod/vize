import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  baselineToml,
  loadBudgets,
} from "../../tools/benchmarks/scripts/instruction-counts-lib.mjs";
import {
  parseHotspots,
  reportOverBudgetHotspots,
} from "../../tools/benchmarks/scripts/instruction-hotspots.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const dump = (body: string, total: number, id = "over_budget", positions = "line") =>
  `# callgrind format\nversion: 1\ndesc: Trigger: Client Request: ${id}\n` +
  `positions: ${positions}\nevents: Ir\n${body}\ntotals: ${total}\n`;
const body =
  "fn=main\n1 12\ncfn=helper\ncalls=1 1\n1 50\n" +
  "cfn=leaf\ncalls=3 1\n1 30\nfn=helper\n1 20\ncfn=leaf\ncalls=2 1\n1 30\n" +
  "fn=leaf\n1 60";

void test("self costs and inclusive call-edge costs match independent function goldens", () => {
  const parsed = parseHotspots(dump(body, 92));
  assert.equal(parsed.instructions, 92);
  assert.deepEqual(
    parsed.rows.map(({ function: fn, exclusive, inclusive }) => [fn, exclusive, inclusive]),
    [
      ["main", 12, 92],
      ["helper", 20, 50],
      ["leaf", 60, 60],
    ],
  );
  // Inclusive rows overlap; only exclusive costs sum to the measured window.
  assert.equal(
    parsed.rows.reduce((sum, row) => sum + row.exclusive, 0),
    92,
  );
  const recursive = parseHotspots(dump("fn=recur\n1 5\ncfn=recur\ncalls=2 1\n1 20", 5));
  assert.deepEqual(
    recursive.rows.map(({ exclusive, inclusive }) => [exclusive, inclusive]),
    [[5, 25]],
  );
});

void test("compressed shared names, multiple positions and object ownership retain costs", () => {
  const text =
    "ob=(1) app\nfl=(1) inline.rs\nfn=(1) eval\n0x100 10 2\n+4 * 3\n" +
    "cfn=(2) nested\ncalls=2 1 2\n* * 7\nfn=(2)\n* +1 7\n" +
    "cob=(2) libc\nob=(2)\nfn=(1)\n* * 11\nob=(1)\nfn=(1)\n* * 2";
  assert.deepEqual(parseHotspots(dump(text, 25, "over_budget", "instr line")).rows, [
    { object: "app", function: "eval", exclusive: 7, inclusive: 14 },
    { object: "app", function: "nested", exclusive: 7, inclusive: 7 },
    { object: "libc", function: "eval", exclusive: 11, inclusive: 11 },
  ]);
  assert.equal(parseHotspots(dump("fn=zero\n1\nfn=work\n+1 0xA", 10)).rows[0].exclusive, 0);
});

void test("malformed diagnostics cannot mint plausible function costs", () => {
  for (const invalid of [
    dump("fn=(1)\n1 1", 1),
    dump("fn=(1) first\nfn=(1) second\n1 1", 1),
    dump("1 1", 1),
    dump("fn=work\ncalls=1 1", 1),
    dump("fn=work\ncalls=1 1\n1 1", 1),
    dump("fn=work\ncalls=1 1\nfn=other\n1 1", 1),
    dump("fn=work\n1 9007199254740992", 1),
    dump("fn=work\n1 1 99", 1),
    dump("fn=work\n1 1", 2),
    dump("fn=work\n1 1", 1, "over_budget", "address"),
    dump("fn=work\n1 1 1", 1, "over_budget", "line line"),
  ])
    assert.throws(() => parseHotspots(invalid));
  assert.throws(() => parseHotspots(dump(body, 92).replace("events: Ir", "events: Ir Dr")));
});

void test("reporter reads only failed first-run workloads and retains exact top costs", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "instruction-hotspots-"));
  try {
    fs.mkdirSync(path.join(directory, "run-1"));
    fs.writeFileSync(path.join(directory, "run-1", "suite.callgrind.1"), dump(body, 92));
    fs.writeFileSync(
      path.join(directory, "run-1", "suite.callgrind.2"),
      dump("bad body intentionally unread", 10, "within_budget"),
    );
    fs.writeFileSync(path.join(directory, "run-1", "suite.callgrind.stderr"), "unread stderr");
    const lines: string[] = [];
    reportOverBudgetHotspots(
      path.join(directory, "measurement.json"),
      {
        runs: [{ over_budget: { instructions: 92 }, within_budget: { instructions: 10 } }],
      },
      { instruction: { over_budget: { instructions: 90 }, within_budget: { instructions: 10 } } },
      (line) => lines.push(line),
    );
    assert.equal(lines.length, 7);
    const rows = lines
      .slice(1)
      .map((line) => JSON.parse(line.slice("instruction-hotspot: ".length)));
    assert.deepEqual(
      rows.map(({ mode, function: fn, instructions }) => [mode, fn, instructions]),
      [
        ["exclusive", "leaf", 60],
        ["exclusive", "helper", 20],
        ["exclusive", "main", 12],
        ["inclusive", "main", 92],
        ["inclusive", "leaf", 60],
        ["inclusive", "helper", 50],
      ],
    );
    assert.ok(lines.every((line) => !line.includes("within_budget")));
    fs.writeFileSync(path.join(directory, "run-1", "suite.callgrind.3"), dump(body, 92));
    assert.throws(
      () =>
        reportOverBudgetHotspots(
          path.join(directory, "measurement.json"),
          {
            runs: [{ over_budget: { instructions: 92 } }],
          },
          { instruction: { over_budget: { instructions: 90 } } },
          () => {},
        ),
      /duplicate/,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("real strict CLI reports completed-dump hotspots and keeps the original hard failure", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "instruction-hotspots-cli-"));
  try {
    const { methodology } = loadBudgets(
      path.join(root, "docs/davinci/plan/instruction-budgets.toml"),
    );
    const record = (instructions: number) => ({
      instructions,
      fixture: "synthetic:probe",
      fixture_sha256: "a".repeat(64),
      window: "routine-return",
    });
    const metadata = {
      schema_version: 1,
      source_commit: "a".repeat(40),
      recorded_run: "https://github.com/ubugeeei-prod/vize/actions/runs/123",
      methodology,
    };
    const report = {
      ...metadata,
      source_commit: execFileSync("git", ["rev-parse", "HEAD"], {
        cwd: root,
        encoding: "utf8",
      }).trim(),
      runs: Array.from({ length: 3 }, () => ({ over_budget: record(92) })),
    };
    const budget = {
      ...metadata,
      runs: Array.from({ length: 3 }, () => ({ over_budget: record(90) })),
    };
    fs.writeFileSync(path.join(directory, "measurement.json"), JSON.stringify(report));
    fs.writeFileSync(path.join(directory, "budgets.toml"), baselineToml(budget));
    fs.writeFileSync(path.join(directory, "registry.toml"), "[bench.over_budget]\nbytes = 1\n");
    fs.mkdirSync(path.join(directory, "run-1"));
    fs.writeFileSync(path.join(directory, "run-1", "suite.callgrind.1"), dump(body, 92));
    const run = () =>
      spawnSync(
        process.execPath,
        [
          path.join(root, "tools/benchmarks/scripts/instruction-counts.mjs"),
          "--check",
          "--measurement",
          path.join(directory, "measurement.json"),
          "--budgets",
          path.join(directory, "budgets.toml"),
          "--registry",
          path.join(directory, "registry.toml"),
        ],
        { encoding: "utf8" },
      );
    fs.writeFileSync(
      path.join(directory, "measurement.json"),
      JSON.stringify({ ...report, source_commit: metadata.source_commit }),
    );
    let result = run();
    assert.equal(result.status, 1);
    assert.match(result.stderr, /measurement source is not current checkout/u);
    assert.doesNotMatch(result.stdout, /instruction-hotspot/u);
    fs.writeFileSync(path.join(directory, "measurement.json"), JSON.stringify(report));
    result = run();
    assert.equal(result.status, 1);
    assert.match(result.stdout, /instruction-hotspot: .*"function":"leaf"/u);
    assert.match(result.stderr, /instruction budget exceeded:[\s\S]*over_budget: 92 > 90/u);
    fs.writeFileSync(path.join(directory, "run-1", "suite.callgrind.1"), "broken dump");
    result = run();
    assert.equal(result.status, 1);
    assert.match(result.stderr, /instruction-hotspots: unavailable:/u);
    assert.match(result.stderr, /instruction budget exceeded:[\s\S]*over_budget: 92 > 90/u);
    fs.rmSync(path.join(directory, "run-1"), { recursive: true });
    report.runs = Array.from({ length: 3 }, () => ({ over_budget: record(90) }));
    fs.writeFileSync(path.join(directory, "measurement.json"), JSON.stringify(report));
    result = run();
    assert.equal(result.status, 0);
    assert.match(result.stdout, /1 ceilings hold/u);
    assert.doesNotMatch(result.stdout + result.stderr, /instruction-hotspot/u);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
