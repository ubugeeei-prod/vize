import { rmSync } from "node:fs";
import { resolve } from "node:path";

/** A `critcmp --export` snapshot: benchmark ids mapped to Criterion estimates. */
export type CritcmpExport = { name: string; benchmarks: Record<string, unknown> };
export type CriterionRegression = { name: string; changePercent: number };
export type AbsoluteBudget = { name: string; maxMedianNs: number };
export type AbsoluteBudgetResult = AbsoluteBudget & { medianNs: number; exceeded: boolean };
/** A cargo package's Criterion bench targets, as driven by `criterion-ab.ts`. */
export type CriterionSuite = {
  package: string;
  benches: string[];
  label: string;
  filter?: string;
  absoluteBudgets?: AbsoluteBudget[];
};
export type CriterionRunOptions = { cwd?: string; env?: Record<string, string>; capture?: boolean };
export type CritcmpBaseline = { targetDir: string; baseline: string };
export type CritcmpBaselineExport = CritcmpBaseline & { outputPath: string };
export type CritcmpPaths = { targetDir: string; baselinePaths: string[] };
export type CargoBench = CritcmpBaseline & { pkg: string; benches: string[]; filter?: string };
/** One side of the A/B run: its checkout and isolated Cargo target directory. */
export type CriterionCheckout = { checkoutDir: string; targetDir: string };
export type CriterionSide = CriterionCheckout &
  CritcmpBaseline & { side: string; suites: CriterionSuite[] };
export type VaporPairResult = {
  fixture: string;
  baseNativeNs: number;
  baseRetainedNs: number;
  headNativeNs: number;
  headRetainedNs: number;
  baseRatio: number;
  headRatio: number;
};

export function critcmpExportArgs({ targetDir, baseline }: CritcmpBaseline): string[] {
  return ["--target-dir", targetDir, "--export", baseline];
}

export function criterionEnvironment(targetDir: string): {
  CARGO_TARGET_DIR: string;
  CRITERION_HOME: string;
} {
  return { CARGO_TARGET_DIR: targetDir, CRITERION_HOME: resolve(targetDir, "criterion") };
}

export function resetCriterionResults(targetDir: string): void {
  // Cargo artifacts stay cached, but deleted or unselected benchmarks must not
  // survive into this run's critcmp export or satisfy an absolute budget.
  rmSync(criterionEnvironment(targetDir).CRITERION_HOME, { recursive: true, force: true });
}

export function critcmpArgs({ targetDir, baselinePaths }: CritcmpPaths): string[] {
  return ["--target-dir", targetDir, ...baselinePaths];
}

export function parseCritcmpExport(serialized: string, expectedBaseline: string): CritcmpExport {
  const parsed: unknown = JSON.parse(serialized);
  if (!isCritcmpExport(parsed) || parsed.name !== expectedBaseline) {
    throw new Error(`Invalid critcmp export for ${expectedBaseline}`);
  }
  if (Object.keys(parsed.benchmarks).length === 0) {
    throw new Error(`critcmp export for ${expectedBaseline} contains no benchmarks`);
  }
  return parsed;
}

export function compareBaselineExports(
  base: CritcmpExport,
  head: CritcmpExport,
  thresholdPercent: number | undefined,
): CriterionRegression[] {
  const shared = Object.keys(base.benchmarks).filter((name) => name in head.benchmarks);
  if (shared.length === 0) {
    throw new Error("Criterion base and head exports contain no shared benchmarks");
  }
  if (thresholdPercent == null) {
    return [];
  }

  const regressions: CriterionRegression[] = [];
  for (const name of shared) {
    const baseMedian = medianPointEstimate(base.benchmarks[name], `base/${name}`);
    const headMedian = medianPointEstimate(head.benchmarks[name], `head/${name}`);
    const changePercent = (headMedian / baseMedian - 1) * 100;
    if (changePercent >= thresholdPercent) {
      regressions.push({ name, changePercent });
    }
  }
  return regressions;
}

/**
 * Compare the native and explicitly retained Vapor routes on identical inputs.
 * Ratios are report-only until a reference-runner budget has been measured and
 * reviewed. Require the same complete fixture set on both revisions so a
 * missing lane cannot silently improve the apparent result.
 */
export function compareVaporNativePairs(
  base: CritcmpExport,
  head: CritcmpExport,
): VaporPairResult[] {
  const prefix = "vapor_native_pair/";
  // These are the seven checked-in `native_pair` inputs in the Vapor bench.
  // Changing the corpus requires an explicit comparison-contract update.
  const expectedFixtures = [
    "components",
    "control_flow",
    "events",
    "expressions",
    "spreads",
    "templates",
    "text_runs",
  ];
  const fixtureNames = (exported: CritcmpExport, side: string): string[] => {
    const fixtures = new Set<string>();
    for (const name of Object.keys(exported.benchmarks)) {
      if (!name.startsWith(prefix)) continue;
      const match = /^vapor_native_pair\/([^/]+)\/(s3|legacy)$/.exec(name);
      if (!match) throw new Error(`Unexpected Vapor pair benchmark in ${side}: ${name}`);
      fixtures.add(match[1]);
    }
    const names = [...fixtures].sort((left, right) => left.localeCompare(right));
    if (names.join("\0") !== expectedFixtures.join("\0")) {
      throw new Error(
        `Vapor native/retained fixture set in ${side} differs from the pinned corpus`,
      );
    }
    return names;
  };
  const baseFixtures = fixtureNames(base, "base");
  const headFixtures = fixtureNames(head, "head");
  if (baseFixtures.join("\0") !== headFixtures.join("\0")) {
    throw new Error("Vapor native/retained fixture sets differ between base and head");
  }
  return headFixtures.map((fixture) => {
    const median = (exported: CritcmpExport, side: string, lane: string): number => {
      const name = `${prefix}${fixture}/${lane}`;
      if (!(name in exported.benchmarks)) {
        throw new Error(`Missing Vapor pair benchmark in ${side}: ${name}`);
      }
      return medianPointEstimate(exported.benchmarks[name], `${side}/${name}`);
    };
    const baseNativeNs = median(base, "base", "s3");
    const baseRetainedNs = median(base, "base", "legacy");
    const headNativeNs = median(head, "head", "s3");
    const headRetainedNs = median(head, "head", "legacy");
    return {
      fixture,
      baseNativeNs,
      baseRetainedNs,
      headNativeNs,
      headRetainedNs,
      baseRatio: baseNativeNs / baseRetainedNs,
      headRatio: headNativeNs / headRetainedNs,
    };
  });
}

/**
 * Evaluate exact Criterion benchmark medians against conservative hard limits.
 *
 * Criterion exports point estimates in nanoseconds. Missing, duplicate, or
 * invalid budget declarations fail closed instead of silently weakening CI.
 */
export function evaluateAbsoluteBudgets(head: unknown, budgets: unknown): AbsoluteBudgetResult[] {
  if (!hasBenchmarks(head)) {
    throw new Error("Criterion absolute budgets require a parsed head export");
  }
  if (!Array.isArray(budgets)) {
    throw new Error("Criterion absolute budgets must be an array");
  }
  const names = new Set<string>();
  return budgets.map((budget: unknown) => {
    if (!isRecord(budget) || typeof budget.name !== "string" || budget.name.length === 0) {
      throw new Error("Criterion absolute budget requires a benchmark name");
    }
    if (names.has(budget.name)) {
      throw new Error(`Duplicate Criterion absolute budget: ${budget.name}`);
    }
    names.add(budget.name);
    if (
      typeof budget.maxMedianNs !== "number" ||
      !Number.isFinite(budget.maxMedianNs) ||
      budget.maxMedianNs <= 0
    ) {
      throw new Error(`Invalid Criterion absolute budget for ${budget.name}`);
    }
    if (!(budget.name in head.benchmarks)) {
      throw new Error(`Criterion absolute budget benchmark is missing: ${budget.name}`);
    }
    const medianNs = medianPointEstimate(head.benchmarks[budget.name], `head/${budget.name}`);
    return {
      name: budget.name,
      medianNs,
      maxMedianNs: budget.maxMedianNs,
      exceeded: medianNs > budget.maxMedianNs,
    };
  });
}

export function validateComparisonTable(table: string): void {
  const lines = table.split("\n").filter((line) => line.trim().length > 0);
  const columns = lines[0]?.trim().split(/\s+/);
  if (columns?.join(" ") !== "group base head") {
    throw new Error(`critcmp did not produce base/head columns: ${lines[0] ?? "empty output"}`);
  }
  if (lines.length < 3) {
    throw new Error("critcmp produced no comparison rows");
  }
}

function medianPointEstimate(benchmark: unknown, label: string): number {
  // JSON-only optional chain: a non-object step reads as undefined.
  const estimates = isRecord(benchmark) ? benchmark.criterion_estimates_v1 : undefined;
  const median = isRecord(estimates) ? estimates.median : undefined;
  const value = isRecord(median) ? median.point_estimate : undefined;
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) {
    throw new Error(`Invalid Criterion median for ${label}`);
  }
  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value != null && !Array.isArray(value);
}

function hasBenchmarks(value: unknown): value is Pick<CritcmpExport, "benchmarks"> {
  return isRecord(value) && isRecord(value.benchmarks);
}

function isCritcmpExport(value: unknown): value is CritcmpExport {
  return isRecord(value) && typeof value.name === "string" && hasBenchmarks(value);
}
