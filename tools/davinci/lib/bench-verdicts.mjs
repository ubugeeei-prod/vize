// Verdict rows for the davinci bench budget gate
// (tools/davinci/bench-compare.mjs): one row per bench id, in sorted order,
// so the stdout is a stable, diffable oracle.
//
// Gates, per registered bench
//   wall p50 — against the BASELINE report (never the budget number), with
//     the entry's tolerance applied in exact integer arithmetic (BigInt, the
//     tolerance scaled to basis points). Needs both a committed baseline
//     report and a recorded `wall_p50_ns`; without either it is report-only,
//     because a wall number is only comparable against the reference runner.
//   allocs   — against the BUDGET number, exactly: any difference fails,
//     including a change to or from null. Allocation counts are deterministic
//     AND machine-independent, so this gate does not wait for the reference
//     runner — it runs even when the wall side is report-only. `allocs = 0`
//     therefore means a measured zero, never "unrecorded".
//   rss      — report-only (see the [bench] field docs in budgets.toml).
//
// Registry drift is a failure in both directions — silent drift is the enemy:
// a budgets entry with no current result is a disappeared bench, and a result
// with no entry is an unregistered (therefore ungated) bench.

export function byBenchId(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

function formatCount(value) {
  return value === null ? "n/a" : String(value);
}

function formatRss(value) {
  return value === null ? "n/a" : `${value}B`;
}

function wallLimit(baselineP50, toleranceBp) {
  return (BigInt(baselineP50) * (10000n + BigInt(toleranceBp))) / 10000n;
}

/** The reason this bench's wall gate cannot run, or null when it can. */
function wallReportOnlyReason(budget, baseline) {
  if (baseline == null) return "no committed baseline report";
  if (budget.wall_p50_ns === 0) return "budgets.toml wall baseline not yet recorded";
  return null;
}

export function compare(budgets, baselineReports, currentReports) {
  const rows = [];
  let breaches = 0;
  let gatedOk = 0;
  let allocGated = 0;
  const ids = [
    ...new Set([...budgets.keys(), ...currentReports.keys(), ...baselineReports.keys()]),
  ].sort(byBenchId);
  for (const id of ids) {
    const budget = budgets.get(id);
    const current = currentReports.get(id);
    const baseline = baselineReports.get(id);
    if (budget == null) {
      const where = current != null ? "current" : "baseline";
      rows.push(
        `FAIL ${id} unregistered bench (${where} result has no budgets.toml [bench] entry)`,
      );
      breaches += 1;
      continue;
    }
    if (current == null) {
      rows.push(`FAIL ${id} bench disappeared (budgets.toml entry has no current result)`);
      breaches += 1;
      continue;
    }
    const benchRows = [];
    const wallSkipped = wallReportOnlyReason(budget, baseline);
    let limit = null;
    if (wallSkipped == null) {
      limit = wallLimit(baseline.wall_ns.p50, budget.toleranceBp);
      if (BigInt(current.wall_ns.p50) > limit) {
        benchRows.push(
          `FAIL ${id} wall_p50 ${current.wall_ns.p50}ns > limit ${limit}ns ` +
            `(baseline ${baseline.wall_ns.p50}ns + ${budget.toleranceBp / 100}% tolerance)`,
        );
      }
    }
    if (current.allocs !== budget.allocs) {
      benchRows.push(
        `FAIL ${id} allocs ${formatCount(budget.allocs)} -> ${formatCount(current.allocs)} ` +
          `(exact gate against budgets.toml: allocs are deterministic and machine-independent)`,
      );
    }
    if (budget.allocBytesPeak !== undefined && current.alloc_bytes_peak !== budget.allocBytesPeak) {
      benchRows.push(
        `FAIL ${id} alloc_bytes_peak ${formatCount(budget.allocBytesPeak)} -> ` +
          `${formatCount(current.alloc_bytes_peak)} (exact deterministic peak gate)`,
      );
    }
    const peak =
      budget.allocBytesPeak === undefined
        ? ""
        : ` alloc_bytes_peak ${formatCount(current.alloc_bytes_peak)}B`;
    if (benchRows.length > 0) {
      rows.push(...benchRows);
      breaches += benchRows.length;
    } else if (wallSkipped != null) {
      rows.push(
        `alloc-gated ${id} allocs ${formatCount(current.allocs)}${peak} ok ` +
          `(wall_p50 ${current.wall_ns.p50}ns report-only: ${wallSkipped}) ` +
          `rss ${formatRss(current.rss_peak_bytes)}`,
      );
      allocGated += 1;
    } else {
      rows.push(
        `ok ${id} wall_p50 ${current.wall_ns.p50}ns ` +
          `(baseline ${baseline.wall_ns.p50}ns limit ${limit}ns) ` +
          `allocs ${formatCount(current.allocs)}${peak} rss ${formatRss(current.rss_peak_bytes)}`,
      );
      gatedOk += 1;
    }
  }
  return { rows, breaches, gatedOk, allocGated };
}

export function reconciliationOnly(budgets, currentReports) {
  const rows = [];
  const ids = [...new Set([...budgets.keys(), ...currentReports.keys()])].sort(byBenchId);
  for (const id of ids) {
    if (!budgets.has(id)) {
      rows.push(`FAIL ${id} unregistered bench (current result has no budgets.toml [bench] entry)`);
    } else if (!currentReports.has(id)) {
      rows.push(`FAIL ${id} bench disappeared (budgets.toml entry has no current result)`);
    }
  }
  return rows;
}
