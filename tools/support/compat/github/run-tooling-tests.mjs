import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolingTestFiles } from "./plan-tooling-tests.mjs";
import { mergeToolingShardTests, mergeToolingShardCount } from "./tooling-merge-shards.mjs";

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
  if (process.argv[2] === "--merge-shard") {
    if (
      process.argv.length !== 3 ||
      process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD !== "1" ||
      !process.env.VIZE_LSP_BIN
    ) {
      throw new Error("merge tooling shards require a source-built CLI and no extra arguments");
    }
    const available = toolingTestFiles();
    const shardId = Number(process.env.VIZE_TOOLING_MERGE_SHARD);
    const tests = mergeToolingShardTests(available, shardId);
    // Matrix jobs have separate checkouts; tests remain serial within each.
    const reporters = process.env.VIZE_TOOLING_TIMING_FILE
      ? [
          "--test-reporter=spec",
          "--test-reporter=./tools/support/compat/github/tooling-file-timing-reporter.mjs",
          "--test-reporter-destination=stdout",
          `--test-reporter-destination=${process.env.VIZE_TOOLING_TIMING_FILE}`,
        ]
      : [];
    const args = ["--test", "--test-concurrency=1", ...reporters, ...tests];
    process.stdout.write(
      `Merge tooling shard ${shardId}/${mergeToolingShardCount}: ${tests.length}/${available.length} files.\n`,
    );
    const result = spawnSync(process.execPath, args, {
      cwd: root,
      env: process.env,
      stdio: "inherit",
    });
    if (result.error) throw result.error;
    process.exitCode = result.status ?? 1;
  } else {
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
}
