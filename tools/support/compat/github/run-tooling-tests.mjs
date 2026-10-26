import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolingTestFiles } from "./plan-tooling-tests.mjs";
import { TOOLING_TIMINGS_LOG_MARKER, selectToolingShard } from "./tooling-test-shards.ts";
import { prepareCssCustodyHistory } from "./prepare-css-custody-history.mjs";

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
    const history = prepareCssCustodyHistory(root, args.slice(2));
    if (history.revisions.length) {
      process.stdout.write(`CSS custody history: ${JSON.stringify(history)}\n`);
    }
    // Selection defers known scenarios. An unclassified new requirement must
    // still fail closed; disabling the requirement would hide missing coverage.
    const env = { ...process.env, VIZE_TEST_REQUIRE_TSGO: "1" };
    // The spec reporter output is unchanged. A second reporter records each file's wall time,
    // which update-tooling-test-timings.ts reads back from the job log.
    const timings = mkdtempSync(join(tmpdir(), "vize-tooling-timings-"));
    const destination = join(timings, "files.json");
    try {
      const reporters = [
        "--test-reporter=spec",
        "--test-reporter-destination=stdout",
        `--test-reporter=${fileURLToPath(new URL("./tooling-test-timing-reporter.ts", import.meta.url))}`,
        `--test-reporter-destination=${destination}`,
      ];
      const result = spawnSync(process.execPath, [...reporters, ...args], {
        cwd: root,
        env,
        stdio: "inherit",
      });
      if (result.error) throw result.error;
      process.exitCode = result.status ?? 1;
      try {
        const files = JSON.parse(readFileSync(destination, "utf8"));
        const shard = process.env.VIZE_TOOLING_TEST_SHARD ?? "";
        const record = JSON.stringify({ tier: plan.tier, shard, files });
        process.stdout.write(`\n${TOOLING_TIMINGS_LOG_MARKER} ${record}\n`);
      } catch (error) {
        // Timing evidence is advisory; it never changes the test result.
        process.stdout.write(`\nTooling test timings unavailable: ${error.message}\n`);
      }
    } finally {
      rmSync(timings, { recursive: true, force: true });
    }
  } else {
    process.stdout.write("No tooling tests selected by the declared inputs.\n");
  }
}
