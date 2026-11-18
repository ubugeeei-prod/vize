// usage: sum-runs.mjs <dirA> <dirB> : per-project FP/FN and rule deltas between two artifact trees
import fs from "node:fs";
import path from "node:path";
const load = (dir) => {
  const out = new Map();
  const walk = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) walk(p);
      else if (e.name.endsWith("-lint-divergence.json")) {
        const a = JSON.parse(fs.readFileSync(p, "utf8"));
        const rules = {};
        for (const [k, l] of [["FP", a.divergence.falsePositives], ["FN", a.divergence.falseNegatives]])
          for (const r of l) { rules[r.ruleId] ??= { FP: 0, FN: 0 }; rules[r.ruleId][k]++; }
        out.set(a.project, { FP: a.divergence.summary.falsePositiveCount, FN: a.divergence.summary.falseNegativeCount, rules });
      }
    }
  };
  walk(dir);
  return out;
};
const [a, b] = [load(process.argv[2]), load(process.argv[3])];
let ta = { FP: 0, FN: 0 }, tb = { FP: 0, FN: 0 };
const ruleDelta = {};
for (const [p, B] of b) {
  const A = a.get(p);
  if (!A) { console.log(`${p}: only in B`); continue; }
  ta.FP += A.FP; ta.FN += A.FN; tb.FP += B.FP; tb.FN += B.FN;
  for (const r of new Set([...Object.keys(A.rules), ...Object.keys(B.rules)])) {
    const x = A.rules[r] ?? { FP: 0, FN: 0 }, y = B.rules[r] ?? { FP: 0, FN: 0 };
    ruleDelta[r] ??= { FP: 0, FN: 0, FPa: 0, FNa: 0 };
    ruleDelta[r].FP += y.FP - x.FP; ruleDelta[r].FN += y.FN - x.FN; ruleDelta[r].FPa += x.FP; ruleDelta[r].FNa += x.FN;
  }
  if (A.FP !== B.FP || A.FN !== B.FN) console.log(`${p}: FP ${A.FP}->${B.FP} FN ${A.FN}->${B.FN}`);
}
console.log(`TOTAL (common projects) FP ${ta.FP}->${tb.FP} FN ${ta.FN}->${tb.FN}`);
for (const [r, d] of Object.entries(ruleDelta).filter(([, d]) => d.FP || d.FN).sort((x, y) => (x[1].FP + x[1].FN) - (y[1].FP + y[1].FN)))
  console.log(`  ${r}: FP ${d.FPa}${d.FP >= 0 ? "+" : ""}${d.FP} FN ${d.FNa}${d.FN >= 0 ? "+" : ""}${d.FN}`);
