//! Focused cycle targets and their documented bounds (#4126).

import fs from "node:fs";
import path from "node:path";

export type CycleTarget = {
  readonly id: string;
  readonly description: string;
  /** Project directory, relative to the repository root. */
  readonly projectDir: string;
  /** Glob patterns handed to `vize check`. */
  readonly patterns: readonly string[];
  /** Explicit `--servers`, or `null` to let `vize` auto-tune. */
  readonly servers: number | null;
  /**
   * Whether the cycle includes generated virtual TypeScript in JSON output.
   *
   * The high-output ecosystem-products fixture hit the original spawn
   * exhaustion after printing virtual TS, so the constrained guard must cover
   * that output mode rather than only the quiet diagnostic report.
   */
  readonly showVirtualTs: boolean;
  /**
   * Hard cap on this target's cycle count, or `null` to run the full
   * `--cycles` count.
   *
   * Repetition is how the guard catches an intermittent spawn failure, so it is
   * traded away only for a target whose single cycle already costs a minute:
   * `ecosystem-products` typechecks its dependency-heavy corpus in ~69s per
   * cycle on the hosted lane (the blocking snapshot phase measures the same
   * 68.8s), and 20 of those spend ~23 minutes inside a `vue-parity` job with a
   * 30-minute timeout. Overrunning it reports as a *cancelled* job, which
   * fails the `test-report` gate without a single red assertion to read.
   */
  readonly maxCycles: number | null;
  /** Corsa CLI processes the run is expected to reach at its widest. */
  readonly corsaProcesses: number;
  /**
   * Files `vize check` must report on every cycle.
   *
   * Hash equality alone would be satisfied by a run that checked nothing, as
   * long as it checked nothing every time. This is the assertion that the
   * corpus was really walked; for a `_git` fixture it moves only when the
   * pinned submodule moves.
   */
  readonly expectedFileCount: number;
  /** Whether the corpus carries authored diagnostics that must keep appearing. */
  readonly expectsDiagnostics: boolean;
  /**
   * Processes the run may have live in its own group at peak: the `vize`
   * process plus one Corsa runtime per shard. Measured, not estimated — the
   * package-manager shim `exec`s in place, so a shard never costs more than one
   * slot.
   */
  readonly peakGroupProcessBound: number;
  /**
   * Documented peak live-task bound, as `base + perCpu * cpuCount`.
   *
   * A task is a thread: `RLIMIT_NPROC` and the cgroup `pids` controller both
   * count threads, and a Corsa process sizes its Go runtime from the core
   * count while `vize` sizes its checker pool the same way, so the bound has
   * to scale with the runner. This number is also the budget the cycle is
   * constrained to, so exceeding it fails the cycle with `EAGAIN` from the
   * kernel rather than with a soft comparison.
   */
  readonly taskBudget: (cpuCount: number) => number;
};

export const cycleTargets: readonly CycleTarget[] = [
  {
    corsaProcesses: 1,
    description:
      "the 18-file fixture the lane checks first; below the 64-file shard threshold, so exactly one Corsa process",
    expectedFileCount: 18,
    expectsDiagnostics: true,
    id: "single-corsa",
    maxCycles: null,
    patterns: ["src/**/*.vue"],
    peakGroupProcessBound: 2,
    projectDir: "tests/_fixtures/_projects/typecheck-errors",
    servers: null,
    showVirtualTs: false,
    taskBudget: (cpuCount) => 64 + 8 * cpuCount,
  },
  {
    corsaProcesses: 1,
    description:
      "the dependency-heavy ecosystem-products fixture in the same `--show-virtual-ts` mode as the blocking snapshot phase",
    expectedFileCount: 9,
    expectsDiagnostics: false,
    id: "ecosystem-products-virtual-ts",
    // Three is the smallest count that still measures reproducibility: cycle 1
    // pins the exit code and output hash, and cycles 2 and 3 have to reproduce
    // it. Anything cheaper would only prove the fixture checks once.
    maxCycles: 3,
    patterns: ["src/**/*.vue"],
    peakGroupProcessBound: 2,
    projectDir: "tests/_fixtures/_projects/ecosystem-products",
    servers: null,
    showVirtualTs: true,
    taskBudget: (cpuCount) => 128 + 16 * cpuCount,
  },
  {
    corsaProcesses: 8,
    description:
      "the create-vue template corpus at the shard cap; its templates partition into eight balanced components, so `--servers 8` really runs eight Corsa processes",
    expectedFileCount: 42,
    expectsDiagnostics: false,
    id: "maximum-shard",
    maxCycles: null,
    patterns: ["template/**/*.vue"],
    peakGroupProcessBound: 9,
    projectDir: "tests/_fixtures/_git/create-vue",
    servers: 8,
    showVirtualTs: false,
    taskBudget: (cpuCount) => 128 + 32 * cpuCount,
  },
];

const BIN_EXT = process.platform === "win32" ? ".exe" : "";

/**
 * Resolve the `vize` binary so a cycle measures the same build the fixture
 * phases run. `VIZE_TEST_BIN` (the test-suite convention the `vue-parity`
 * action sets) wins over `VIZE_BIN`, then the staged build profiles in the
 * order `tests/_helpers/apps.ts` prefers them.
 *
 * Every candidate resolves against `repoRoot` because a cycle spawns `vize`
 * with the fixture directory as its cwd, so the relative `target/ci/vize` the
 * action passes would otherwise be looked up under the fixture.
 */
export function resolveVizeBin(repoRoot: string): string {
  const override = process.env.VIZE_TEST_BIN ?? process.env.VIZE_BIN;
  if (override != null && override.length > 0) {
    return path.resolve(repoRoot, override);
  }
  const candidates = ["ci", "release", "debug"].map((profile) =>
    path.join(repoRoot, "target", profile, `vize${BIN_EXT}`),
  );
  return candidates.find((candidate) => fs.existsSync(candidate)) ?? candidates[1]!;
}

/** Resolve the Corsa runtime, preferring `corsa` and falling back to `tsgo`. */
export function resolveCorsaBin(repoRoot: string): string {
  const override = process.env.CORSA_BIN;
  if (override != null && override.length > 0) {
    return override;
  }
  const primary = path.join(repoRoot, "node_modules", ".bin", "corsa");
  return fs.existsSync(primary) ? primary : path.join(repoRoot, "node_modules", ".bin", "tsgo");
}

/** Build the `vize check` argv for one target. */
export function checkArgv(target: CycleTarget, corsaBin: string): string[] {
  const argv = [
    "check",
    ...target.patterns,
    "--format",
    "json",
    "--quiet",
    "--corsa-path",
    corsaBin,
  ];
  if (target.servers != null) {
    argv.push("--servers", String(target.servers));
  }
  if (target.showVirtualTs) {
    argv.push("--show-virtual-ts");
  }
  return argv;
}
