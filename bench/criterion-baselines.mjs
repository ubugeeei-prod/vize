export function critcmpExportArgs({ targetDir, baseline }) {
  return ["--target-dir", targetDir, "--export", baseline];
}

export function criterionEnvironment(targetDir) {
  return { CARGO_TARGET_DIR: targetDir };
}

export function critcmpArgs({ targetDir, baselinePaths }) {
  return ["--target-dir", targetDir, ...baselinePaths];
}

export function parseCritcmpExport(serialized, expectedBaseline) {
  const parsed = JSON.parse(serialized);
  if (!isRecord(parsed) || parsed.name !== expectedBaseline || !isRecord(parsed.benchmarks)) {
    throw new Error(`Invalid critcmp export for ${expectedBaseline}`);
  }
  if (Object.keys(parsed.benchmarks).length === 0) {
    throw new Error(`critcmp export for ${expectedBaseline} contains no benchmarks`);
  }
  return parsed;
}

export function compareBaselineExports(base, head, thresholdPercent) {
  const shared = Object.keys(base.benchmarks).filter((name) => name in head.benchmarks);
  if (shared.length === 0) {
    throw new Error("Criterion base and head exports contain no shared benchmarks");
  }
  if (thresholdPercent == null) {
    return [];
  }

  const regressions = [];
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
 * Evaluate exact Criterion benchmark medians against conservative hard limits.
 *
 * Criterion exports point estimates in nanoseconds. Missing, duplicate, or
 * invalid budget declarations fail closed instead of silently weakening CI.
 */
export function evaluateAbsoluteBudgets(head, budgets) {
  if (!isRecord(head) || !isRecord(head.benchmarks)) {
    throw new Error("Criterion absolute budgets require a parsed head export");
  }
  if (!Array.isArray(budgets)) {
    throw new Error("Criterion absolute budgets must be an array");
  }
  const names = new Set();
  return budgets.map((budget) => {
    if (!isRecord(budget) || typeof budget.name !== "string" || budget.name.length === 0) {
      throw new Error("Criterion absolute budget requires a benchmark name");
    }
    if (names.has(budget.name)) {
      throw new Error(`Duplicate Criterion absolute budget: ${budget.name}`);
    }
    names.add(budget.name);
    if (!Number.isFinite(budget.maxMedianNs) || budget.maxMedianNs <= 0) {
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

export function validateComparisonTable(table) {
  const lines = table.split("\n").filter((line) => line.trim().length > 0);
  const columns = lines[0]?.trim().split(/\s+/);
  if (columns?.join(" ") !== "group base head") {
    throw new Error(`critcmp did not produce base/head columns: ${lines[0] ?? "empty output"}`);
  }
  if (lines.length < 3) {
    throw new Error("critcmp produced no comparison rows");
  }
}

function medianPointEstimate(benchmark, label) {
  const value = benchmark?.criterion_estimates_v1?.median?.point_estimate;
  if (!Number.isFinite(value) || value <= 0) {
    throw new Error(`Invalid Criterion median for ${label}`);
  }
  return value;
}

function isRecord(value) {
  return typeof value === "object" && value != null && !Array.isArray(value);
}
