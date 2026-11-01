import { appendFileSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolingTestFiles } from "./plan-tooling-tests.mjs";
import { toolingTestCommand } from "./run-tooling-tests.mjs";

const browserTest = "tests/tooling/formatter-css-rule-layout.test.mjs";

// These erased boundary views retain the original JS property access and the
// existing callee's validation/errors. They do not certify parsed input.
type TierView = { readonly tier?: unknown };
type TestCommand = (plan: unknown, available: unknown, shard: unknown) => unknown[];
type ReadUtf8 = (file: unknown, encoding: "utf8") => string;
type AppendText = (file: unknown, text: string) => void;

export function needsFormatterCssBrowser(
  plan: unknown,
  available: unknown,
  shard: unknown,
  tier: unknown,
): boolean {
  if (!(["pr", "merge"] as readonly unknown[]).includes(tier) || (plan as TierView).tier !== tier) {
    throw new Error("CSS browser preparation does not match the execution tier");
  }
  return (toolingTestCommand as TestCommand)(plan, available, shard).slice(2).includes(browserTest);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const plan: unknown = JSON.parse(
    (readFileSync as ReadUtf8)(process.env.VIZE_TOOLING_TEST_PLAN, "utf8"),
  );
  const required = needsFormatterCssBrowser(
    plan,
    toolingTestFiles(),
    process.env.VIZE_TOOLING_TEST_SHARD,
    process.env.VIZE_TOOLING_TEST_TIER,
  );
  (appendFileSync as AppendText)(process.env.GITHUB_OUTPUT, `required=${required}\n`);
}
