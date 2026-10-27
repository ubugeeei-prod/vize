#!/usr/bin/env node
/** Advisory observations for every PR. No performance threshold fails this collector. */
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { parseArgs } from "node:util";
import { pathToFileURL } from "node:url";
import { makeTasks, laneThresholdPercent } from "./compare-pr-lanes.ts";
import { summarizeBenchmarkRuns } from "./compare-pr-results.ts";
import { writeObservationReport } from "./pr-performance-observation-report.ts";
import { verifyObservationBindings } from "./pr-performance-observation-binding.ts";
import { qualifyCheck } from "./pr-performance-observation-check.ts";
import {
  binaryEvidence,
  corpusInventory,
  invocationSucceeded,
  observationEnvironment,
  runObservation,
  sha256,
  writeJson,
} from "./pr-performance-observation-evidence.ts";

export function advisoryComparison(
  baseRuns: number[],
  headRuns: number[],
  reason: string | null,
  id: string,
  label: string,
  thresholdPercent: number,
) {
  if (
    reason ||
    baseRuns.length === 0 ||
    baseRuns.length !== headRuns.length ||
    [...baseRuns, ...headRuns].some((value) => !Number.isFinite(value) || value <= 0)
  )
    return {
      availability: "unavailable",
      reason: reason ?? "missing/invalid paired samples",
      result: null,
    };
  return {
    availability: "observed",
    reason: null,
    result: summarizeBenchmarkRuns({ id, label, baseRuns, headRuns, thresholdPercent }),
  };
}

export type Options = {
  baseBin: string;
  headBin: string;
  baseSource: string;
  headSource: string;
  inputDir: string;
  out: string;
  runs: number;
  warmups: number;
  thresholdPercent: number;
  timeoutMs: number;
  tasks: string;
  backend: string | null;
  vueDir: string | null;
  buildReceipts: string | null;
};
export type Lane = {
  id: string;
  label: string;
  args: string[];
  threads: number | "max";
  thresholdHintPercent: number;
  reason: string | null;
  correctness: unknown;
  baseRuns: number[];
  headRuns: number[];
  raw: unknown[];
  rss: { base: number | null; head: number | null; ratio: number | null };
  comparison: ReturnType<typeof advisoryComparison>;
};

export function collectObservation(options: Options) {
  const { out, inputDir } = options;
  assert.ok(!existsSync(out), "refuse to overwrite observation evidence");
  assert.ok(relative(inputDir, out).startsWith(".."), "output must be outside the corpus");
  assert.ok(options.runs > 0 && Number.isSafeInteger(options.runs));
  assert.ok(options.warmups >= 0 && Number.isSafeInteger(options.warmups));
  assert.ok(options.timeoutMs > 0 && Number.isSafeInteger(options.timeoutMs));
  assert.ok(options.thresholdPercent > 0 && Number.isFinite(options.thresholdPercent));
  for (const source of [options.baseSource, options.headSource])
    assert.match(source, /^[a-f0-9]{40}$/u);
  mkdirSync(out, { recursive: true });
  const corpus = corpusInventory(inputDir);
  assert.ok(corpus.sfcFiles.length > 0, "empty SFC corpus");
  writeJson(join(out, "corpus.json"), corpus);
  const binaries = {
    base: binaryEvidence(options.baseBin, join(out, "versions", "base"), options.timeoutMs),
    head: binaryEvidence(options.headBin, join(out, "versions", "head"), options.timeoutMs),
  };
  const bindings = verifyObservationBindings(
    options.buildReceipts,
    { base: options.baseSource, head: options.headSource },
    binaries,
  );
  writeJson(join(out, "binding.json"), bindings);
  const backend = options.backend
    ? binaryEvidence(options.backend, join(out, "versions", "backend"), options.timeoutMs)
    : null;
  const vue =
    options.vueDir && existsSync(join(options.vueDir, "package.json"))
      ? {
          path: options.vueDir,
          packageJsonSha256: sha256(join(options.vueDir, "package.json")),
          version: JSON.parse(readFileSync(join(options.vueDir, "package.json"), "utf8")).version,
        }
      : null;
  const tasks = makeTasks(inputDir, options.tasks);
  assert.ok(tasks.length > 0, "no enabled lanes");
  const results: Lane[] = [];
  for (const task of tasks) {
    const isCheck = task.id.startsWith("check"),
      threads = task.threads ?? 1;
    const args = [
      ...task.args,
      ...(isCheck
        ? ["--no-config", ...(options.backend ? ["--corsa-path", options.backend] : [])]
        : []),
    ];
    const env = observationEnvironment(threads);
    const row: Lane = {
      id: task.id,
      label: task.label,
      args,
      threads,
      thresholdHintPercent: laneThresholdPercent(task, options.thresholdPercent),
      reason: null,
      correctness: null,
      baseRuns: [],
      headRuns: [],
      raw: [],
      rss: { base: null, head: null, ratio: null },
      comparison: advisoryComparison(
        [],
        [],
        "not measured",
        task.id,
        task.label,
        options.thresholdPercent,
      ),
    };
    if (binaries.base.versionStatus !== 0 || binaries.head.versionStatus !== 0)
      row.reason = "binary version probe failed";
    if (!bindings.qualified) row.reason = `source build identity unqualified: ${bindings.reason}`;
    if (isCheck) {
      if (!backend || backend.versionStatus !== 0) row.reason = "explicit backend unavailable";
      const controls = Object.fromEntries(
        (["base", "head"] as const).map((side) => [
          side,
          qualifyCheck({
            binary: binaries[side].path,
            backend: options.backend,
            vueDir: options.vueDir,
            inputDir,
            out: join(out, "controls", task.id, side),
            expectedFiles: corpus.sfcFiles,
            threads,
            args,
            env,
            timeoutMs: options.timeoutMs,
          }),
        ]),
      );
      row.correctness = controls;
      if (!controls.base.qualified || !controls.head.qualified)
        row.reason = "check coverage/invalid/repair witness did not qualify";
      else if (JSON.stringify(controls.base.baseline) !== JSON.stringify(controls.head.baseline))
        row.reason = "base/head full diagnostic packets differ; no performance ranking";
    } else
      row.correctness = {
        scope: "exit contract and unchanged corpus only; no complete semantic oracle",
      };
    const statuses: Record<string, number[]> = { base: [], head: [] };
    const run = (side: "base" | "head", phase: string, index: number, memory = false) => {
      const invocation = runObservation({
        executable: binaries[side].path,
        args,
        cwd: inputDir,
        env,
        rawDir: join(out, "raw", task.id, `${phase}-${index}-${side}`),
        timeoutMs: options.timeoutMs,
        memory,
      });
      row.raw.push({ side, phase, index, ...invocation.receipt });
      if (
        !invocationSucceeded(invocation, task.allowNonZeroExit) &&
        invocation.receipt.error !== "GNU time unavailable"
      )
        row.reason = `${side} ${phase} failed or timed out`;
      if (invocation.receipt.status !== null && phase === "timing")
        statuses[side].push(invocation.receipt.status);
      if (memory) row.rss[side] = invocation.receipt.maxRssBytes;
      return invocation.receipt.wallMs;
    };
    // A failed independent witness must not produce an apparent timing win.
    if (!row.reason) {
      for (let i = 0; i < options.warmups; i++) {
        run("base", "warmup", i);
        run("head", "warmup", i);
      }
      for (let i = 0; i < options.runs && !row.reason; i++) {
        for (const side of i % 2 ? (["head", "base"] as const) : (["base", "head"] as const)) {
          const ms = run(side, "timing", i);
          if (ms !== null) row[side === "base" ? "baseRuns" : "headRuns"].push(ms);
        }
      }
      if (JSON.stringify(statuses.base) !== JSON.stringify(statuses.head))
        row.reason = "base/head exit contracts differ";
      if (!row.reason) {
        run("base", "rss", 0, true);
        run("head", "rss", 0, true);
      }
    }
    results.push(row);
  }
  const after = corpusInventory(inputDir);
  const unchanged =
    after.sha256 === corpus.sha256 &&
    sha256(options.baseBin) === binaries.base.sha256 &&
    sha256(options.headBin) === binaries.head.sha256 &&
    (!backend || sha256(backend.path) === backend.sha256);
  for (const row of results) {
    if (!unchanged) row.reason = "input or measured binary changed during collection";
    row.comparison = advisoryComparison(
      row.baseRuns,
      row.headRuns,
      row.reason,
      row.id,
      row.label,
      row.thresholdHintPercent,
    );
    row.rss.ratio =
      !row.reason && row.rss.base && row.rss.head ? row.rss.head / row.rss.base : null;
  }
  return writeObservationReport({
    options,
    corpus,
    after,
    unchanged,
    binaries,
    backend,
    vue,
    results,
  });
}

export function main(argv = process.argv.slice(2)) {
  const { values } = parseArgs({
    args: argv,
    options: Object.fromEntries(
      [
        "base-bin",
        "head-bin",
        "base-source",
        "head-source",
        "input-dir",
        "out",
        "runs",
        "warmups",
        "threshold-percent",
        "timeout-ms",
        "tasks",
        "corsa-path",
        "vue-dir",
        "build-receipts",
      ].map((name) => [name, { type: "string" as const }]),
    ),
  });
  const required = (name: string) => {
    const value = values[name];
    assert.ok(value, `missing --${name}`);
    return value as string;
  };
  const optionalPath = (name: string) => (values[name] ? resolve(values[name] as string) : null);
  const report = collectObservation({
    baseBin: resolve(required("base-bin")),
    headBin: resolve(required("head-bin")),
    baseSource: required("base-source"),
    headSource: required("head-source"),
    inputDir: resolve(required("input-dir")),
    out: resolve(required("out")),
    runs: Number(values.runs ?? 5),
    warmups: Number(values.warmups ?? 1),
    thresholdPercent: Number(values["threshold-percent"] ?? 5),
    timeoutMs: Number(values["timeout-ms"] ?? 120000),
    tasks: (values.tasks as string) ?? "",
    backend: optionalPath("corsa-path"),
    vueDir: optionalPath("vue-dir"),
    buildReceipts: optionalPath("build-receipts"),
  });
  console.log(
    JSON.stringify({
      advisory: true,
      observed: report.results.filter((row) => row.comparison.availability === "observed").length,
      unavailable: report.results.filter((row) => row.comparison.availability === "unavailable")
        .length,
      out: required("out"),
    }),
  );
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main();
  } catch (error) {
    console.error(String(error));
    process.exitCode = 1;
  }
}
