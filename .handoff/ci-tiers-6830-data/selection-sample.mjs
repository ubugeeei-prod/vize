// node selection-sample.mjs <repo> <out.json> <commit>... : PR-tier tooling selection per commit diff
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
const [repo, out, ...commits] = process.argv.slice(2);
const planner = await import(pathToFileURL(`${repo}/tools/support/compat/github/plan-tooling-tests.mjs`).href);
const shards = await import(pathToFileURL(`${repo}/tools/support/compat/github/tooling-test-shards.ts`).href);
const result = {};
for (const commit of commits) {
  const paths = execFileSync("git", ["-C", repo, "diff", "--name-only", `${commit}^`, commit], { encoding: "utf8" })
    .trim().split("\n").filter(Boolean);
  const plan = planner.planToolingTests(paths, { tier: "pr", cwd: repo });
  const shardList = shards.toolingShards({ ...plan, leadSteps: [] });
  result[commit] = { paths: paths.length, tests: plan.tests, shards: shardList.length,
    seconds: Math.round(shardList.reduce((a, s) => a + s.testMs, 0) / 1000) };
  console.log(commit.slice(0, 10), paths.length, plan.tests.length, shardList.length, result[commit].seconds);
}
writeFileSync(out, JSON.stringify(result));
