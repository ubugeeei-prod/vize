// Derived counts every rendered section quotes. Computed once from the matrix
// so the summary bullets, the tables and the cross-checks cannot disagree.

import { byKey } from "./ordering.ts";
import type { Matrix, MatrixRule } from "./rule-parity-matrix.ts";
import { CLASSIFICATIONS, META_KINDS } from "./rule-parity-paths.ts";
import { TIERS } from "./rule-parity-tiers.ts";

export type RuleStats = ReturnType<typeof summarize>;

export function summarize(matrix: Pick<Matrix, "nonRuleFiles" | "rules">) {
  const { nonRuleFiles, rules } = matrix;
  const rows = [...rules.values()].sort((a, b) => byKey(a.name, b.name));

  const count = (pred: (rule: MatrixRule) => boolean): number => rows.filter(pred).length;
  const familyCounts = new Map<string, number>();
  for (const kind of META_KINDS.values()) {
    familyCounts.set(
      kind,
      count((r) => r.family === kind),
    );
  }
  const surfaceCounts = new Map<string, number>();
  for (const r of rows) {
    for (const s of r.surfaces) surfaceCounts.set(s, (surfaceCounts.get(s) ?? 0) + 1);
  }
  const laneCounts = new Map<string, number>();
  for (const r of rows) {
    if (r.jsxLane !== "none") laneCounts.set(r.jsxLane, (laneCounts.get(r.jsxLane) ?? 0) + 1);
  }
  const classCounts = new Map<string, number>();
  for (const c of CLASSIFICATIONS)
    classCounts.set(
      c,
      count((r) => r.classification === c),
    );

  const tierCounts = new Map(TIERS.map((tier) => [tier, count((r) => r.tier === tier)]));

  return {
    rows,
    tierCounts,
    count,
    familyCounts,
    surfaceCounts,
    laneCounts,
    classCounts,
    sfcSet: rows.filter((r) => r.sfc === "yes"),
    jsxSet: rows.filter((r) => r.jsx === "yes"),
    both: rows.filter((r) => r.sfc === "yes" && r.jsx === "yes"),
    sfcOnly: rows.filter((r) => r.sfc === "yes" && r.jsx !== "yes"),
    jsxOnly: rows.filter((r) => r.sfc !== "yes" && r.jsx === "yes"),
    neither: rows.filter((r) => r.sfc !== "yes" && r.jsx !== "yes"),
    croquisUsers: rows.filter((r) => r.croquis.size > 0 || r.ctxSites > 0),
    overriddenRows: rows.filter((r) => r.overrideReason !== null),
    unregistered: rows.filter((r) => !r.registered && r.family !== "musea"),
    moduleFiles: nonRuleFiles.filter((f) => f.kind === "module"),
    testFiles: nonRuleFiles.filter((f) => f.kind === "test"),
    helperFiles: nonRuleFiles.filter((f) => f.kind === "helper"),
  };
}
