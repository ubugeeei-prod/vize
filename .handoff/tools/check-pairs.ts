import fs from "node:fs";
const [wt, rule, ...artifacts] = process.argv.slice(2);
const { pairRuleLocationDivergences } = await import(`${wt}/tools/support/fixtures/lint_divergence/records.ts`);
for (const artifact of artifacts) {
  const a = JSON.parse(fs.readFileSync(artifact, "utf8"));
  const fp = a.divergence.falsePositives.filter((r: any) => r.ruleId === rule);
  const fn = a.divergence.falseNegatives.filter((r: any) => r.ruleId === rule);
  if (fp.length + fn.length === 0) continue;
  const pairs = pairRuleLocationDivergences([...fp], [...fn]);
  console.log(a.project, "FP", fp.length, "FN", fn.length, "paired", pairs.length);
}
