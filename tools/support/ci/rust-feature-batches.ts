// Packs selected experimental Cargo feature rows into matrix jobs. Every job pays
// about 35 s of checkout, toolchain, sticky-disk and dependency setup before its
// first Cargo command, while half the rows run under a minute of Cargo work. At
// the Blacksmith concurrency cap each extra job also waits for a 32-vCPU slot.
// A job therefore runs several rows in sequence, and the estimates below decide
// only which rows share a job, never whether a row runs.
import type { RustFeatureRow } from "./rust-feature-tests.ts";

export interface RustFeatureMatrix {
  readonly include: readonly { readonly rows: string; readonly runner: string }[];
}

// Most of a row's Cargo work walks the workspace crate chain one crate at a
// time, which leaves a 32-vCPU runner mostly idle; Linux rows take the 16-vCPU
// validation pool instead.
const linuxRunner = "blacksmith-16vcpu-ubuntu-2404";
const windowsRunner = "blacksmith-32vcpu-windows-2025";

/** Separates the row ids of one job, in the job name and on the `run` command line. */
export const RUST_FEATURE_ROW_SEPARATOR = ", ";

// Cargo work per job, in seconds. Near the longest single Linux row plus its
// setup, so a packed job ends with the Rust test shards that gate the same report.
export const RUST_FEATURE_BATCH_SECONDS = 240;

// Cargo work of each row on the Linux runner: the 75th-percentile job duration
// over a day of PR and merge-queue runs, minus the shared setup, rounded to 5 s.
// A row without an entry is costed at the whole budget.
export const rustFeatureRowSeconds: Readonly<Record<string, number>> = {
  "art-sources": 10,
  "bundler-blocks": 20,
  "cli-feature-sets": 30,
  "client-server-decisions": 25,
  "component-listeners": 50,
  "component-props": 50,
  "declaration-products": 145,
  "diagnostic-positions": 25,
  "editor-script-queries": 90,
  "expression-queries": 20,
  "graph-queries": 45,
  "image-alt-rule-group": 55,
  "jsx-module-projection": 20,
  "legacy-language-server": 135,
  "lexical-arrows": 65,
  "literal-products": 95,
  "navigation-products": 50,
  "open-document-blocks": 45,
  "plugin-template-products": 105,
  "production-reach": 15,
  "profile-allocations": 15,
  "resident-template-passes": 125,
  "retained-script-formatting": 25,
  "retained-tsrx": 45,
  "script-binding-products": 45,
  "script-query-tools": 70,
  "scss-products": 155,
  "style-modules": 100,
  "styled-projections": 70,
  "template-passes": 75,
  "template-tool-projections": 65,
  "versioned-fixes": 75,
};

const rowSeconds = (row: RustFeatureRow): number =>
  rustFeatureRowSeconds[row.id] ?? RUST_FEATURE_BATCH_SECONDS;

// Longest-processing-time packing onto the least-loaded job. Ties resolve by
// declaration order and job index, so the same selection always yields the same jobs.
function pack(rows: readonly RustFeatureRow[], total: number): RustFeatureRow[][] {
  const loads = Array.from({ length: total }, () => 0);
  const jobs: number[][] = Array.from({ length: total }, () => []);
  const order = rows
    .map((row, index) => ({ index, seconds: rowSeconds(row) }))
    .sort((left, right) => right.seconds - left.seconds || left.index - right.index);
  for (const { index, seconds } of order) {
    let target = 0;
    for (let job = 1; job < total; job += 1) if (loads[job]! < loads[target]!) target = job;
    loads[target]! += seconds;
    jobs[target]!.push(index);
  }
  return jobs
    .filter((indexes) => indexes.length > 0)
    .map((indexes) => indexes.sort((left, right) => left - right).map((index) => rows[index]!));
}

/** The fewest jobs whose estimated Cargo work stays within the budget (or within the heaviest row). */
export function rustFeatureBatches(rows: readonly RustFeatureRow[]): RustFeatureRow[][] {
  if (rows.length === 0) return [];
  const budget = Math.max(RUST_FEATURE_BATCH_SECONDS, ...rows.map(rowSeconds));
  for (let total = 1; ; total += 1) {
    const jobs = pack(rows, total);
    const fits = jobs.every((job) => job.reduce((sum, row) => sum + rowSeconds(row), 0) <= budget);
    if (fits || total === rows.length) return jobs;
  }
}

/** Matrix of the selected rows: Windows rows never share a job with Linux rows. */
export function rustFeatureMatrix(rows: readonly RustFeatureRow[]): RustFeatureMatrix {
  const entry = (runner: string) => (job: readonly RustFeatureRow[]) => ({
    rows: job.map((row) => row.id).join(RUST_FEATURE_ROW_SEPARATOR),
    runner,
  });
  return {
    include: [
      ...rustFeatureBatches(rows.filter((row) => !row.windows)).map(entry(linuxRunner)),
      ...rustFeatureBatches(rows.filter((row) => row.windows)).map(entry(windowsRunner)),
    ],
  };
}

/** Row ids of one matrix job's `rows` value. */
export function rustFeatureJobRows(rows: string): string[] {
  return rows.split(RUST_FEATURE_ROW_SEPARATOR);
}
