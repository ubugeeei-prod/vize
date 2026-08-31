/**
 * Lane catalogue for the PR and long-baseline benchmark gates.
 *
 * `tools/benchmarks/scripts/compare-pr.mjs` times each lane defined here for a base and a head
 * `vize` binary. `.github/workflows/benchmark.yml` runs that comparison twice:
 * per pull request against the PR base, and weekly against one fixed
 * historical SHA (#3586). Both runs read this list, so a surface is only
 * protected from long-term drift once it has a lane in it.
 */

import { existsSync } from "node:fs";
import { join } from "node:path";

/**
 * Build the child environment for one lane.
 *
 * Lanes are pinned to a single Rayon worker by default so a change in per-file
 * work cannot be diluted by the runner's core count. The `max` lanes delete the
 * pin instead of writing a core count, because that is exactly what the
 * published tool benchmark measures: `tools/benchmarks/scripts/compare-tools.mjs` runs its
 * `vize-lint-max` / `vize-fmt-max` variants with no `RAYON_NUM_THREADS` set at
 * all, so Rayon sizes its own pool from the machine.
 */
export function laneEnvironment(threads, base) {
  const env = { ...base };
  if (threads === "max") {
    delete env.RAYON_NUM_THREADS;
    return env;
  }
  env.RAYON_NUM_THREADS = String(threads);
  return env;
}

/**
 * Budget for the two lanes whose measured noise is wider than the shared 5%.
 *
 * The 5% default was sized (#1411) for lanes that take 120-190ms on the
 * 1000-file corpus. `lint-max` and `fmt-max` clear that same corpus in 37-39ms
 * with the Rayon pool unpinned, so 5% of their wall time is under 2ms -- inside
 * process startup and scheduler jitter on a shared 32-vCPU runner. Every
 * release commit gives a free null measurement, because its diff is version
 * strings in manifests, `Cargo.lock`, `editors/*` and READMEs and cannot change
 * throughput. Over the seven such comparisons run since #3622 introduced the
 * paired estimator (v0.313.0 through v0.317.0):
 *
 * | lane             | base median | change on a no-code diff | sd    |
 * | ---------------- | ----------: | ------------------------ | ----: |
 * | Format (max)     |      38.9ms | -3.93% .. +4.49%         |  2.58 |
 * | Lint (max)       |      37.1ms | -3.46% .. +0.60%         |  1.37 |
 * | Type check (1T)  |     830.2ms | -1.05% .. +3.29%         |  1.46 |
 * | Lint (1T)        |     122.5ms | -2.78% .. +1.90%         |  1.51 |
 * | Compile SFC      |     189.9ms | -2.55% .. +1.62%         |  1.41 |
 * | Type check (max) |     839.2ms | -2.45% .. +0.68%         |  1.08 |
 * | Format (1T)      |     147.4ms | -1.72% .. +1.23%         |  1.20 |
 *
 * `Format (max)` reached +4.49% on a commit that changed no code: 0.5 points
 * short of cancelling a release. (v0.312.0's gate did cancel one, reporting
 * +5.37%, but that predates #3622 -- rescoring that run's own samples with the
 * paired estimator gives -3.93%.) A budget sitting inside its lane's noise band
 * reports coin flips, not regressions.
 *
 * 10% is 2.2x the widest excursion measured on a diff that changes no code, and
 * more than three standard deviations above the noisier of the two lanes. It
 * stays far below what these lanes exist to catch: #3460's published
 * `vize lint (max)` moved +16% and `vize fmt (max)` +58% over a window the
 * pinned lanes scored as neutral-to-better, and both still fail at 10%.
 *
 * `check-max` is unpinned too but keeps the 5% default: at 839ms it is nowhere
 * near the noise floor, and the same seven comparisons put it inside +-2.45%.
 */
export const UNPINNED_FAST_LANE_THRESHOLD_PERCENT = 10;

/** The regression budget one lane is judged against. */
export function laneThresholdPercent(task, defaultThresholdPercent) {
  return task.thresholdPercent ?? defaultThresholdPercent;
}

export function makeTasks(inputDir, taskFilter) {
  const tsconfig = join(inputDir, "tsconfig.json");
  const pattern = ".";
  const allTasks = [
    {
      id: "compile",
      label: "Compile SFC",
      args: ["build", pattern, "--format", "stats", "--threads", "1", "--continue-on-error"],
      allowNonZeroExit: false,
    },
    {
      id: "lint",
      label: "Lint (1T)",
      args: ["lint", pattern, "--quiet"],
      allowNonZeroExit: true,
    },
    {
      // #3460 reported the published `vize lint (max)` median moving +16% over
      // a window in which the single-threaded median moved +2.6%. `vize lint`
      // on a large corpus is mostly serial wall time (file discovery, config,
      // summary) once the per-file work is spread across every core, so a
      // regression in that serial part is ~7x more visible here than in the
      // pinned lane above. Same invocation, unpinned pool.
      id: "lint-max",
      label: "Lint (max)",
      args: ["lint", pattern, "--quiet"],
      threads: "max",
      thresholdPercent: UNPINNED_FAST_LANE_THRESHOLD_PERCENT,
      allowNonZeroExit: true,
    },
    {
      id: "fmt",
      label: "Format (1T)",
      // `*.vue` instead of `.`: fmt expands `.` into a gitignore-aware walk
      // and tools/benchmarks/scripts/__in__ is gitignored, so that walk finds zero files; the
      // plain glob matches the corpus directly (same invocation as
      // tools/benchmarks/scripts/fmt.ts and compare-tools.mjs). `--check` formats in memory and
      // never writes, so the corpus stays byte-identical between the
      // alternating base/head runs. The generated corpus is intentionally
      // unformatted, so the non-zero "would reformat" exit is expected.
      // fmt has no --threads flag; `threads` pins the Rayon pool instead.
      args: ["fmt", "*.vue", "--check"],
      allowNonZeroExit: true,
    },
    {
      // #3460's published `vize fmt (max)` median moved +58% over a window in
      // which the single-threaded median moved -35%: formatting got cheaper
      // per file while the parallel wall clock got worse. A pinned lane scores
      // that window as an improvement, so the parallel path needs its own.
      id: "fmt-max",
      label: "Format (max)",
      args: ["fmt", "*.vue", "--check"],
      threads: "max",
      thresholdPercent: UNPINNED_FAST_LANE_THRESHOLD_PERCENT,
      allowNonZeroExit: true,
    },
    {
      id: "check",
      label: "Type check (1T)",
      // --servers 1 pins the lane to a single Corsa server so it isolates
      // single-program performance and stays deterministic on shared CI
      // runners. Keep this id stable so `--tasks check` still selects the
      // single-program lane.
      args: ["check", pattern, "--quiet", "--servers", "1", "--tsconfig", tsconfig],
      allowNonZeroExit: true,
      enabled: existsSync(tsconfig),
    },
    {
      id: "check-max",
      label: "Type check (max)",
      // Omit --servers so vize uses the same auto-tuned Corsa sharding path
      // measured by the tool benchmark. This catches regressions hidden by the
      // single-server lane without weakening the single-program signal above.
      args: ["check", pattern, "--quiet", "--tsconfig", tsconfig],
      allowNonZeroExit: true,
      enabled: existsSync(tsconfig),
    },
  ];

  const requested = new Set(
    taskFilter
      .split(",")
      .map((task) => task.trim())
      .filter(Boolean),
  );
  return allTasks.filter((task) => {
    if (task.enabled === false) {
      return false;
    }
    return requested.size === 0 || requested.has(task.id);
  });
}
