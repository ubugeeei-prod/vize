import { execFileSync, spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";

import { parseTomlLite } from "../../support/compat/levels/toml-lite.ts";

interface TreeSample {
  rssBytes: number;
  processes: number;
}

interface Measurement {
  rss_peak_mib: number;
  rss_idle_mib: number;
  samples: number;
  max_processes: number;
  elapsed_seconds: number;
  session: Record<string, unknown>;
}

interface BudgetFile {
  resource?: Record<string, Record<string, { ceiling?: number }>>;
}

const root = fileURLToPath(new URL("../../../", import.meta.url));
const pageBytes = Number(execFileSync("getconf", ["PAGESIZE"], { encoding: "utf8" }));

function option(args: readonly string[], name: string): string | undefined {
  const index = args.indexOf(name);
  return index === -1 ? undefined : args[index + 1];
}

function positive(args: readonly string[], name: string, fallback: number): number {
  const source = option(args, name);
  if (source === undefined) return fallback;
  const value = Number(source);
  if (!Number.isSafeInteger(value) || value <= 0)
    throw new Error(`${name} needs a positive integer`);
  return value;
}

/** Sum resident pages for one live process and all of its descendants. */
function sampleProcessTree(rootPid: number): TreeSample | undefined {
  const parentByPid = new Map<number, number>();
  const residentPages = new Map<number, number>();
  for (const name of fs.readdirSync("/proc")) {
    if (!/^\d+$/u.test(name)) continue;
    try {
      const pid = Number(name);
      const stat = fs.readFileSync(`/proc/${name}/stat`, "utf8");
      const fields = stat
        .slice(stat.lastIndexOf(") ") + 2)
        .trim()
        .split(/\s+/u);
      parentByPid.set(pid, Number(fields[1]));
      residentPages.set(pid, Math.max(0, Number(fields[21])));
    } catch {
      // A process may exit between enumerating /proc and reading its stat row.
    }
  }
  if (!residentPages.has(rootPid)) return undefined;
  let pages = 0;
  let processes = 0;
  for (const [pid, rss] of residentPages) {
    let cursor: number | undefined = pid;
    const visited = new Set<number>();
    while (cursor !== undefined && cursor !== 0 && !visited.has(cursor)) {
      if (cursor === rootPid) {
        pages += rss;
        processes += 1;
        break;
      }
      visited.add(cursor);
      cursor = parentByPid.get(cursor);
    }
  }
  return { rssBytes: pages * pageBytes, processes };
}

async function measure(binary: string, files: number, edits: number): Promise<Measurement> {
  const child = spawn(
    binary,
    ["--files", String(files), "--edits", String(edits), "--hold-seconds", "10"],
    { stdio: ["ignore", "pipe", "pipe"] },
  );
  if (child.pid === undefined) throw new Error(`cannot start resident probe: ${binary}`);
  const stdout: Buffer[] = [];
  const stderr: Buffer[] = [];
  child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
  child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));

  let exitCode: number | null | undefined;
  let exitSignal: NodeJS.Signals | null = null;
  let processError: Error | undefined;
  const exited = new Promise<void>((resolve) => {
    child.once("error", (error) => {
      processError = error;
      exitCode = -1;
      resolve();
    });
    child.once("close", (code, signal) => {
      exitCode = code;
      exitSignal = signal;
      resolve();
    });
  });
  const started = performance.now();
  const tail: number[] = [];
  let peak = 0;
  let samples = 0;
  let maxProcesses = 0;
  while (exitCode === undefined && performance.now() - started < 300_000) {
    const sample = sampleProcessTree(child.pid);
    if (sample !== undefined) {
      peak = Math.max(peak, sample.rssBytes);
      if (sample.rssBytes > 0) {
        tail.push(sample.rssBytes);
        if (tail.length > 100) tail.shift();
      }
      samples += 1;
      maxProcesses = Math.max(maxProcesses, sample.processes);
    }
    await delay(50);
  }
  if (exitCode === undefined) {
    child.kill("SIGKILL");
    await exited;
    throw new Error("resident probe did not finish within 300 seconds");
  }
  await exited;
  if (processError !== undefined)
    throw new Error(`cannot run resident probe: ${processError.message}`);
  if (exitCode !== 0) {
    throw new Error(
      `resident probe failed with ${exitSignal ?? exitCode}: ${Buffer.concat(stderr).toString("utf8")}`,
    );
  }
  const session = JSON.parse(Buffer.concat(stdout).toString("utf8")) as Record<string, unknown>;
  const idle = Math.max(0, ...tail);
  const validBooleans = ["stage_equivalence", "summary_equivalence", "stale_config_rejected"].every(
    (key) => session[key] === true,
  );
  if (
    session.files !== files ||
    session.edits !== edits ||
    !validBooleans ||
    session.body_edit_dependent_executions !== 0 ||
    session.signature_edit_affected_consumers !== 1 ||
    samples === 0 ||
    peak === 0 ||
    idle === 0
  ) {
    throw new Error(`incomplete resident session or RSS evidence: ${JSON.stringify(session)}`);
  }
  return {
    rss_peak_mib: peak / 1_048_576,
    rss_idle_mib: idle / 1_048_576,
    samples,
    max_processes: maxProcesses,
    elapsed_seconds: (performance.now() - started) / 1_000,
    session,
  };
}

function ceiling(budgets: BudgetFile, preset: string, metric: string): number {
  const value = budgets.resource?.[preset]?.[metric]?.ceiling;
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) {
    throw new Error(`missing positive ${preset}.${metric} ceiling`);
  }
  return value;
}

async function main(): Promise<void> {
  if (process.platform !== "linux" || !Number.isFinite(pageBytes) || pageBytes <= 0) {
    throw new Error("resident RSS measurement requires the Linux reference runner");
  }
  const args = process.argv.slice(2);
  const server = option(args, "--server");
  if (server === undefined)
    throw new Error("--server <resource_session example binary> is required");
  const binary = fs.realpathSync(server);
  const files = positive(args, "--files", 10_000);
  const edits = positive(args, "--edits", 32);
  const runs = positive(args, "--runs", 3);
  const budgets = parseTomlLite(
    fs.readFileSync(path.join(root, "docs/davinci/plan/budgets.toml"), "utf8"),
  ) as BudgetFile;
  const preset = "linux-x64-ci";
  const peakCap = ceiling(budgets, preset, "rss_peak_mib");
  const idleCap = ceiling(budgets, preset, "rss_idle_mib");
  const measurements: Measurement[] = [];
  for (let run = 0; run < runs; run += 1) {
    const measurement = await measure(binary, files, edits);
    process.stderr.write(
      `resident resource run ${run + 1}/${runs}: ${JSON.stringify(measurement)}\n`,
    );
    measurements.push(measurement);
  }
  const maximum = (key: "rss_peak_mib" | "rss_idle_mib"): number =>
    Math.max(...measurements.map((measurement) => measurement[key]));
  const peak = maximum("rss_peak_mib");
  const idle = maximum("rss_idle_mib");
  const report = {
    scope: "synthetic resident database; fixture alpha inputs; no language server or Corsa",
    reference_preset: preset,
    files,
    edits_per_run: edits,
    runs,
    sampler: "Linux /proc/<pid>/stat RSS summed over the process tree every 50 ms",
    idle_hold_seconds: 10,
    idle_sampling: "maximum of the final 100 nonzero RSS samples (5 seconds)",
    statistic: "maximum over fresh-process runs",
    rss_peak_mib: peak,
    rss_idle_mib: idle,
    reference_rss_peak_ceiling_mib: peakCap,
    reference_rss_idle_ceiling_mib: idleCap,
    measurements,
  };
  const text = `${JSON.stringify(report, null, 2)}\n`;
  const output = option(args, "--out");
  if (output !== undefined) fs.writeFileSync(output, text);
  process.stdout.write(text);
  if (peak > peakCap || idle > idleCap) {
    throw new Error(
      `resident RSS exceeded the unchanged reference ceilings: peak ${peak}/${peakCap}, idle ${idle}/${idleCap} MiB`,
    );
  }
}

main().catch((error: unknown) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
});
