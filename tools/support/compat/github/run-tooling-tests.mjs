import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolingTestFiles } from "./plan-tooling-tests.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));

export function toolingTestCommand(plan, available = toolingTestFiles()) {
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
  // Shared fixture/report paths prohibit file-level process concurrency.
  return ["--test", "--test-concurrency=1", ...plan.tests];
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const planPath =
    process.argv[2] ?? process.env.VIZE_TOOLING_TEST_PLAN ?? "target/tooling-test-plan.json";
  const plan = JSON.parse(readFileSync(planPath, "utf8"));
  const args = toolingTestCommand(plan);
  if (plan.tests.length > 0) {
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
