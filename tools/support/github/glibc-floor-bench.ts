/**
 * Counts the instructions each Linux release linking recipe executes, under
 * callgrind (deterministic, unlike wall time on a shared runner), and records
 * binary and gzip sizes.
 *
 * For the CLI it also counts the instructions before `main`: the dynamic
 * loader's (or a static-PIE's own) relocation processing and initializers,
 * which dominate single-file editor and lint-staged runs. That is the full
 * run minus a second run collected only inside `main`.
 *
 * Usage:
 *   node tools/support/github/glibc-floor-bench.ts measure --out <json> --runs <n>
 *     --file <one.vue> --corpus <dir> [--corsa <tsgo>]
 *     --cli <variant>=<vize> ... --binding <variant>=<.node> ...
 *   node tools/support/github/glibc-floor-bench.ts summarize <json>
 *
 * The first `--cli` and the first `--binding` variant are the baselines (the
 * recipe shipping today); every other variant is reported against them.
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import zlib from "node:zlib";

import { isEntrypoint, runEntrypoint } from "../command-runtime.ts";

export interface Measurement {
  readonly kind: "cli" | "binding";
  readonly scenario: string;
  readonly variant: string;
  /** Total instructions (callgrind Ir) per run. */
  readonly instructions: readonly number[];
  /** Instructions before `main` per run (CLI only). */
  readonly beforeMain?: readonly number[];
}

export interface Size {
  readonly kind: "cli" | "binding";
  readonly variant: string;
  readonly bytes: number;
  readonly gzipBytes: number;
}

interface Options {
  out: string;
  runs: number;
  file: string;
  corpus: string;
  corsa: string | undefined;
  cli: [string, string][];
  binding: [string, string][];
}

function parseMeasure(args: readonly string[]): Options {
  const options: Options = {
    out: "",
    runs: 0,
    file: "",
    corpus: "",
    corsa: undefined,
    cli: [],
    binding: [],
  };
  for (let index = 0; index < args.length; index += 2) {
    const [flag, value] = [args[index], args[index + 1]];
    if (value === undefined) throw new Error(`${flag} needs a value`);
    if (flag === "--out") options.out = value;
    else if (flag === "--runs") options.runs = Number(value);
    else if (flag === "--file") options.file = path.resolve(value);
    else if (flag === "--corpus") options.corpus = path.resolve(value);
    else if (flag === "--corsa") options.corsa = path.resolve(value);
    else if (flag === "--cli" || flag === "--binding") {
      const [variant, file] = value.split("=", 2);
      if (!variant || !file) throw new Error(`${flag} expects <variant>=<path>`);
      (flag === "--cli" ? options.cli : options.binding).push([variant, path.resolve(file)]);
    } else throw new Error(`Unknown argument: ${flag}`);
  }
  if (!Number.isInteger(options.runs) || options.runs < 1) throw new Error("--runs must be >= 1");
  if (!options.out || !options.file || !options.corpus) {
    throw new Error("--out, --file and --corpus are required");
  }
  return options;
}

/** Total instructions from a callgrind output file's `totals:` or `summary:` line. */
export function callgrindTotal(text: string): number {
  const total = /^(?:totals|summary): (\d+)$/mu.exec(text);
  if (total === null) throw new Error("callgrind output has no totals");
  return Number(total[1]);
}

function callgrind(
  command: string,
  args: readonly string[],
  scratch: string,
  env: NodeJS.ProcessEnv,
  extra: readonly string[] = [],
): number {
  const out = path.join(scratch, `callgrind.${process.hrtime.bigint()}.out`);
  const run = spawnSync(
    "valgrind",
    ["--tool=callgrind", `--callgrind-out-file=${out}`, ...extra, command, ...args],
    { cwd: scratch, env, encoding: "utf8", maxBuffer: 1 << 30 },
  );
  if (run.error) throw run.error;
  // `vize lint` and `vize check` exit 1 when they report problems.
  if (run.status === null || run.status > 1) {
    throw new Error(
      `${command} ${args.join(" ")} failed (${run.status})\n${run.stderr.slice(-4000)}`,
    );
  }
  const total = callgrindTotal(fs.readFileSync(out, "utf8"));
  fs.rmSync(out, { force: true });
  return total;
}

const bindingScript = (compile: boolean) =>
  compile
    ? `const native = require(process.argv[1]);
const file = process.argv[2];
const result = native.compileSfc(require("node:fs").readFileSync(file, "utf8"), { filename: file });
if (!result.code) throw new Error("compileSfc produced no code");`
    : "require(process.argv[1]);";

function size(kind: Size["kind"], variant: string, file: string): Size {
  const bytes = fs.readFileSync(file);
  return { kind, variant, bytes: bytes.length, gzipBytes: zlib.gzipSync(bytes).length };
}

export function measure(args: readonly string[]): number {
  const options = parseMeasure(args);
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "glibc-floor-bench-"));
  const env = { ...process.env, ...(options.corsa ? { CORSA_PATH: options.corsa } : {}) };
  const measurements: Measurement[] = [];
  const record = (
    kind: Measurement["kind"],
    scenario: string,
    variant: string,
    command: string,
    commandArgs: readonly string[],
    splitMain: boolean,
  ) => {
    const instructions: number[] = [];
    const beforeMain: number[] = [];
    for (let run = 0; run < options.runs; run += 1) {
      const total = callgrind(command, commandArgs, scratch, env);
      instructions.push(total);
      if (splitMain) {
        const inMain = callgrind(command, commandArgs, scratch, env, [
          "--collect-atstart=no",
          "--toggle-collect=main",
        ]);
        beforeMain.push(total - inMain);
      }
    }
    measurements.push({
      kind,
      scenario,
      variant,
      instructions,
      ...(splitMain ? { beforeMain } : {}),
    });
    const before = splitMain ? ` (before main ${beforeMain.join(", ")})` : "";
    console.log(`${scenario} ${variant}: ${instructions.join(", ")}${before}`);
  };
  const lintArgs = ["--no-config", "--format", "json", "--quiet"];
  for (const [variant, cli] of options.cli) {
    record("cli", "vize --version", variant, cli, ["--version"], true);
    record("cli", "vize lint <one file>", variant, cli, ["lint", options.file, ...lintArgs], true);
    if (options.corsa) {
      const check = ["check", options.file, ...lintArgs];
      record("cli", "vize check <one file>", variant, cli, check, true);
    }
    const corpus = ["lint", options.corpus, ...lintArgs];
    record("cli", "vize lint <corpus>", variant, cli, corpus, false);
  }
  if (options.binding.length > 0) {
    record("binding", "node -e 0 (no binding)", "-", process.execPath, ["-e", "0"], false);
  }
  for (const [variant, binding] of options.binding) {
    const load = ["-e", bindingScript(false), binding];
    record("binding", "require(binding)", variant, process.execPath, load, false);
    const compile = ["-e", bindingScript(true), binding, options.file];
    record("binding", "require + compileSfc <one file>", variant, process.execPath, compile, false);
  }
  const sizes = [
    ...options.cli.map(([variant, file]) => size("cli", variant, file)),
    ...options.binding.map(([variant, file]) => size("binding", variant, file)),
  ];
  fs.mkdirSync(path.dirname(path.resolve(options.out)), { recursive: true });
  fs.writeFileSync(options.out, `${JSON.stringify({ measurements, sizes }, null, 2)}\n`);
  fs.rmSync(scratch, { recursive: true, force: true });
  return 0;
}

function median(values: readonly number[]): number {
  const sorted = [...values].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[middle]! : (sorted[middle - 1]! + sorted[middle]!) / 2;
}

const count = (value: number) => Math.round(value).toLocaleString("en-US");
const delta = (value: number, base: number) =>
  base > 0 ? `${value >= base ? "+" : ""}${((value / base - 1) * 100).toFixed(2)}%` : "";

/** Markdown: instruction medians per scenario and variant, then sizes, against each baseline. */
export function summarize(measurements: readonly Measurement[], sizes: readonly Size[]): string {
  const lines = [
    "| Scenario | Variant | Instructions | vs baseline | Before `main` | vs baseline |",
    "| --- | --- | --- | --- | --- | --- |",
  ];
  for (const scenario of new Set(measurements.map((entry) => entry.scenario))) {
    const rows = measurements.filter((entry) => entry.scenario === scenario);
    const base = median(rows[0]!.instructions);
    const baseBefore = rows[0]!.beforeMain ? median(rows[0]!.beforeMain) : 0;
    for (const row of rows) {
      const first = row === rows[0];
      const value = median(row.instructions);
      const before = row.beforeMain ? median(row.beforeMain) : undefined;
      lines.push(
        `| ${scenario} | ${row.variant} | ${count(value)} | ${first ? "baseline" : delta(value, base)} | ${before === undefined ? "" : count(before)} | ${before === undefined || first ? "" : delta(before, baseBefore)} |`,
      );
    }
  }
  lines.push("", "| Artifact | Variant | Bytes | vs baseline | gzip bytes | vs baseline |");
  lines.push("| --- | --- | --- | --- | --- | --- |");
  for (const kind of ["cli", "binding"] as const) {
    const rows = sizes.filter((entry) => entry.kind === kind);
    for (const row of rows) {
      const first = row === rows[0];
      lines.push(
        `| ${kind} | ${row.variant} | ${count(row.bytes)} | ${first ? "baseline" : delta(row.bytes, rows[0]!.bytes)} | ${count(row.gzipBytes)} | ${first ? "baseline" : delta(row.gzipBytes, rows[0]!.gzipBytes)} |`,
      );
    }
  }
  return `${lines.join("\n")}\n`;
}

export function main(args: readonly string[]): number {
  const [command, ...rest] = args;
  if (command === "measure") return measure(rest);
  if (command === "summarize" && rest.length === 1) {
    const { measurements, sizes } = JSON.parse(fs.readFileSync(rest[0]!, "utf8")) as {
      measurements: Measurement[];
      sizes: Size[];
    };
    process.stdout.write(summarize(measurements, sizes));
    return 0;
  }
  throw new Error(
    "Usage: node tools/support/github/glibc-floor-bench.ts <measure ... | summarize <json>>",
  );
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(process.argv.slice(2)));
