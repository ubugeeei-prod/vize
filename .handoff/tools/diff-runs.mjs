// usage: node diff-runs.mjs <before-dir> <after-dir> <project>...
import fs from "node:fs";
const [before, after, ...projects] = process.argv.slice(2);
const counts = (dir, p) => {
  const a = JSON.parse(fs.readFileSync(`${dir}/${p}-lint-divergence.json`, "utf8")).divergence;
  const by = {};
  for (const [k, l] of [["FP", a.falsePositives], ["FN", a.falseNegatives]])
    for (const r of l) { by[r.ruleId] ??= { FP: 0, FN: 0 }; by[r.ruleId][k]++; }
  return { total: [a.summary.falsePositiveCount, a.summary.falseNegativeCount], by, shared: a.summary.sharedCount ?? a.shared?.length };
};
for (const p of projects) {
  let b, x;
  try { b = counts(before, p); x = counts(after, p); } catch (e) { console.log(p, "missing", e.message.slice(0, 80)); continue; }
  console.log(`${p}: FP ${b.total[0]} -> ${x.total[0]}, FN ${b.total[1]} -> ${x.total[1]}, shared ${b.shared} -> ${x.shared}`);
  const rules = new Set([...Object.keys(b.by), ...Object.keys(x.by)]);
  for (const r of [...rules].sort()) {
    const o = b.by[r] ?? { FP: 0, FN: 0 }, n = x.by[r] ?? { FP: 0, FN: 0 };
    if (o.FP !== n.FP || o.FN !== n.FN) console.log(`   ${r}: FP ${o.FP}->${n.FP} FN ${o.FN}->${n.FN}`);
  }
}
