/**
 * Base vs head cost of the diagnostic digest (`vize check` / `vize build`
 * grouping, crates/vize/src/render/digest.rs) on real corpora, with an A/A
 * control: a second copy of the base binary measured as its own variant, so
 * the base/A-A spread is the runner's noise floor for every metric.
 *
 * Every round runs every variant once, in an order rotated per round, after
 * one unrecorded warm-up each. A run is a fresh process under GNU time (wall,
 * user+sys CPU, max RSS) and, when the kernel allows it, `perf stat`
 * (user-space instructions of the whole process tree).
 *
 * Each scenario also checks output, independently of timing:
 * - zero-digest scenarios (no diagnostic of a hinted kind) must print exactly
 *   what base prints;
 * - digest scenarios must print base's output under `--verbose`, and a digest
 *   block without it.
 * Build error listings come out in completion order, so outputs compare as
 * sorted lines; durations are normalized.
 *
 * Driver of .github/workflows/diagnostic-digest-bench.yml.
 */

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { catalogVersion } from "./typecheck-cold-pair.ts";

export type Variant = { name: string; bin: string };

export type Scenario = {
  id: string;
  /** `elk` or `misskey`: which corpus directory the command runs in. */
  corpus: "elk" | "misskey";
  args: (checker: string) => string[];
  /** Whether the corpus reports diagnostics of a hinted kind here. */
  digest: boolean;
};

const BUILD = ["build", "--no-config", "--format", "stats", "--slow-threshold", "1000000"];

export const SCENARIOS: readonly Scenario[] = [
  {
    id: "elk-build-strict",
    corpus: "elk",
    args: () => [...BUILD, "--template-syntax", "strict", "**/*.vue"],
    digest: true,
  },
  { id: "elk-build", corpus: "elk", args: () => [...BUILD, "**/*.vue"], digest: false },
  { id: "misskey-build", corpus: "misskey", args: () => [...BUILD, "src/**/*.vue"], digest: false },
  {
    id: "misskey-check-strict",
    corpus: "misskey",
    args: (checker) => [
      "check",
      "--no-config",
      "--corsa-path",
      checker,
      "--tsconfig",
      "tsconfig.digest-strict.json",
    ],
    digest: true,
  },
  {
    id: "misskey-check",
    corpus: "misskey",
    args: (checker) => ["check", "--no-config", "--corsa-path", checker],
    digest: false,
  },
];

/** The strict-templates project the digest scenarios check, beside the corpus's own. */
export const STRICT_TSCONFIG = `${JSON.stringify(
  { extends: "./tsconfig.json", vueCompilerOptions: { strictTemplates: true } },
  null,
  2,
)}\n`;

export type Sample = {
  wallMs: number;
  cpuMs: number;
  rssMb: number;
  instructions: number | null;
};

export type Row = { scenario: string; variant: string; exit: number | null; samples: Sample[] };

/** Variant order for round `round`: rotated so no variant always runs first. */
export function rotation<T>(items: readonly T[], round: number): T[] {
  const shift = round % Math.max(items.length, 1);
  return [...items.slice(shift), ...items.slice(0, shift)];
}

/** Nearest-rank percentile of `values` (`p` in 0..100). */
export function percentile(values: readonly number[], p: number): number {
  if (values.length === 0) return Number.NaN;
  const sorted = [...values].sort((left, right) => left - right);
  const rank = Math.min(sorted.length, Math.max(1, Math.ceil((p / 100) * sorted.length)));
  return sorted[rank - 1]!;
}

/** `%e %U %S %M` from GNU time: wall s, user s, system s, max RSS KiB. */
export function parseGnuTime(text: string): Omit<Sample, "instructions"> {
  const line = text.trim().split("\n").at(-1) ?? "";
  const [wall, user, system, rss] = line.trim().split(/\s+/u).map(Number);
  if ([wall, user, system, rss].some((value) => value === undefined || Number.isNaN(value))) {
    throw new Error(`unreadable GNU time output: ${text}`);
  }
  return { wallMs: wall! * 1000, cpuMs: (user! + system!) * 1000, rssMb: rss! / 1024 };
}

/** The `instructions:u` count of `perf stat -x,`, or null when not counted. */
export function parsePerfStat(text: string): number | null {
  for (const line of text.split("\n")) {
    const [value, , event] = line.split(",");
    if (event?.startsWith("instructions") && value && /^\d+$/u.test(value)) return Number(value);
  }
  return null;
}

/** An ANSI SGR escape (`ESC [ … m`). */
const ANSI = new RegExp(`${String.fromCharCode(0x1b)}\\[[0-9;]*m`, "gu");

/** Output with ANSI escapes and durations normalized, as sorted lines. */
export function normalizeOutput(text: string): string[] {
  return text
    .replace(ANSI, "")
    .replace(/\b\d+(?:\.\d+)?(?:ns|µs|ms|s)\b/gu, "<duration>")
    .split("\n")
    .map((line) => line.trimEnd())
    .filter((line) => line.length > 0)
    .sort();
}

type Run = { exit: number | null; output: string; sample: Sample };

/** GNU time; `DIGEST_BENCH_TIME` points at another install (macOS ships BSD time). */
const GNU_TIME = process.env.DIGEST_BENCH_TIME ?? "/usr/bin/time";

function run(bin: string, args: string[], cwd: string, work: string, perf: boolean): Run {
  const timeFile = join(work, "time.txt");
  const perfFile = join(work, "perf.txt");
  rmSync(perfFile, { force: true });
  const timed = ["-f", "%e %U %S %M", "-o", timeFile, bin, ...args];
  const [command, commandArgs] = perf
    ? ["perf", ["stat", "-x,", "-e", "instructions:u", "-o", perfFile, "--", GNU_TIME, ...timed]]
    : [GNU_TIME, timed];
  const result = spawnSync(command, commandArgs, {
    cwd,
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
    timeout: 900_000,
  });
  if (result.error) throw result.error;
  const sample = {
    ...parseGnuTime(readFileSync(timeFile, "utf8")),
    instructions:
      perf && existsSync(perfFile) ? parsePerfStat(readFileSync(perfFile, "utf8")) : null,
  };
  return { exit: result.status, output: `${result.stdout}${result.stderr}`, sample };
}

const REPO = join(fileURLToPath(import.meta.url), "../../../..");

/** The pinned native checker (`corsa-runtime` catalog), installed into `dir`. */
function installChecker(dir: string): string {
  const name = `@typescript/typescript-${process.platform}-${process.arch}`;
  const version = catalogVersion(
    readFileSync(join(REPO, "pnpm-workspace.yaml"), "utf8"),
    "corsa-runtime",
    name,
  );
  mkdirSync(dir, { recursive: true });
  writeFileSync(
    join(dir, "package.json"),
    `${JSON.stringify({ private: true, dependencies: { [name]: version } })}\n`,
  );
  const result = spawnSync(
    "npm",
    ["install", "--no-audit", "--no-fund", "--ignore-scripts", "--no-package-lock"],
    {
      cwd: dir,
      encoding: "utf8",
      timeout: 900_000,
    },
  );
  if (result.status !== 0) throw new Error(`checker install failed\n${result.stderr}`);
  const checker = join(dir, "node_modules", ...name.split("/"), "lib", "tsc");
  if (!existsSync(checker)) throw new Error(`checker missing at ${checker}`);
  return checker;
}

function perfAvailable(work: string): boolean {
  const probe = join(work, "perf-probe.txt");
  const result = spawnSync("perf", [
    "stat",
    "-x,",
    "-e",
    "instructions:u",
    "-o",
    probe,
    "--",
    "true",
  ]);
  return (
    result.status === 0 && existsSync(probe) && parsePerfStat(readFileSync(probe, "utf8")) !== null
  );
}

export type OutputCheck = { scenario: string; ok: boolean; note: string };

function diff(left: string[], right: string[]): string {
  const only = (from: string[], other: string[]) => {
    const rest = [...other];
    return from.filter((line) => {
      const at = rest.indexOf(line);
      if (at < 0) return true;
      rest.splice(at, 1);
      return false;
    });
  };
  const removed = only(left, right).slice(0, 5);
  const added = only(right, left).slice(0, 5);
  return [...removed.map((line) => `- ${line}`), ...added.map((line) => `+ ${line}`)].join("\n");
}

const fmt = (value: number, digits = 1) => (Number.isFinite(value) ? value.toFixed(digits) : "—");
const delta = (head: number, base: number) =>
  Number.isFinite(head) && Number.isFinite(base) && base !== 0
    ? `${head >= base ? "+" : ""}${(((head - base) / base) * 100).toFixed(2)}%`
    : "—";

export function report(
  rows: readonly Row[],
  checks: readonly OutputCheck[],
  notes: string[],
): string {
  const lines = ["# Diagnostic digest: base vs head, with an A/A control", ""];
  for (const scenario of new Set(rows.map((row) => row.scenario))) {
    const scenarioRows = rows.filter((row) => row.scenario === scenario);
    const p50 = (row: Row | undefined, pick: (sample: Sample) => number | null) =>
      percentile(
        (row?.samples ?? []).map(pick).filter((value): value is number => value !== null),
        50,
      );
    const base = scenarioRows.find((row) => row.variant === "base");
    lines.push(`## ${scenario}`, "");
    lines.push(
      "| variant | runs | exit | wall p50 ms | wall p95 ms | CPU p50 ms | max RSS p50 MiB | instructions p50 | Δ wall p50 | Δ CPU p50 | Δ instructions p50 |",
      "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    );
    for (const row of scenarioRows) {
      const wall = row.samples.map((sample) => sample.wallMs);
      const instructions = p50(row, (sample) => sample.instructions);
      lines.push(
        `| ${row.variant} | ${row.samples.length} | ${row.exit} | ${fmt(percentile(wall, 50))} | ${fmt(percentile(wall, 95))} | ${fmt(p50(row, (sample) => sample.cpuMs))} | ${fmt(p50(row, (sample) => sample.rssMb))} | ${Number.isFinite(instructions) ? instructions.toFixed(0) : "—"} | ${
          row === base
            ? ""
            : delta(
                percentile(wall, 50),
                p50(base, (sample) => sample.wallMs),
              )
        } | ${
          row === base
            ? ""
            : delta(
                p50(row, (sample) => sample.cpuMs),
                p50(base, (sample) => sample.cpuMs),
              )
        } | ${
          row === base
            ? ""
            : delta(
                instructions,
                p50(base, (sample) => sample.instructions),
              )
        } |`,
      );
    }
    for (const check of checks.filter((entry) => entry.scenario === scenario)) {
      lines.push("", `Output: ${check.ok ? "✓" : "✗"} ${check.note}`);
    }
    lines.push("");
  }
  if (notes.length > 0) lines.push("## Notes", "", ...notes.map((note) => `- ${note}`), "");
  return `${lines.join("\n")}\n`;
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      bin: { type: "string", multiple: true },
      elk: { type: "string" },
      misskey: { type: "string" },
      checker: { type: "string" },
      rounds: { type: "string", default: "7" },
      scenarios: { type: "string", default: "" },
      out: { type: "string" },
    },
  });
  const variants: Variant[] = (values.bin ?? []).map((spec) => {
    const [name = "", bin = ""] = spec.split("=");
    if (!name || !existsSync(bin))
      throw new Error(`--bin wants <name>=<existing path>, got ${spec}`);
    return { name, bin };
  });
  if (
    !variants.some((variant) => variant.name === "base") ||
    !variants.some((v) => v.name === "head")
  ) {
    throw new Error("--bin base=… and --bin head=… are required");
  }
  const rounds = Number(values.rounds);
  if (!Number.isInteger(rounds) || rounds < 1)
    throw new Error(`--rounds wants a count, got ${values.rounds}`);
  const out = values.out ?? "diagnostic-digest-bench";
  const work = join(out, "work");
  mkdirSync(work, { recursive: true });
  const dirs = { elk: values.elk ?? "", misskey: values.misskey ?? "" };
  const checker = values.checker ?? installChecker(join(out, "checker"));
  const selected = values.scenarios
    ? SCENARIOS.filter((scenario) => values.scenarios!.split(",").includes(scenario.id))
    : SCENARIOS;
  const perf = perfAvailable(work);
  const notes = [
    `runner: ${process.env.RUNNER_NAME ?? "local"}; ${rounds} rounds × ${variants.length} variants per scenario after one warm-up each; variant order rotated per round`,
    `A/A control: \`base-aa\` is a second copy of the base binary`,
    perf
      ? "instructions: `perf stat -e instructions:u` (process tree)"
      : "instructions: perf unavailable on this runner",
  ];
  if (dirs.misskey)
    writeFileSync(join(dirs.misskey, "tsconfig.digest-strict.json"), STRICT_TSCONFIG);

  const rows: Row[] = [];
  const checks: OutputCheck[] = [];
  for (const scenario of selected) {
    const cwd = dirs[scenario.corpus];
    if (!cwd || !existsSync(cwd))
      throw new Error(`--${scenario.corpus} is required for ${scenario.id}`);
    const args = scenario.args(checker);
    const outputs = new Map<string, Run>();
    for (const variant of variants)
      outputs.set(variant.name, run(variant.bin, args, cwd, work, false));
    const byVariant = new Map(variants.map((variant) => [variant.name, [] as Sample[]]));
    for (let round = 0; round < rounds; round += 1) {
      for (const variant of rotation(variants, round)) {
        byVariant.get(variant.name)!.push(run(variant.bin, args, cwd, work, perf).sample);
      }
      process.stderr.write(`${scenario.id}: round ${round + 1}/${rounds}\n`);
    }
    for (const variant of variants) {
      rows.push({
        scenario: scenario.id,
        variant: variant.name,
        exit: outputs.get(variant.name)!.exit,
        samples: byVariant.get(variant.name)!,
      });
    }

    const base = outputs.get("base")!;
    const head = outputs.get("head")!;
    const exits = new Set([...outputs.values()].map((entry) => entry.exit));
    if (exits.size !== 1) {
      checks.push({
        scenario: scenario.id,
        ok: false,
        note: `exit codes differ: ${[...exits].join(", ")}`,
      });
    }
    const baseLines = normalizeOutput(base.output);
    if (scenario.digest) {
      const headBin = variants.find((variant) => variant.name === "head")!.bin;
      const verbose = run(headBin, [...args, "--verbose"], cwd, work, false);
      const same = JSON.stringify(normalizeOutput(verbose.output)) === JSON.stringify(baseLines);
      const blocks = (head.output.replace(ANSI, "").match(/^(?:error|warning)\[/gmu) ?? []).length;
      checks.push({
        scenario: scenario.id,
        ok: same && blocks > 0 && verbose.exit === base.exit,
        note: `head \`--verbose\` ${same ? "prints exactly base's output" : `differs from base:\n\n\`\`\`diff\n${diff(baseLines, normalizeOutput(verbose.output))}\n\`\`\``}; head default prints ${blocks} digest block(s)`,
      });
    } else {
      const same = JSON.stringify(normalizeOutput(head.output)) === JSON.stringify(baseLines);
      checks.push({
        scenario: scenario.id,
        ok: same,
        note: same
          ? "head prints exactly base's output (no diagnostic of a hinted kind)"
          : `head differs from base:\n\n\`\`\`diff\n${diff(baseLines, normalizeOutput(head.output))}\n\`\`\``,
      });
    }
    writeFileSync(
      join(out, "results.json"),
      `${JSON.stringify({ rows, checks, notes }, null, 2)}\n`,
    );
  }
  const markdown = report(rows, checks, notes);
  writeFileSync(join(out, "report.md"), markdown);
  process.stdout.write(markdown);
  if (checks.some((check) => !check.ok)) process.exitCode = 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) await main();
