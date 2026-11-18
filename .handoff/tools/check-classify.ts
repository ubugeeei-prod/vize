// usage: node check-classify.ts <worktree> <fixtures-worktree> <rule-regex> <artifact.json>...
import fs from "node:fs";
const [wt, fixtures, ruleRegex, ...artifacts] = process.argv.slice(2);
const { patinaIntentionalReason, baselineIntentionalReason } = await import(`${wt}/tools/support/fixtures/lint_divergence/records.ts`);
const registry = JSON.parse(fs.readFileSync(`${fixtures}/tests/_fixtures/vue-ecosystem-fixtures.json`, "utf8"));
const re = new RegExp(ruleRegex);
const counts: Record<string, number> = {};
for (const artifact of artifacts) {
  const a = JSON.parse(fs.readFileSync(artifact, "utf8"));
  const p = a.project;
  const root = registry.projects.find((entry: any) => entry.id === p).fixturePath;
  for (const [kind, list] of [["FP", a.divergence.falsePositives], ["FN", a.divergence.falseNegatives]] as const) {
    for (const r of list) {
      if (!re.test(r.ruleId)) continue;
      let src: string;
      try { src = fs.readFileSync(`${fixtures}/${root}/${r.file}`, "utf8"); } catch { counts[`${p} ${kind} missing`] = (counts[`${p} ${kind} missing`] ?? 0) + 1; continue; }
      const record = kind === "FN" ? { ...r, ruleId: r.upstreamRuleId ?? r.ruleId } : r;
      const reason = kind === "FP" ? patinaIntentionalReason(record, src, p) : baselineIntentionalReason(record, src);
      const key = `${p} ${kind} ${reason === null ? "counted" : "classified"}`;
      counts[key] = (counts[key] ?? 0) + 1;
    }
  }
}
console.log(counts);
