#!/usr/bin/env node
/** Same-runner, alternating base/head qualification for the nonlinear scaling budget. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { availableParallelism, totalmem } from "node:os";
import { createRequire } from "node:module";
import { basename, dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { assertFileCoverage, checkPacket } from "./pr-performance-observation-check.ts";
import {
  binaryEvidence,
  corpusInventory,
  sha256,
  writeJson,
} from "./pr-performance-observation-evidence.ts";
import { verifyObservationBindings } from "./pr-performance-observation-binding.ts";
import {
  generateScaleProject,
  type ScaleProject,
  writeProject,
} from "./scale-project-generator.ts";
import { type ProcessOutput, gate, parseRun, refusalText } from "./typecheck-scaling-gate.ts";
import { formatGrowthBudgetFailures } from "./typecheck-scaling-budget.ts";
import {
  type Cell,
  type Sample,
  median,
  rowOf,
  scalingBudget,
} from "./typecheck-scaling-report.ts";
import { measure, NODE_OPTIONS } from "./typecheck-scaling.ts";

type Role = "base" | "head";
type PairedSample = Sample & {
  role: Role;
  phase: "cold" | "timing";
  cycle: number;
  order: number;
  packetDigest: string;
  fileCount: number;
  isolation: { cache: string; temp: string };
};

export const pairedOrder = (cycle: number): readonly Role[] =>
  cycle % 2 === 0 ? ["base", "head"] : ["head", "base"];

export function packetDigest(packet: ReturnType<typeof checkPacket>): string {
  return createHash("sha256").update(JSON.stringify(packet)).digest("hex");
}

export function assertPairedDigests(
  samples: readonly Pick<PairedSample, "packetDigest">[],
): string {
  assert.ok(samples.length > 0, "no complete diagnostic packets");
  const digests = new Set(samples.map((sample) => sample.packetDigest));
  assert.equal(digests.size, 1, "base/head complete diagnostic packets differ between cycles");
  return samples[0]!.packetDigest;
}

const linkVue = (project: string, vueDir: string) => {
  const modules = join(project, "node_modules");
  mkdirSync(modules, { recursive: true });
  symlinkSync(vueDir, join(modules, "vue"), "dir");
  const namespace = join(dirname(vueDir), "@vue");
  if (existsSync(namespace)) symlinkSync(namespace, join(modules, "@vue"), "dir");
};

const cell = (role: Role, corpus: ScaleProject, samples: PairedSample[]): Cell => ({
  tool: "vize-check",
  label: `vize check (${role})`,
  vize: true,
  kind: "vue",
  sfcs: corpus.sfcs,
  plants: corpus.plants.length,
  mode: "-p",
  cold: samples.find((sample) => sample.phase === "cold") ?? null,
  runs: samples.filter((sample) => sample.phase === "timing"),
  refused: null,
});

const medianRatio = (pairs: { base: number; head: number }[]) =>
  median(pairs.map(({ base, head }) => head / base));

export function pairedRatios(samples: readonly PairedSample[]) {
  const cycles = [
    ...new Set(samples.filter((sample) => sample.phase === "timing").map((s) => s.cycle)),
  ];
  const pairs = cycles.map((cycle) => {
    const pick = (role: Role) =>
      samples.find((s) => s.phase === "timing" && s.cycle === cycle && s.role === role)!;
    return { base: pick("base"), head: pick("head") };
  });
  const values = (key: "wallMs" | "cpuMs" | "maxRssKb") =>
    pairs.map((pair) => {
      const base = pair.base[key],
        head = pair.head[key];
      assert.ok(base != null && head != null && base > 0 && head > 0, `missing ${key}`);
      return { base, head };
    });
  return {
    wall: medianRatio(values("wallMs")),
    cpu: medianRatio(values("cpuMs")),
    rss: medianRatio(values("maxRssKb")),
  };
}

type Options = {
  baseBin: string;
  headBin: string;
  baseSource: string;
  headSource: string;
  backend: string;
  vueDir: string;
  buildReceipts: string;
  sizes: number[];
  runs: number;
  work: string;
  out: string;
  timeoutMs: number;
  runner: string;
  runUrl: string | null;
};

export function resolvePairedProviders(providerRoot: string) {
  const root = resolve(providerRoot);
  const require = createRequire(join(root, "tests/package.json"));
  return {
    backend: realpathSync(join(root, "node_modules/@typescript/typescript-linux-x64/lib/tsc")),
    vueDir: dirname(realpathSync(require.resolve("vue/package.json"))),
  };
}

type SummaryReport = {
  sources: { base: string; head: string };
  runner: string;
  runs: number;
  corpora: {
    sfcs: number;
    packetDigest: string;
    paired: ReturnType<typeof pairedRatios>;
  }[];
  roles: Record<Role, { cells: Cell[]; budget: ReturnType<typeof scalingBudget> }>;
  qualified: boolean;
};

function renderSummary(report: SummaryReport): string {
  const lines = [
    "# Paired type-check scaling",
    "",
    `Exact sources \`${report.sources.base}\` and \`${report.sources.head}\` on \`${report.runner}\`. ${report.runs} same-runner paired cycles per size; even cycles run base/head and odd cycles head/base. Every invocation has an independent temp and XDG cache directory.`,
    "",
    "Complete diagnostic packets include every checked SFC and must have one digest across base, head, cold and all timed cycles. CPU is user+sys for the reaped process tree; RSS is GNU time `%M`.",
    "",
    "| SFCs | Role | Cold wall s | Cold CPU s | Cold RSS MiB | Median wall s | Median CPU s | Median RSS MiB | Packet SHA-256 |",
    "| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |",
  ];
  for (const corpus of report.corpora) {
    for (const role of ["base", "head"] as const) {
      const row = rowOf(report.roles[role].cells.find((entry) => entry.sfcs === corpus.sfcs)!);
      const cold = row.cell.cold!;
      const seconds = (value: number | null) => (value == null ? "n/a" : (value / 1000).toFixed(3));
      const mib = (value: number | null) => (value == null ? "n/a" : (value / 1024).toFixed(1));
      lines.push(
        `| ${corpus.sfcs} | ${role} | ${seconds(cold.wallMs)} | ${seconds(cold.cpuMs)} | ${mib(cold.maxRssKb)} | ${seconds(row.wallMs)} | ${seconds(row.cpuMs)} | ${mib(row.rssKb)} | \`${corpus.packetDigest}\` |`,
      );
    }
  }
  lines.push(
    "",
    "| SFCs | Median paired head/base wall | CPU | RSS |",
    "| ---: | ---: | ---: | ---: |",
    ...report.corpora.map(
      (corpus) =>
        `| ${corpus.sfcs} | ${corpus.paired.wall.toFixed(4)}x | ${corpus.paired.cpu.toFixed(4)}x | ${corpus.paired.rss.toFixed(4)}x |`,
    ),
    "",
    "| Role | Metric | Shape | Value | Ceiling | Result |",
    "| --- | --- | --- | ---: | ---: | --- |",
  );
  for (const role of ["base", "head"] as const) {
    for (const entry of report.roles[role].budget.checked) {
      lines.push(
        `| ${role} | ${entry.metric} | ${entry.shape} | ${entry.value === "infinite" ? "infinite" : `${entry.value.toFixed(4)}x`} | ${entry.ceiling.toFixed(2)}x | ${entry.ok ? "pass" : "fail"} |`,
      );
    }
  }
  lines.push("", `Budget: **${report.qualified ? "passed" : "failed"}**.`, "");
  return lines.join("\n");
}

export function collectPairedScaling(options: Options) {
  assert.ok(!existsSync(options.out), "refuse to overwrite paired evidence");
  assert.ok(relative(options.work, options.out).startsWith(".."), "output must be outside work");
  assert.deepEqual(
    options.sizes,
    [1000, 5000, 20000],
    "the declared budget requires its exact size vector",
  );
  assert.equal(options.runs, 5, "exactly five paired cycles are required");
  mkdirSync(options.work, { recursive: true });
  mkdirSync(options.out, { recursive: true });
  const binaries = {
    base: binaryEvidence(options.baseBin, join(options.out, "versions/base"), options.timeoutMs),
    head: binaryEvidence(options.headBin, join(options.out, "versions/head"), options.timeoutMs),
  };
  const bindings = verifyObservationBindings(
    options.buildReceipts,
    {
      base: options.baseSource,
      head: options.headSource,
    },
    binaries,
  );
  assert.ok(bindings.qualified, bindings.reason ?? "source build binding refused");
  const backend = binaryEvidence(
    options.backend,
    join(options.out, "versions/backend"),
    options.timeoutMs,
  );
  assert.equal(backend.versionStatus, 0, "backend version probe failed");
  const cells: Record<Role, Cell[]> = {
    base: [],
    head: [],
  };
  const corpora = [] as {
    sfcs: number;
    inventory: ReturnType<typeof corpusInventory>;
    packetDigest: string;
    paired: ReturnType<typeof pairedRatios>;
  }[];
  for (const sfcs of options.sizes) {
    const corpus = generateScaleProject({ sfcs, kind: "vue", parityPlants: true });
    const dir = join(options.work, `vue-${sfcs}`);
    writeProject(dir, corpus.files);
    linkVue(dir, options.vueDir);
    const before = corpusInventory(dir);
    const samples: PairedSample[] = [];
    const run = (role: Role, phase: "cold" | "timing", cycle: number, order: number) => {
      const isolated = mkdtempSync(join(options.work, `invoke-${sfcs}-${role}-`));
      const cache = join(isolated, "cache"),
        temp = join(isolated, "temp");
      mkdirSync(cache);
      mkdirSync(temp);
      try {
        const measured = measure(
          binaries[role].path,
          [
            "check",
            ".",
            "--format",
            "json",
            "--quiet",
            "--tsconfig",
            join(dir, "tsconfig.json"),
            "--corsa-path",
            backend.path,
            "--no-config",
          ],
          dir,
          cache,
          join(isolated, "gnu-time.txt"),
          options.timeoutMs,
          true,
          { TMPDIR: temp, TMP: temp, TEMP: temp, GOMAXPROCS: "8" },
        );
        const output = measured.output as ProcessOutput;
        assert.ok(
          output.status === 0 || output.status === 1,
          `${role} exited ${output.status}: ${output.stderr.slice(-2000)}`,
        );
        const packet = checkPacket(output.stdout);
        assertFileCoverage(packet, before.sfcFiles);
        const parsed = parseRun(output, dir, "json");
        const verdict = gate("vize-check", parsed.diagnostics, corpus.plants);
        assert.ok(verdict.ok, refusalText("vize-check", verdict));
        assert.ok(
          measured.cpuMs != null && measured.maxRssKb != null,
          "Linux CPU/RSS evidence required",
        );
        samples.push({
          role,
          phase,
          cycle,
          order,
          packetDigest: packetDigest(packet),
          fileCount: packet.fileCount,
          isolation: { cache: `${basename(isolated)}/cache`, temp: `${basename(isolated)}/temp` },
          wallMs: measured.wallMs,
          cpuMs: measured.cpuMs,
          maxRssKb: measured.maxRssKb,
          errors: verdict.errors,
          warnings: verdict.warnings,
          fingerprint: parsed.fingerprint,
          types: null,
          instantiations: null,
          checkTimeS: null,
        });
      } finally {
        rmSync(isolated, { recursive: true, force: true });
      }
    };
    pairedOrder(-1).forEach((role, order) => run(role, "cold", -1, order));
    for (let cycle = 0; cycle < options.runs; cycle++)
      pairedOrder(cycle).forEach((role, order) => run(role, "timing", cycle, order));
    const digest = assertPairedDigests(samples);
    assert.equal(corpusInventory(dir).sha256, before.sha256, "corpus changed during measurement");
    for (const role of ["base", "head"] as const)
      cells[role].push(
        cell(
          role,
          corpus,
          samples.filter((sample) => sample.role === role),
        ),
      );
    corpora.push({ sfcs, inventory: before, packetDigest: digest, paired: pairedRatios(samples) });
    writeJson(join(options.out, `samples-${sfcs}.json`), samples);
    rmSync(dir, { recursive: true, force: true });
  }
  const roles = {
    base: { cells: cells.base, budget: scalingBudget(cells.base) },
    head: { cells: cells.head, budget: scalingBudget(cells.head) },
  };
  const report = {
    schemaVersion: 1,
    sources: { base: options.baseSource, head: options.headSource },
    runner: options.runner,
    machine: `${availableParallelism()} vCPUs, ${(totalmem() / 2 ** 30).toFixed(0)} GiB`,
    generatedAt: new Date().toISOString(),
    runUrl: options.runUrl,
    runs: options.runs,
    nodeOptions: NODE_OPTIONS,
    binaries,
    backend,
    bindings,
    corpora,
    roles,
    unchangedBinaries:
      sha256(options.baseBin) === binaries.base.sha256 &&
      sha256(options.headBin) === binaries.head.sha256,
    qualified: roles.base.budget.status === "passed" && roles.head.budget.status === "passed",
  };
  assert.ok(report.unchangedBinaries, "measured binary changed");
  writeJson(join(options.out, "report.json"), report);
  writeFileSync(join(options.out, "summary.md"), renderSummary(report));
  return report;
}

function main() {
  const { values } = parseArgs({
    options: Object.fromEntries(
      [
        "base-bin",
        "head-bin",
        "base-source",
        "head-source",
        "provider-root",
        "build-receipts",
        "sizes",
        "runs",
        "work",
        "out",
        "timeout-ms",
        "runner",
        "run-url",
      ].map((name) => [name, { type: "string" as const }]),
    ),
  });
  const required = (name: string) => {
    const value = values[name];
    assert.ok(value, `missing --${name}`);
    return value as string;
  };
  const sources = [required("base-source"), required("head-source")];
  sources.forEach((source) => assert.match(source, /^[0-9a-f]{40}$/u));
  const providers = resolvePairedProviders(required("provider-root"));
  const report = collectPairedScaling({
    baseBin: resolve(required("base-bin")),
    headBin: resolve(required("head-bin")),
    baseSource: sources[0]!,
    headSource: sources[1]!,
    backend: providers.backend,
    vueDir: providers.vueDir,
    buildReceipts: resolve(required("build-receipts")),
    sizes: required("sizes").split(",").map(Number),
    runs: Number(required("runs")),
    work: resolve(required("work")),
    out: resolve(required("out")),
    timeoutMs: Number(values["timeout-ms"] ?? 2_700_000),
    runner: values.runner ?? "local",
    runUrl: values["run-url"] ?? null,
  });
  if (!report.qualified) {
    console.error(
      formatGrowthBudgetFailures([
        ...report.roles.base.budget.failures,
        ...report.roles.head.budget.failures,
      ]),
    );
    process.exitCode = 1;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
