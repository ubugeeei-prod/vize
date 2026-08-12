//! The manifest-driven supervisor for `test:check:fixtures` (#4126).
//!
//! Usage (from `tests/`):
//!
//! ```sh
//! node tooling/support/check-fixtures/supervisor.ts
//! node tooling/support/check-fixtures/supervisor.ts --only typecheck-errors
//! ```
//!
//! Every phase in `manifest.ts` runs here, in manifest order, one at a time,
//! with the same runner options the old single shell string used. The
//! supervisor adds three things that string could not: per-phase attribution of
//! the process budget, an artifact that survives a mid-run kill, and a guard
//! that fails the lane when a phase leaves a `node`, `vize`, `tsgo`, or `corsa`
//! task behind.
//!
//! It deliberately does not retry, sleep, or swallow a spawn failure: an
//! `EAGAIN` is reported as `spawn-failed` with the runner's budget attached.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  CHECK_FIXTURE_ENV,
  CHECK_FIXTURE_NODE_ARGS,
  type CheckFixturePhase,
  checkFixturePhases,
} from "./manifest.ts";
import { describePhase, runPhase } from "./phase-runner.ts";
import { createReport, type SupervisorReport, writeReport } from "./report.ts";
import { readRunnerFacts } from "./runner-facts.ts";

const HERE = path.dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = path.resolve(HERE, "../../../..");
export const TESTS_DIR = path.join(REPO_ROOT, "tests");
export const DEFAULT_METRICS_DIR = path.join(
  REPO_ROOT,
  "target",
  "vize-tests",
  "metrics",
  "check-fixtures-topology",
);
export const DEFAULT_SAMPLE_INTERVAL_MS = 200;

export type SupervisorOptions = {
  readonly phases?: readonly CheckFixturePhase[];
  readonly cwd?: string;
  readonly metricsDir?: string;
  readonly nodeArgs?: readonly string[];
  readonly env?: NodeJS.ProcessEnv;
  readonly sampleIntervalMs?: number;
  /** Capture phase output rather than inheriting the lane's streams. */
  readonly capture?: boolean;
};

/**
 * Run every phase and return the report.
 *
 * A failing phase does not stop the lane. The old runner reported all 35 files
 * before exiting non-zero, and an artifact that stops at the first red phase
 * would lose the topology evidence for everything after it.
 */
export async function runSupervisor(options: SupervisorOptions = {}): Promise<SupervisorReport> {
  const phases = options.phases ?? checkFixturePhases;
  const cwd = options.cwd ?? TESTS_DIR;
  const metricsDir = options.metricsDir ?? DEFAULT_METRICS_DIR;
  const report = createReport(readRunnerFacts(), cwd);
  writeReport(metricsDir, report);

  const startedAt = performance.now();
  for (const phase of phases) {
    const outcome = await runPhase(phase, {
      capture: options.capture,
      cwd,
      env: { ...process.env, ...CHECK_FIXTURE_ENV, ...options.env },
      nodeArgs: options.nodeArgs ?? CHECK_FIXTURE_NODE_ARGS,
      sampleIntervalMs: options.sampleIntervalMs ?? DEFAULT_SAMPLE_INTERVAL_MS,
      startedAt,
    });
    report.phases.push(outcome);
    console.log(`[check-fixtures] ${describePhase(outcome)}`);
    writeReport(metricsDir, report);
  }

  report.finishedAtIso = new Date().toISOString();
  report.status = report.phases.every((phase) => phase.status === "passed") ? "passed" : "failed";
  writeReport(metricsDir, report);
  return report;
}

/**
 * Parse a sampling interval in milliseconds.
 *
 * `Number.parseInt` alone lets `--sample-interval-ms nope` through as `NaN`,
 * and `setInterval` clamps `NaN` and `0` to one millisecond, which turns the
 * sampler into a busy loop that competes with the phase for the very process
 * budget it is measuring.
 */
function parseIntervalMs(flag: string, value: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 1) {
    throw new Error(`${flag} expects an integer of at least 1, received: ${value}`);
  }
  return parsed;
}

export function parseArgs(argv: readonly string[]): {
  only: string[];
  metricsDir: string | undefined;
  sampleIntervalMs: number | undefined;
} {
  const only: string[] = [];
  let metricsDir: string | undefined;
  let sampleIntervalMs: number | undefined;
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    const value = argv[index + 1];
    if (flag === "--only" && value != null) {
      only.push(value);
      index += 1;
    } else if (flag === "--metrics-dir" && value != null) {
      metricsDir = value;
      index += 1;
    } else if (flag === "--sample-interval-ms" && value != null) {
      sampleIntervalMs = parseIntervalMs(flag, value);
      index += 1;
    } else {
      throw new Error(`unknown supervisor argument: ${String(flag)}`);
    }
  }
  return { metricsDir, only, sampleIntervalMs };
}

/** Resolve `--only` ids against the manifest, failing closed on a typo. */
export function selectPhases(only: readonly string[]): readonly CheckFixturePhase[] {
  if (only.length === 0) {
    return checkFixturePhases;
  }
  return only.map((id) => {
    const phase = checkFixturePhases.find((candidate) => candidate.id === id);
    if (phase == null) {
      throw new Error(`unknown phase id: ${id}`);
    }
    return phase;
  });
}

function failureSummary(report: SupervisorReport): string[] {
  return report.phases
    .filter((phase) => phase.status !== "passed")
    .map((phase) => `  ${describePhase(phase)}`);
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  const report = await runSupervisor({
    metricsDir: args.metricsDir,
    phases: selectPhases(args.only),
    sampleIntervalMs: args.sampleIntervalMs,
  });
  const failures = failureSummary(report);
  if (failures.length > 0) {
    console.error(`[check-fixtures] ${failures.length} phase(s) failed:`);
    for (const line of failures) {
      console.error(line);
    }
    process.exitCode = 1;
  }
}

/**
 * Decide whether this module is the process entry point.
 *
 * A plain `process.argv[1] === fileURLToPath(import.meta.url)` compares an
 * unresolved argv path against a specifier ESM already resolved through
 * symlinks, so any symlinked checkout made the two differ, skipped `main`, and
 * exited 0 with no phase run at all. Prefer `import.meta.main`, and fall back
 * to comparing both sides after `realpath`.
 */
function invokedDirectly(entry: string | undefined, modulePath: string): boolean {
  // Read reflectively: `import.meta.main` only exists on newer runtimes.
  const flagged: unknown = Reflect.get(import.meta, "main");
  if (typeof flagged === "boolean") {
    return flagged;
  }
  if (entry == null) {
    return false;
  }
  return fs.realpathSync(entry) === fs.realpathSync(modulePath);
}

const modulePath = fileURLToPath(import.meta.url);
const entryPath = process.argv[1];
if (invokedDirectly(entryPath, modulePath)) {
  await main();
} else if (entryPath != null && path.basename(entryPath) === path.basename(modulePath)) {
  // Invoked as a script yet not recognised as the entry point: report it rather
  // than exiting 0 for phases that never ran.
  console.error(
    `[check-fixtures] refusing to exit 0: ${entryPath} looks like this supervisor ` +
      `but did not resolve to ${modulePath}`,
  );
  process.exit(1);
}
