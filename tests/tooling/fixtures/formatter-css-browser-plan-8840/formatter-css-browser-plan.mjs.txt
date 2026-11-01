import { appendFileSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolingTestFiles } from "./plan-tooling-tests.mjs";
import { toolingTestCommand } from "./run-tooling-tests.mjs";

const browserTest = "tests/tooling/formatter-css-rule-layout.test.mjs";

export function needsFormatterCssBrowser(plan, available, shard, tier) {
  if (!["pr", "merge"].includes(tier) || plan.tier !== tier) {
    throw new Error("CSS browser preparation does not match the execution tier");
  }
  return toolingTestCommand(plan, available, shard).slice(2).includes(browserTest);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const plan = JSON.parse(readFileSync(process.env.VIZE_TOOLING_TEST_PLAN, "utf8"));
  const required = needsFormatterCssBrowser(
    plan,
    toolingTestFiles(),
    process.env.VIZE_TOOLING_TEST_SHARD,
    process.env.VIZE_TOOLING_TEST_TIER,
  );
  appendFileSync(process.env.GITHUB_OUTPUT, `required=${required}\n`);
}
