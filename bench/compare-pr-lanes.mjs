/**
 * Lane catalogue for the PR and long-baseline benchmark gates.
 *
 * `bench/compare-pr.mjs` times each lane defined here for a base and a head
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
 * published tool benchmark measures: `bench/compare-tools.mjs` runs its
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
      allowNonZeroExit: true,
    },
    {
      id: "fmt",
      label: "Format (1T)",
      // `*.vue` instead of `.`: fmt expands `.` into a gitignore-aware walk
      // and bench/__in__ is gitignored, so that walk finds zero files; the
      // plain glob matches the corpus directly (same invocation as
      // bench/fmt.ts and compare-tools.mjs). `--check` formats in memory and
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
