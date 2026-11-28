// Run a CI lane with a private temp directory and fail when Vize leaves
// storage behind in it.
//
//   node tools/ci/lib/scripts/temp-leak-gate.ts --label <lane> [--repeat <n>] -- <command>...
//
// Each run gets a fresh TMPDIR/TMP/TEMP below RUNNER_TEMP (or the system temp
// directory). After the command exits, every entry still in that directory is
// reported with its file count and bytes, beside the run's wall time and the
// command's user/sys CPU time; repeated runs also report p50 and p95. An entry
// Vize owns (named vize*, .vize*, or oxlint-vize*) fails the gate unless it
// is one of the shared roots that Vize keeps on purpose and it holds no file.
// The private directory is removed afterwards. The command's own failure wins
// over a leak.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { isEntrypoint } from "../../../support/command-runtime.ts";

export interface Entry {
  readonly name: string;
  readonly files: number;
  readonly bytes: number;
}

export interface RunReport {
  readonly label: string;
  readonly run: number;
  readonly status: number;
  readonly wallMs: number;
  readonly userMs: number | null;
  readonly sysMs: number | null;
  readonly files: number;
  readonly bytes: number;
  readonly entries: readonly Entry[];
  readonly leaked: readonly string[];
}

const ownedName = /^(?:\.?vize|oxlint-vize)/u;
// Per-user roots that hold per-session entries; an empty tree is expected.
const retainedRoot = /^vize-(?:canon|oxlint)(?:-\d+)?$/u;
const marker = "vize-temp-leak";

export function measure(target: string): { files: number; bytes: number } {
  const stat = fs.lstatSync(target);
  if (!stat.isDirectory()) return { files: 1, bytes: stat.size };
  let files = 0;
  let bytes = 0;
  for (const name of fs.readdirSync(target)) {
    const inner = measure(path.join(target, name));
    files += inner.files;
    bytes += inner.bytes;
  }
  return { files, bytes };
}

export function leftovers(directory: string): Entry[] {
  return fs
    .readdirSync(directory)
    .sort()
    .map((name) => ({ name, ...measure(path.join(directory, name)) }));
}

/** Leftover entries that Vize owns and did not keep on purpose. */
export function leaked(entries: readonly Entry[]): string[] {
  return entries
    .filter((entry) => ownedName.test(entry.name))
    .filter((entry) => !(retainedRoot.test(entry.name) && entry.files === 0))
    .map((entry) => entry.name);
}

/** The children line of POSIX `times`: `<user> <sys>`, each `<m>m<s>s`. */
export function parseTimes(output: string): { userMs: number; sysMs: number } | null {
  const line = output.trim().split("\n").at(-1) ?? "";
  const values = [...line.matchAll(/(\d+)m([\d.]+)s/gu)].map(
    ([, minutes, seconds]) => (Number(minutes) * 60 + Number(seconds)) * 1000,
  );
  return values.length === 2 ? { userMs: values[0], sysMs: values[1] } : null;
}

export function percentile(values: readonly number[], fraction: number): number {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.min(sorted.length - 1, Math.ceil(fraction * sorted.length) - 1)];
}

function runOnce(label: string, run: number, command: readonly string[]): RunReport {
  const base = process.env.RUNNER_TEMP ?? os.tmpdir();
  const directory = fs.mkdtempSync(path.join(base, "temp-leak-gate-"));
  const env = { ...process.env, TMPDIR: directory, TMP: directory, TEMP: directory };
  const timed = process.platform !== "win32";
  const started = process.hrtime.bigint();
  // `times` after the command reports the CPU time of every waited child.
  const result = timed
    ? spawnSync(
        "sh",
        ["-c", 'status=0; "$@" || status=$?; times >&3; exit "$status"', "sh", ...command],
        { env, stdio: ["inherit", "inherit", "inherit", "pipe"], encoding: "utf8" },
      )
    : spawnSync(command[0], command.slice(1), { env, stdio: "inherit" });
  const wallMs = Number(process.hrtime.bigint() - started) / 1e6;
  const cpu = timed ? parseTimes(String(result.output?.[3] ?? "")) : null;
  try {
    const entries = leftovers(directory);
    return {
      label,
      run,
      status: result.status ?? 1,
      wallMs,
      userMs: cpu?.userMs ?? null,
      sysMs: cpu?.sysMs ?? null,
      files: entries.reduce((sum, entry) => sum + entry.files, 0),
      bytes: entries.reduce((sum, entry) => sum + entry.bytes, 0),
      entries,
      leaked: leaked(entries),
    };
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

export function summary(reports: readonly RunReport[]): string[] {
  const lines = [
    `| lane | run | status | wall ms | sys ms | files | bytes | leaked |`,
    `| --- | --- | --- | --- | --- | --- | --- | --- |`,
  ];
  for (const report of reports)
    lines.push(
      `| ${report.label} | ${report.run} | ${report.status} | ${report.wallMs.toFixed(0)} | ${report.sysMs?.toFixed(0) ?? "n/a"} | ${report.files} | ${report.bytes} | ${report.leaked.join(", ") || "none"} |`,
    );
  if (reports.length > 1) {
    const wall = reports.map((report) => report.wallMs);
    const sys = reports.flatMap((report) => (report.sysMs === null ? [] : [report.sysMs]));
    lines.push(
      "",
      `wall p50 ${percentile(wall, 0.5).toFixed(0)} ms, p95 ${percentile(wall, 0.95).toFixed(0)} ms` +
        (sys.length
          ? `; sys p50 ${percentile(sys, 0.5).toFixed(0)} ms, p95 ${percentile(sys, 0.95).toFixed(0)} ms`
          : ""),
    );
  }
  return lines;
}

export function parseArguments(argv: readonly string[]): {
  label: string;
  repeat: number;
  command: string[];
} {
  const split = argv.indexOf("--");
  const options = split < 0 ? argv : argv.slice(0, split);
  const value = (name: string) => {
    const at = options.indexOf(name);
    return at < 0 ? undefined : options[at + 1];
  };
  const command = split < 0 ? [] : argv.slice(split + 1);
  const label = value("--label");
  const repeat = Number(value("--repeat") ?? "1");
  if (!label || !command.length || !Number.isInteger(repeat) || repeat < 1)
    throw new Error("usage: temp-leak-gate.ts --label <lane> [--repeat <n>] -- <command>...");
  return { label, repeat, command };
}

export function main(argv: readonly string[]): number {
  const { label, repeat, command } = parseArguments(argv);
  const reports: RunReport[] = [];
  for (let run = 1; run <= repeat; run++) {
    const report = runOnce(label, run, command);
    reports.push(report);
    process.stdout.write(`${marker} ${JSON.stringify(report)}\n`);
    if (report.status !== 0 || report.leaked.length) break;
  }
  const table = summary(reports);
  process.stdout.write(`${table.join("\n")}\n`);
  if (process.env.GITHUB_STEP_SUMMARY)
    fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${table.join("\n")}\n\n`);
  const failed = reports.find((report) => report.status !== 0);
  if (failed) return failed.status;
  const leaking = reports.find((report) => report.leaked.length);
  if (!leaking) return 0;
  for (const name of leaking.leaked) {
    const entry = leaking.entries.find((candidate) => candidate.name === name);
    process.stderr.write(
      `${label}: left ${name} in the temp directory (${entry?.files ?? 0} files, ${entry?.bytes ?? 0} bytes)\n`,
    );
  }
  return 1;
}

if (isEntrypoint(import.meta.url)) process.exitCode = main(process.argv.slice(2));
