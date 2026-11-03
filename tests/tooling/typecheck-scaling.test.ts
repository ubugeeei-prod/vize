import assert from "node:assert/strict";
import { existsSync, mkdtempSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { type TestContext, test } from "node:test";

import { parseGnuTime, runOrder } from "../../tools/benchmarks/scripts/typecheck-scaling.ts";
import {
  MIN_SFCS,
  type Plant,
  corpusShape,
  generateMonorepo,
  generateProject,
  monorepoMapperConfigs,
  plantCounts,
} from "../../tools/benchmarks/scripts/typecheck-scaling-corpus.ts";
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
import { PLANT_KINDS } from "../../tools/benchmarks/scripts/typecheck-scaling-sfc.ts";
import { root } from "./support/github-workflows.ts";

const plantLine = (files: Map<string, string>, plant: Plant) =>
  (files.get(plant.file) ?? "").split("\n")[plant.line - 1] ?? "";

const PLANT_TEXT: Record<string, RegExp> = {
  script: /^const planted\d+: number = "deep-\d+";$/u,
  "emit-script": /emit\("apply-filter", 42\)/u,
  generic: /<KitSelect :options="rows" :label-of="\(row\) => row\.score\d+" \/>/u,
  slot: /\{\{ row\.missing\d+ \}\}/u,
  "emit-template": /@blur-commit="onPlantCommit"/u,
  "v-model": /<KitField v-model="climate"/u,
  global: /<GBadge :count="'many'" \/>/u,
};

void test("a generated project has exactly the asked SFCs and every plant on its own line", () => {
  const corpus = generateProject(MIN_SFCS + 20);
  const vue = [...corpus.files.keys()].filter((path) => path.endsWith(".vue"));
  assert.equal(vue.length, MIN_SFCS + 20);
  assert.equal(corpus.features, 2);
  assert.deepEqual(
    plantCounts(corpus.plants),
    Object.fromEntries(PLANT_KINDS.map((kind) => [kind, 2])),
  );
  for (const plant of corpus.plants) {
    assert.match(
      plantLine(corpus.files, plant),
      PLANT_TEXT[plant.kind] as RegExp,
      `${plant.file}:${plant.line}`,
    );
  }
  assert.deepEqual(generateProject(MIN_SFCS + 20).files, corpus.files);
  const generic = corpus.files.get("src/features/f0000/Layer00.vue") ?? "";
  assert.match(generic, /<script setup lang="ts" generic="T extends Row0">/u);
  assert.match(
    corpus.files.get("src/components.d.ts") ?? "",
    /GBadge: \(typeof import\("\.\/global\/GBadge\.vue"\)\)\["default"\];/u,
  );
  assert.throws(() => generateProject(MIN_SFCS - 1), /at least/u);
});

void test("plants scale with the corpus and always reach the last feature", () => {
  assert.deepEqual(corpusShape(1000), { features: 49, extraWidgets: 7 });
  const corpus = generateProject(1000);
  assert.equal(corpus.plants.length, 6 * PLANT_KINDS.length);
  assert.ok(corpus.plants.some((plant) => plant.file.startsWith("src/features/f0048/")));
  assert.ok(corpus.files.has("src/features/f0048/Widget16.vue"));
  assert.match(
    corpus.files.get("src/features/f0000/Page.vue") ?? "",
    /import Page2 from "\.\.\/f0002\/Page\.vue";/u,
  );
});

void test("the monorepo lists packages after their references and keeps mapper configs parallel", () => {
  const corpus = generateMonorepo(MIN_SFCS + 5 * 20, 6);
  assert.equal(corpus.kind, "monorepo");
  assert.equal(
    [...corpus.files.keys()].filter((path) => path.endsWith(".vue")).length,
    MIN_SFCS + 100,
  );
  const order = corpus.packages;
  assert.deepEqual(order.slice(0, 2), ["packages/kit", "packages/feat-5"]);
  assert.equal(order.at(-1), "packages/app");
  for (const [index, name] of order.entries()) {
    const config = JSON.parse(corpus.files.get(`${name}/tsconfig.json`) ?? "{}") as {
      references: { path: string }[];
      compilerOptions: { rootDir: string; outDir: string };
    };
    assert.equal(config.compilerOptions.rootDir, "src");
    for (const { path } of config.references) {
      assert.ok(order.indexOf(join(name, path)) < index, `${name} -> ${path}`);
    }
  }
  assert.deepEqual(JSON.parse(corpus.files.get("tsconfig.json") ?? "{}").files, []);
  const plantedPackages = new Set(corpus.plants.map((plant) => plant.file.split("/src/")[0]));
  assert.deepEqual([...plantedPackages].sort(), ["packages/feat-0", "packages/feat-5"]);
  assert.ok(corpus.files.has("packages/feat-3/src/components.d.ts"));
  const mapper = monorepoMapperConfigs(
    corpus,
    { id: "vize", package: "vize" },
    "tsconfig.vize-mapper.json",
  );
  const feat = JSON.parse(mapper.get("packages/feat-0/tsconfig.vize-mapper.json") ?? "{}") as {
    references: { path: string }[];
    compilerOptions: { outDir: string };
  };
  assert.equal(feat.compilerOptions.outDir, ".out-vize-mapper");
  assert.ok(feat.references.every(({ path }) => path.endsWith("/tsconfig.vize-mapper.json")));
  assert.equal(
    JSON.parse(mapper.get("tsconfig.vize-mapper.json") ?? "{}").references.length,
    order.length,
  );
});

const diagnostic = (file: string, line: number, code: string, kind = "error"): Diagnostic => [
  file,
  kind,
  String(line),
  "3",
  code,
  "message",
];
const PLANTS: Plant[] = [
  { kind: "script", file: "src/a.vue", line: 4 },
  { kind: "v-model", file: "src/b.vue", line: 9 },
  { kind: "emit-script", file: "src/b.vue", line: 12 },
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
  kind: "project",
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

void test(
  "vue-tsc reports exactly the planted errors of a small corpus",
  {
    skip:
      !existsSync(join(root, "tools/benchmarks/scripts/node_modules/vue-tsc")) &&
      "vue-tsc is not installed",
  },
  async (t: TestContext) => {
    const { spawnSync } = await import("node:child_process");
    const { writeCorpus } =
      await import("../../tools/benchmarks/scripts/typecheck-scaling-corpus.ts");
    const dir = realpathSync(mkdtempSync(join(tmpdir(), "typecheck-scaling-vue-tsc-")));
    t.after(() => rmSync(dir, { recursive: true, force: true }));
    const corpus = generateProject(MIN_SFCS);
    writeCorpus(dir, corpus.files, false);
    const modules = join(root, "tools/benchmarks/scripts/node_modules");
    const { symlinkSync, mkdirSync } = await import("node:fs");
    mkdirSync(join(dir, "node_modules"));
    symlinkSync(realpathSync(join(modules, "vue")), join(dir, "node_modules", "vue"));
    const vueTsc = join(realpathSync(join(modules, "vue-tsc")), "bin", "vue-tsc.js");
    const result = spawnSync(
      process.execPath,
      [vueTsc, "--noEmit", "--pretty", "false", "-p", join(dir, "tsconfig.json")],
      { cwd: dir, encoding: "utf8" },
    );
    const parsed = parseRun(
      { status: result.status, stdout: result.stdout, stderr: result.stderr },
      dir,
      "text",
    );
    const verdict = gate("vue-tsc", parsed.diagnostics, corpus.plants);
    assert.equal(verdict.ok, true, [...verdict.missing, ...verdict.unexpected].join("\n"));
  },
);
