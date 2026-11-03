import assert from "node:assert/strict";
import { mkdtempSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { type TestContext, test } from "node:test";

import { parseGnuTime, runOrder } from "../../tools/benchmarks/scripts/typecheck-scaling.ts";
import type { Plant } from "../../tools/benchmarks/scripts/scale-project-generator.ts";
import {
  type Diagnostic,
  gate,
  parseRun,
  splitStats,
  statNumber,
} from "../../tools/benchmarks/scripts/typecheck-scaling-gate.ts";
import {
  type Cell,
  type Sample,
  type ScalingResults,
  growthOf,
  renderSummary,
  rowOf,
} from "../../tools/benchmarks/scripts/typecheck-scaling-report.ts";
import {
  niceCeiling,
  renderScalingSvg,
} from "../../tools/benchmarks/scripts/typecheck-scaling-svg.ts";

const diagnostic = (file: string, line: number, code: string, kind = "error"): Diagnostic => [
  file,
  kind,
  String(line),
  "3",
  code,
  "message",
];
const PLANTS: Plant[] = [
  { kind: "script", file: "src/a.vue", line: 4, codes: ["TS2322"] },
  { kind: "v-model", file: "src/b.vue", line: 9, codes: ["TS2322"] },
  { kind: "emit-script", file: "src/b.vue", line: 12, codes: ["TS2345", "TS2769"] },
];

void test("the gate accepts exactly the planted errors and refuses anything else", () => {
  const exact = [
    diagnostic("src/a.vue", 4, "TS2322"),
    diagnostic("src/b.vue", 9, "TS2322"),
    diagnostic("src/b.vue", 12, "TS2769"),
  ];
  assert.equal(gate("tool", exact, PLANTS).ok, true);
  assert.equal(
    gate("tool", [...exact, diagnostic("src/c.vue", 1, "TS7006", "warning")], PLANTS).warnings,
    1,
  );
  const nothing = gate("tool", [], PLANTS);
  assert.equal(nothing.ok, false);
  assert.equal(nothing.missing.length, 3);
  const extra = gate("tool", [...exact, diagnostic("src/c.vue", 2, "TS2353")], PLANTS);
  assert.deepEqual([extra.ok, extra.unexpected.length], [false, 1]);
  const wrongCode = gate("tool", [diagnostic("src/a.vue", 4, "TS2339"), ...exact.slice(1)], PLANTS);
  assert.deepEqual(
    [wrongCode.ok, wrongCode.missing.length, wrongCode.unexpected.length],
    [false, 1, 1],
  );
  const mapperPlants: Plant[] = [PLANTS[0] as Plant, PLANTS[1] as Plant];
  assert.equal(gate("tsc-vize-mapper", [exact[0] as Diagnostic], mapperPlants).ok, true);
  assert.equal(gate("tsc-vize-mapper", exact.slice(0, 2), mapperPlants).ok, false);
  const doubled = gate("tool", [...exact, diagnostic("src/b.vue", 9, "TS2322")], PLANTS);
  assert.deepEqual([doubled.ok, doubled.unexpected.length], [false, 1]);
});

void test("statistics split off tsc output and vize JSON parses into the same diagnostics", (t: TestContext) => {
  const cwd = realpathSync(mkdtempSync(join(tmpdir(), "typecheck-scaling-")));
  t.after(() => rmSync(cwd, { recursive: true, force: true }));
  const stdout = [
    "src/a.vue(4,7): error TS2322: Type 'string' is not assignable to type 'number'.",
    "Files:                         120",
    "Types:                      54,321",
    "Instantiations:             987654",
    "Memory used:               123456K",
    "Check time:                  1.25s",
    "",
  ].join("\n");
  const { text, stats } = splitStats(stdout);
  assert.equal(text.split("\n").length, 1);
  assert.equal(statNumber(stats, ["Types"]), 54321);
  assert.equal(statNumber(stats, ["Check time"]), 1.25);
  assert.equal(statNumber(stats, ["Missing"]), null);
  assert.throws(
    () => splitStats(`${stdout}src/b.vue(1,1): error TS1: late\n`),
    /after the statistics/u,
  );
  const text_ = parseRun({ status: 2, stdout, stderr: "" }, cwd, "text");
  assert.deepEqual(
    text_.diagnostics.map((d) => d.slice(0, 5)),
    [["src/a.vue", "error", "4", "7", "TS2322"]],
  );
  const json = JSON.stringify({
    files: [
      {
        file: "src/a.vue",
        diagnostics: ["error:4:7 [TS2322] Type 'string' is not assignable to type 'number'."],
      },
    ],
    errorCount: 1,
    warningCount: 0,
  });
  const vize = parseRun({ status: 1, stdout: json, stderr: "" }, cwd, "json");
  assert.deepEqual(vize.diagnostics, text_.diagnostics);
  assert.throws(() => parseRun({ status: 0, stdout: "", stderr: "" }, cwd, "json"));
});

const sample = (cpuMs: number, rssKb: number): Sample => ({
  wallMs: cpuMs / 4,
  cpuMs,
  maxRssKb: rssKb,
  errors: 1,
  warnings: 0,
  fingerprint: "f",
  types: 10,
  instantiations: 20,
  checkTimeS: 0.5,
});
const cell = (
  tool: Cell["tool"],
  sfcs: number,
  cpuMs: number,
  rssKb: number,
  refused: string | null = null,
): Cell => ({
  tool,
  label: tool,
  vize: tool !== "vue-tsc",
  kind: "vue",
  sfcs,
  plants: 7,
  mode: "-p",
  cold: sample(cpuMs * 1.2, rssKb),
  runs: [sample(cpuMs, rssKb), sample(cpuMs * 1.1, rssKb), sample(cpuMs * 0.9, rssKb)],
  refused,
});
const results = (cells: Cell[]): ScalingResults => ({
  schemaVersion: 1,
  sha: "0".repeat(40),
  runner: "test",
  machine: "32 vCPUs, 128 GiB",
  measured: true,
  runs: 3,
  generatedAt: "2026-11-03T00:00:00.000Z",
  runUrl: null,
  nodeOptions: "--max-old-space-size=32768",
  gnuTime: "GNU time 1.9",
  provenance: { vize: { version: "1.0.0", path: "/x", sha256: "ab" } },
  cells,
});

void test("growth flags per-SFC growth and steps, and refused cells never contribute", () => {
  const cells = [
    cell("vue-tsc", 1000, 1000, 100_000),
    cell("vue-tsc", 5000, 5000, 500_000),
    cell("vue-tsc", 20000, 20000, 2_000_000),
    cell("vize-check", 1000, 1000, 100_000),
    cell("vize-check", 5000, 5000, 400_000),
    cell("vize-check", 20000, 60000, 1_000_000),
    cell("tsc-vue-mapper", 1000, 1000, 1, "missed plants"),
    cell("tsc-vue-mapper", 5000, 5000, 1),
  ];
  const growth = growthOf(cells.map(rowOf));
  const linear = growth.find((g) => g.tool === "vue-tsc");
  assert.equal(linear?.flags.length, 0);
  assert.equal(linear?.perSfc.cpuMs, 1);
  const bad = growth.find((g) => g.tool === "vize-check");
  assert.equal(bad?.perSfc.cpuMs, 3);
  assert.ok((bad?.step.cpuMs ?? 0) > 1.5);
  assert.equal(bad?.flags.length, 2);
  assert.equal(
    growth.some((g) => g.tool === "tsc-vue-mapper"),
    false,
  );
  const summary = renderSummary(results(cells));
  assert.match(summary, /## Refused \(1\)/u);
  assert.match(summary, /\| vize-check \| 20000 \| 15\.00 \| 60\.00 \|/u);
  assert.match(summary, /\*\*superlinear\*\*: per-SFC CPU ×3\.00 from 1000 to 20000/u);
});

void test("the plot draws one line per tool in each of the six panels", () => {
  assert.equal(niceCeiling(0.7), 1);
  assert.equal(niceCeiling(37), 50);
  const svg = renderScalingSvg(
    results([
      cell("vue-tsc", 1000, 1000, 100_000),
      cell("vue-tsc", 5000, 5000, 500_000),
      cell("vize-check", 1000, 900, 90_000),
      cell("vize-check", 5000, 4000, 400_000),
    ]),
  );
  assert.match(svg, /^<svg xmlns="http:\/\/www\.w3\.org\/2000\/svg"/u);
  assert.equal(svg.match(/<polyline /gu)?.length, 12);
  assert.equal(svg.match(/stroke-dasharray/gu)?.length, 7);
  assert.match(svg, /<title>vize-check, 5000 SFCs: 0\.80 ms<\/title>/u);
});

void test("GNU time records and run order", () => {
  assert.deepEqual(
    parseGnuTime("vize-time wall=1.50 user=2.00 sys=0.25 maxrss_kb=4096 status=2\n"),
    { maxRssKb: 4096, status: 2 },
  );
  assert.throws(() => parseGnuTime("Command terminated by signal 9\n"), /unexpected GNU time/u);
  assert.deepEqual(runOrder([1, 2, 3], 1), [3, 2, 1]);
});
