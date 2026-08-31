// Derived counts every rendered section quotes. Computed once from the matrix
// so the summary bullets, the tables and the cross-checks cannot disagree.

import { byKey } from "./ordering.mjs";
import { CLASSIFICATIONS, META_KINDS } from "./rule-parity-paths.mjs";

export function summarize(matrix) {
  const { nonRuleFiles, rules } = matrix;
  const rows = [...rules.values()].sort((a, b) => byKey(a.name, b.name));

  const count = (pred) => rows.filter(pred).length;
  const familyCounts = new Map();
  for (const kind of META_KINDS.values()) {
    familyCounts.set(
      kind,
      count((r) => r.family === kind),
    );
  }
  const surfaceCounts = new Map();
  for (const r of rows) {
    for (const s of r.surfaces) surfaceCounts.set(s, (surfaceCounts.get(s) ?? 0) + 1);
  }
  const laneCounts = new Map();
  for (const r of rows) {
    if (r.jsxLane !== "none") laneCounts.set(r.jsxLane, (laneCounts.get(r.jsxLane) ?? 0) + 1);
  }
  const classCounts = new Map();
  for (const c of CLASSIFICATIONS)
    classCounts.set(
      c,
      count((r) => r.classification === c),
    );

  return {
    rows,
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
