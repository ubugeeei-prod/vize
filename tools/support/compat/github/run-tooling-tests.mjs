import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolingTestFiles } from "./plan-tooling-tests.mjs";
import { mergeToolingShardTests, mergeToolingShardCount } from "./tooling-merge-shards.mjs";
import { pureToolingTests } from "../../../config/vite-plus/tooling-test-scopes.ts";

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

export function prToolingCohortCommand(plan, cohort, available = toolingTestFiles()) {
  if (plan.tier !== "pr" || !["pure", "full"].includes(cohort)) {
    throw new Error("PR tooling cohort must be pure or full");
  }
  toolingTestCommand(plan, available);
  const pure = plan.pureTests;
  const allowedPure = new Set(pureToolingTests);
  if (
    !Array.isArray(pure) ||
    new Set(pure).size !== pure.length ||
    pure.some((file) => !plan.tests.includes(file) || !allowedPure.has(file))
  ) {
    throw new Error("invalid pure tooling cohort in PR plan");
  }
  const pureSet = new Set(pure);
  const selected = plan.tests.filter((file) => pureSet.has(file) === (cohort === "pure"));
  return ["--test", "--test-concurrency=1", ...selected];
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
    const args = ["--test", "--test-concurrency=1", ...tests];
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
    const cohort = process.argv[2] === "--pr-cohort" ? process.argv[3] : undefined;
    if (cohort && process.argv.length !== 4) throw new Error("unexpected PR cohort arguments");
    const planPath = cohort
      ? (process.env.VIZE_TOOLING_TEST_PLAN ?? "target/tooling-test-plan.json")
      : (process.argv[2] ?? process.env.VIZE_TOOLING_TEST_PLAN ?? "target/tooling-test-plan.json");
    const plan = JSON.parse(readFileSync(planPath, "utf8"));
    const args = cohort ? prToolingCohortCommand(plan, cohort) : toolingTestCommand(plan);
    if (args.length > 2) {
      if (
        cohort === "full" &&
        (process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD !== "1" || !process.env.VIZE_LSP_BIN)
      ) {
        throw new Error("full PR tooling cohort requires a source-built CLI");
      }
      // Selection defers known scenarios. An unclassified new requirement must
      // still fail closed; disabling the requirement would hide missing coverage.
      const env = { ...process.env, VIZE_TEST_REQUIRE_TSGO: "1" };
      const result = spawnSync(process.execPath, args, { cwd: root, env, stdio: "inherit" });
      if (result.error) throw result.error;
      process.exitCode = result.status ?? 1;
    } else {
      process.stdout.write("No tooling tests selected for this plan or cohort.\n");
    }
  }
}
