import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolingTestFiles } from "./plan-tooling-tests.mjs";
import { selectToolingShard } from "./tooling-test-shards.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));

export function toolingTestCommand(plan, available = toolingTestFiles(), shard = "") {
  if (plan.version !== 1 || !["pr", "merge"].includes(plan.tier) || !Array.isArray(plan.tests)) {
    throw new Error("invalid tooling test plan");
  }
  const allowed = new Set(available);
  if (
    new Set(plan.tests).size !== plan.tests.length ||
    plan.tests.some((file) => !allowed.has(file))
  ) {
    throw new Error("tooling plan contains duplicate, missing, or unrecognized test files");
  }
  if (plan.tier === "merge" && plan.tests.length !== available.length) {
    throw new Error("merge tooling plan must retain every test file");
  }
  // Each isolated runner remains serial because fixtures and reports are shared
  // between test files inside a checkout. Merge plans must contain the whole
  // discovered suite before its files can be partitioned across checkouts.
  return ["--test", "--test-concurrency=1", ...selectToolingShard(plan, shard)];
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const planPath =
    process.argv[2] ?? process.env.VIZE_TOOLING_TEST_PLAN ?? "target/tooling-test-plan.json";
  const plan = JSON.parse(readFileSync(planPath, "utf8"));
  const requiredTier = process.env.VIZE_TOOLING_TEST_TIER;
  if (requiredTier && (!["pr", "merge"].includes(requiredTier) || plan.tier !== requiredTier)) {
    throw new Error("tooling plan does not match the required execution tier");
  }
  const args = toolingTestCommand(plan, toolingTestFiles(), process.env.VIZE_TOOLING_TEST_SHARD);
  if (args.length > 2) {
    // Selection defers known scenarios. An unclassified new requirement must
    // still fail closed; disabling the requirement would hide missing coverage.
    const env = { ...process.env, VIZE_TEST_REQUIRE_TSGO: "1" };
    const result = spawnSync(process.execPath, args, { cwd: root, env, stdio: "inherit" });
    if (result.error) throw result.error;
    process.exitCode = result.status ?? 1;
  } else {
    process.stdout.write("No tooling tests selected by the declared inputs.\n");
  }
}
