#!/usr/bin/env node
/**
 * Compile-only benchmark across native builds (published or source-built).
 *
 * Every (variant, mode) runs in a fresh child process so load cost, warm wall
 * time, CPU-seconds (user+sys, all threads) and peak RSS are measured apart.
 * Each child also hashes every emitted `code`, so two builds can be shown to
 * produce byte-identical output for the corpus.
 *
 *   node compile-history.ts --input DIR --native label=/path/a.node ... \
 *     [--vue /path/to/@vue/compiler-sfc] [--runs 5] [--warmups 2] [--rounds 2] \
 *     [--modes loop1t,batch1t,batchmax] [--out summary.md] [--json results.json]
 */

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import { join, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";

import { isRecord } from "./script-types.ts";

type Mode = "vue1t" | "loop1t" | "batch1t" | "batchmax";
type Spec = { label: string; kind: "vue" | "native"; path: string; mode: Mode };
type Sample = { wallMs: number; cpuMs: number; nativeMs?: number };
type Pass = [codes: string[], failed: number, nativeMs?: number];
type ChildResult = {
  load: Sample;
  runs: Sample[];
  maxRssMb: number;
  files: number;
  failed: number;
  outputBytes: number;
  outputSha256: string;
  /** Hashes over files whose source has no TypeScript script, and over the rest. */
  outputSha256Js: string;
  outputSha256Ts: string;
};
type SourceFile = { filename: string; source: string };

const require = createRequire(import.meta.url);
const selfPath = fileURLToPath(import.meta.url);

function cpuMsSince(start: NodeJS.CpuUsage): number {
  const delta = process.cpuUsage(start);
  return (delta.user + delta.system) / 1000;
}

function readCorpus(input: string): SourceFile[] {
  return readdirSync(input)
    .filter((name) => name.endsWith(".vue"))
    .sort()
    .map((filename) => ({ filename, source: readFileSync(join(input, filename), "utf8") }));
}

function field(value: unknown, key: string): unknown {
  return isRecord(value) ? value[key] : undefined;
}

function nativeFn(binding: unknown, name: string): (...args: unknown[]) => unknown {
  const fn = field(binding, name);
  if (typeof fn !== "function") throw new Error(`native binding has no ${name}`);
  return (...args: unknown[]) => Reflect.apply(fn, binding, args);
}

/** One measured pass; returns per-file code strings and the failure count. */
function makePass(spec: Spec, module: unknown, files: SourceFile[]): () => Pass {
  if (spec.kind === "vue") {
    const parse = nativeFn(module, "parse");
    const compileScript = nativeFn(module, "compileScript");
    const compileTemplate = nativeFn(module, "compileTemplate");
    return () => {
      const codes: string[] = [];
      for (const { filename, source } of files) {
        const descriptor = field(parse(source, { filename }), "descriptor");
        let bindings: unknown = {};
        let script = "";
        if (field(descriptor, "scriptSetup") || field(descriptor, "script")) {
          const result = compileScript(descriptor, { id: filename });
          bindings = field(result, "bindings") ?? {};
          script = String(field(result, "content"));
        }
        let template = "";
        const block = field(descriptor, "template");
        if (block) {
          const result = compileTemplate({
            source: field(block, "content"),
            filename,
            id: filename,
            compilerOptions: { bindingMetadata: bindings },
          });
          template = String(field(result, "code"));
        }
        codes.push(`${script}\n${template}`);
      }
      return [codes, 0];
    };
  }
  if (spec.mode === "loop1t") {
    const compileSfc = nativeFn(module, "compileSfc");
    const extra: unknown = JSON.parse(process.env.COMPILE_BENCH_SFC_OPTIONS ?? "{}");
    return () => {
      const codes: string[] = [];
      let failed = 0;
      for (const { filename, source } of files) {
        const result = compileSfc(source, { ...(isRecord(extra) ? extra : {}), filename });
        const errors = field(result, "errors");
        if (Array.isArray(errors) && errors.length > 0) failed++;
        codes.push(String(field(result, "code")));
      }
      return [codes, failed];
    };
  }
  const batch = nativeFn(module, "compileSfcBatchWithResults");
  const inputs = files.map(({ filename, source }) => ({ path: filename, source }));
  const options = spec.mode === "batch1t" ? { threads: 1 } : {};
  return () => {
    const result = batch(inputs, options);
    const results = field(result, "results");
    if (!Array.isArray(results)) throw new Error("batch returned no results");
    return [
      results.map((entry: unknown) => String(field(entry, "code"))),
      Number(field(result, "failedCount")),
      Number(field(result, "timeMs")),
    ];
  };
}

function runChild(spec: Spec, input: string, runs: number, warmups: number): ChildResult {
  const files = readCorpus(input);
  const loadStart = performance.now();
  const loadCpu = process.cpuUsage();
  const module: unknown = require(spec.path);
  const load = { wallMs: performance.now() - loadStart, cpuMs: cpuMsSince(loadCpu) };
  const pass = makePass(spec, module, files);
  for (let i = 0; i < warmups; i++) pass();
  const samples: Sample[] = [];
  let last: Pass = [[], 0];
  for (let i = 0; i < runs; i++) {
    const start = performance.now();
    const cpu = process.cpuUsage();
    last = pass();
    samples.push({ wallMs: performance.now() - start, cpuMs: cpuMsSince(cpu), nativeMs: last[2] });
  }
  const [codes, failed] = last;
  const hash = createHash("sha256");
  const byLang = { js: createHash("sha256"), ts: createHash("sha256") };
  const tsScript = /<script\b[^>]*\blang=["']tsx?["']/;
  let outputBytes = 0;
  codes.forEach((code, index) => {
    hash.update(code);
    hash.update("\0");
    const lang = tsScript.test(files[index]?.source ?? "") ? byLang.ts : byLang.js;
    lang.update(code);
    lang.update("\0");
    outputBytes += Buffer.byteLength(code);
  });
  return {
    load,
    runs: samples,
    maxRssMb: process.resourceUsage().maxRSS / 1024,
    files: codes.length,
    failed,
    outputBytes,
    outputSha256: hash.digest("hex"),
    outputSha256Js: byLang.js.digest("hex"),
    outputSha256Ts: byLang.ts.digest("hex"),
  };
}

function parseArgs(argv: string[]): Map<string, string[]> {
  const args = new Map<string, string[]>();
  for (let i = 0; i < argv.length; i++) {
    const key = argv[i];
    if (!key?.startsWith("--")) continue;
    const next = argv[i + 1];
    const value = next === undefined || next.startsWith("--") ? "true" : (i++, next);
    args.set(key.slice(2), [...(args.get(key.slice(2)) ?? []), value]);
  }
  return args;
}

function median(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? (sorted[mid] ?? NaN)
    : ((sorted[mid - 1] ?? NaN) + (sorted[mid] ?? NaN)) / 2;
}

function isChildResult(value: unknown): value is ChildResult {
  return isRecord(value) && Array.isArray(value.runs) && typeof value.outputSha256 === "string";
}

function spawnChild(spec: Spec, input: string, runs: number, warmups: number): ChildResult {
  const child = spawnSync(
    process.execPath,
    [
      selfPath,
      "--child",
      JSON.stringify(spec),
      "--input",
      input,
      "--runs",
      `${runs}`,
      "--warmups",
      `${warmups}`,
    ],
    { encoding: "utf8", maxBuffer: 64 * 1024 * 1024, stdio: ["ignore", "pipe", "inherit"] },
  );
  if (child.status !== 0) throw new Error(`${spec.label} ${spec.mode} exited with ${child.status}`);
  const parsed: unknown = JSON.parse(child.stdout);
  if (!isChildResult(parsed)) throw new Error(`${spec.label} ${spec.mode} printed no result`);
  return parsed;
}

const fmt = (value: number, digits = 1) => value.toFixed(digits);

function main(): void {
  const args = parseArgs(process.argv.slice(2));
  const one = (key: string, fallback?: string) => args.get(key)?.at(-1) ?? fallback;
  const input = one("input");
  if (!input) throw new Error("--input is required");
  const runs = Number(one("runs", "5"));
  const warmups = Number(one("warmups", "2"));
  const childSpec = one("child");
  if (childSpec) {
    const parsed: unknown = JSON.parse(childSpec);
    if (!isRecord(parsed)) throw new Error("invalid child spec");
    const spec: Spec = {
      label: String(parsed.label),
      kind: parsed.kind === "vue" ? "vue" : "native",
      path: String(parsed.path),
      mode:
        (["vue1t", "loop1t", "batch1t", "batchmax"] as const).find((m) => m === parsed.mode) ??
        "loop1t",
    };
    process.stdout.write(JSON.stringify(runChild(spec, input, runs, warmups)));
    return;
  }

  const rounds = Number(one("rounds", "2"));
  const modes = (one("modes", "loop1t,batch1t,batchmax") ?? "").split(",").filter(Boolean);
  const specs: Spec[] = [];
  const vue = one("vue");
  if (vue)
    specs.push({ label: "@vue/compiler-sfc", kind: "vue", path: resolve(vue), mode: "vue1t" });
  for (const entry of args.get("native") ?? []) {
    const at = entry.indexOf("=");
    const label = entry.slice(0, at);
    const path = resolve(entry.slice(at + 1));
    for (const mode of modes) {
      if (mode === "loop1t" || mode === "batch1t" || mode === "batchmax")
        specs.push({ label, kind: "native", path, mode });
    }
  }

  const collected = new Map<Spec, ChildResult[]>(specs.map((spec) => [spec, []]));
  for (let round = 0; round < rounds; round++) {
    const ordered = round % 2 === 0 ? specs : [...specs].reverse();
    for (const spec of ordered) {
      const result = spawnChild(spec, input, runs, warmups);
      collected.get(spec)?.push(result);
      const last = result.runs.at(-1);
      console.error(
        `${spec.label} ${spec.mode}: ${fmt(last?.wallMs ?? NaN)} ms wall, ${fmt(last?.cpuMs ?? NaN)} ms cpu`,
      );
    }
  }

  const rows = specs.map((spec) => {
    const results = collected.get(spec) ?? [];
    const samples = results.flatMap((result) => result.runs);
    const first = results[0];
    return {
      label: spec.label,
      mode: spec.mode,
      path: spec.path,
      wallMs: median(samples.map((sample) => sample.wallMs)),
      cpuMs: median(samples.map((sample) => sample.cpuMs)),
      nativeMs: median(samples.map((sample) => sample.nativeMs ?? NaN)),
      loadWallMs: median(results.map((result) => result.load.wallMs)),
      loadCpuMs: median(results.map((result) => result.load.cpuMs)),
      maxRssMb: Math.max(...results.map((result) => result.maxRssMb)),
      files: first?.files ?? 0,
      failed: first?.failed ?? 0,
      outputBytes: first?.outputBytes ?? 0,
      outputSha256: first?.outputSha256 ?? "",
      outputSha256Js: first?.outputSha256Js ?? "",
      outputSha256Ts: first?.outputSha256Ts ?? "",
      stableOutput: results.every((result) => result.outputSha256 === first?.outputSha256),
      samples,
    };
  });
  const vueRow = rows.find((row) => row.mode === "vue1t");
  const lines = [
    "## Compile benchmark",
    "",
    `Runner: ${os.cpus()[0]?.model ?? "unknown"} x${os.availableParallelism()}, ${process.version}. ` +
      `Corpus: ${rows[0]?.files ?? 0} files from \`${input}\`. ${rounds} rounds x ${runs} runs (+${warmups} warmups), fresh process per round.`,
    "",
    "Wall and CPU are medians per pass. CPU = user+sys over all threads. Ratios divide @vue/compiler-sfc (1T) by the row. In native = the batch's own `timeMs` (excludes JS<->native conversion).",
    "",
    "| Build | Mode | Wall | In native | CPU | CPU/Wall | Wall ratio | CPU ratio | Load wall | Load CPU | Peak RSS | Failed | Output bytes | Output SHA-256 | JS-source / TS-source SHA-256 |",
    "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |",
    ...rows.map((row) =>
      [
        row.label,
        row.mode,
        `${fmt(row.wallMs)} ms`,
        Number.isNaN(row.nativeMs) ? "" : `${fmt(row.nativeMs)} ms`,
        `${fmt(row.cpuMs)} ms`,
        fmt(row.cpuMs / row.wallMs, 2),
        vueRow ? `${fmt(vueRow.wallMs / row.wallMs)}x` : "n/a",
        vueRow ? `${fmt(vueRow.cpuMs / row.cpuMs)}x` : "n/a",
        `${fmt(row.loadWallMs)} ms`,
        `${fmt(row.loadCpuMs)} ms`,
        `${fmt(row.maxRssMb, 0)} MB`,
        `${row.failed}`,
        `${row.outputBytes}`,
        `\`${row.outputSha256.slice(0, 16)}\`${row.stableOutput ? "" : " (unstable)"}`,
        `\`${row.outputSha256Js.slice(0, 12)}\` / \`${row.outputSha256Ts.slice(0, 12)}\``,
      ]
        .join(" | ")
        .replace(/^/, "| ")
        .concat(" |"),
    ),
    "",
  ];
  const out = one("out");
  if (out) writeFileSync(out, `${lines.join("\n")}\n`);
  const json = one("json");
  if (json)
    writeFileSync(
      json,
      `${JSON.stringify({ input, runs, warmups, rounds, node: process.version, cpus: os.availableParallelism(), rows }, null, 2)}\n`,
    );
  console.log(lines.join("\n"));
}

main();
