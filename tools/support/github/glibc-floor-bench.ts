import { fileURLToPath } from "node:url";
/**
 * Compares the Linux GNU release linking recipes on one runner: the same
 * source built against the runner's glibc (`host`, the recipe before the
 * glibc floor) and through `cargo zigbuild` at the documented floor
 * (`floor`). Rounds visit both variants in alternating order so drift on the
 * runner lands on both sides.
 *
 * Usage:
 *   node tools/support/github/glibc-floor-bench.ts run --rounds <n> --out <json>
 *     --corpus <dir> --cli host=<vize> --cli floor=<vize>
 *     --binding host=<.node> --binding floor=<.node>
 *   node tools/support/github/glibc-floor-bench.ts summarize <json>
 */
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { isEntrypoint, runEntrypoint } from "../command-runtime.ts";

export interface Sample {
  readonly scenario: string;
  readonly variant: string;
  readonly round: number;
  readonly seconds: number;
}

interface RunOptions {
  rounds: number;
  out: string;
  corpus: string;
  cli: Map<string, string>;
  binding: Map<string, string>;
}

function parseRun(args: readonly string[]): RunOptions {
  const options: RunOptions = {
    rounds: 0,
    out: "",
    corpus: "",
    cli: new Map(),
    binding: new Map(),
  };
  for (let index = 0; index < args.length; index += 2) {
    const [flag, value] = [args[index], args[index + 1]];
    if (value === undefined) throw new Error(`${flag} needs a value`);
    if (flag === "--rounds") options.rounds = Number(value);
    else if (flag === "--out") options.out = value;
    else if (flag === "--corpus") options.corpus = value;
    else if (flag === "--cli" || flag === "--binding") {
      const [variant, file] = value.split("=", 2);
      if (!variant || !file) throw new Error(`${flag} expects <variant>=<path>`);
      (flag === "--cli" ? options.cli : options.binding).set(variant, path.resolve(file));
    } else throw new Error(`Unknown argument: ${flag}`);
  }
  if (!Number.isInteger(options.rounds) || options.rounds < 1)
    throw new Error("--rounds must be >= 1");
  if (!options.out || !options.corpus) throw new Error("--out and --corpus are required");
  return options;
}

/** Alternates which variant goes first so neither side always runs warm. */
export function roundOrder(variants: readonly string[], round: number): string[] {
  return round % 2 === 0 ? [...variants] : [...variants].reverse();
}

function vueFiles(directory: string): string[] {
  return fs
    .readdirSync(directory, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".vue"))
    .map((entry) => path.join(entry.parentPath, entry.name))
    .sort();
}

// Loads one binding in a fresh process and compiles every corpus file 5 times.
function timed(command: string, args: readonly string[], cwd: string): number {
  const started = process.hrtime.bigint();
  const result = spawnSync(command, args, { cwd, encoding: "utf8", maxBuffer: 1 << 30 });
  const seconds = Number(process.hrtime.bigint() - started) / 1e9;
  if (result.error) throw result.error;
  // `vize lint` exits 1 when it reports problems; only a crash invalidates a sample.
  if (result.status === null || result.status > 1) {
    throw new Error(`${command} ${args.join(" ")} failed (${result.status})\n${result.stderr}`);
  }
  return seconds;
}

export function runBench(args: readonly string[]): number {
  const options = parseRun(args);
  const corpus = path.resolve(options.corpus);
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "glibc-floor-bench-"));
  const list = path.join(scratch, "files.txt");
  fs.writeFileSync(list, `${vueFiles(corpus).join("\n")}\n`);
  const scenarios: [string, Map<string, string>, (file: string, round: number) => number][] = [
    [
      "cli-lint",
      options.cli,
      (cli) => timed(cli, ["lint", corpus, "--no-config", "--format", "json", "--quiet"], scratch),
    ],
    [
      "cli-build",
      options.cli,
      (cli, round) =>
        timed(
          cli,
          ["build", corpus, "--output", path.join(scratch, `build-${round}`), "--no-config"],
          scratch,
        ),
    ],
    [
      "binding-compile",
      options.binding,
      (binding) => {
        const result = spawnSync(
          process.execPath,
          [fileURLToPath(new URL("./glibc-floor-compile.cjs", import.meta.url)), binding, list],
          {
            encoding: "utf8",
          },
        );
        if (result.status !== 0) throw new Error(`compile bench failed\n${result.stderr}`);
        return Number(result.stdout.split(" ")[0]);
      },
    ],
  ];
  const samples: Sample[] = [];
  for (const [scenario, variants, measure] of scenarios) {
    if (variants.size === 0) continue;
    const names = [...variants.keys()];
    // One unrecorded warm-up per variant fills the page cache for both.
    for (const name of names) measure(variants.get(name)!, -1);
    for (let round = 0; round < options.rounds; round += 1) {
      for (const variant of roundOrder(names, round)) {
        const seconds = measure(variants.get(variant)!, round);
        samples.push({ scenario, variant, round, seconds });
        console.log(`${scenario} ${variant} round ${round}: ${seconds.toFixed(4)}s`);
      }
    }
  }
  fs.mkdirSync(path.dirname(path.resolve(options.out)), { recursive: true });
  fs.writeFileSync(options.out, `${JSON.stringify({ samples }, null, 2)}\n`);
  fs.rmSync(scratch, { recursive: true, force: true });
  return 0;
}

function median(values: readonly number[]): number {
  const sorted = [...values].sort((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[middle]! : (sorted[middle - 1]! + sorted[middle]!) / 2;
}

/** Markdown table: per scenario, each variant's median and spread, and floor/host. */
export function summarize(samples: readonly Sample[]): string {
  const lines = [
    "| Scenario | Variant | Rounds | Median (s) | Min (s) | Max (s) | vs host |",
    "| --- | --- | --- | --- | --- | --- | --- |",
  ];
  for (const scenario of new Set(samples.map((sample) => sample.scenario))) {
    const of = (variant: string) =>
      samples.filter((sample) => sample.scenario === scenario && sample.variant === variant);
    const host = median(of("host").map((sample) => sample.seconds));
    const variants = samples.filter((s) => s.scenario === scenario).map((s) => s.variant);
    for (const variant of new Set(variants)) {
      const seconds = of(variant).map((sample) => sample.seconds);
      const ratio =
        Number.isFinite(host) && host > 0
          ? `${((median(seconds) / host - 1) * 100).toFixed(1)}%`
          : "";
      lines.push(
        `| ${scenario} | ${variant} | ${seconds.length} | ${median(seconds).toFixed(4)} | ${Math.min(...seconds).toFixed(4)} | ${Math.max(...seconds).toFixed(4)} | ${variant === "host" ? "" : ratio} |`,
      );
    }
  }
  return `${lines.join("\n")}\n`;
}

export function main(args: readonly string[]): number {
  const [command, ...rest] = args;
  if (command === "run") return runBench(rest);
  if (command === "summarize" && rest.length === 1) {
    const { samples } = JSON.parse(fs.readFileSync(rest[0]!, "utf8")) as { samples: Sample[] };
    process.stdout.write(summarize(samples));
    return 0;
  }
  throw new Error(
    "Usage: node tools/support/github/glibc-floor-bench.ts <run ... | summarize <json>>",
  );
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(process.argv.slice(2)));
