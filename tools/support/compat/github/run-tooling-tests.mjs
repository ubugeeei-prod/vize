import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolingTestFiles } from "./plan-tooling-tests.mjs";
import { TOOLING_TIMINGS_LOG_MARKER, selectToolingShard } from "./tooling-test-shards.ts";

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

export function toolingTestObservation(args, env = process.env) {
  if (
    env.GITHUB_ACTIONS !== "true" ||
    !env.RUNNER_TEMP ||
    !args.includes("tests/tooling/empty-vue-star-listing-source.test.mjs")
  )
    return null;
  const directory = join(env.RUNNER_TEMP, "tooling-listing-diagnostics");
  return {
    directory,
    args: [
      ...args.slice(0, 2),
      "--test-reporter=spec",
      "--test-reporter-destination=stdout",
      "--test-reporter=tap",
      `--test-reporter-destination=${join(directory, "tests.tap")}`,
      ...args.slice(2),
    ],
  };
}

export function toolingMemoryObservation(read = (file) => readFileSync(file, "utf8")) {
  return Object.fromEntries(
    [
      "/proc/self/cgroup",
      "/sys/fs/cgroup/memory.events",
      "/sys/fs/cgroup/memory.events.local",
      "/sys/fs/cgroup/memory.current",
      "/sys/fs/cgroup/memory.max",
    ].map((file) => {
      try {
        return [file, { wholeUtf8: read(file) }];
      } catch (error) {
        return [file, { error: { code: error.code, message: error.message } }];
      }
    }),
  );
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
    // The spec reporter output is unchanged. A second reporter records each file's wall time,
    // which update-tooling-test-timings.ts reads back from the job log.
    const observation = toolingTestObservation(args, env);
    if (observation) {
      mkdirSync(observation.directory, { recursive: true });
      env.VIZE_TOOLING_LISTING_OBSERVATION_DIR = observation.directory;
    }
    const startedAt = new Date().toISOString();
    const memoryBefore = observation ? toolingMemoryObservation() : null;
    const timings = mkdtempSync(join(tmpdir(), "vize-tooling-timings-"));
    const destination = join(timings, "files.json");
    try {
      const timingReporter = [
        `--test-reporter=${fileURLToPath(new URL("./tooling-test-timing-reporter.ts", import.meta.url))}`,
        `--test-reporter-destination=${destination}`,
      ];
      // An observed command already carries the spec reporter beside its TAP copy.
      const command = observation
        ? [...timingReporter, ...observation.args]
        : [
            "--test-reporter=spec",
            "--test-reporter-destination=stdout",
            ...timingReporter,
            ...args,
          ];
      const result = spawnSync(process.execPath, command, { cwd: root, env, stdio: "inherit" });
      if (observation)
        writeFileSync(
          join(observation.directory, "outer-result.json"),
          JSON.stringify(
            {
              source: env.GITHUB_SHA,
              run: env.GITHUB_RUN_ID,
              attempt: env.GITHUB_RUN_ATTEMPT,
              job: env.GITHUB_JOB,
              shard: env.VIZE_TOOLING_TEST_SHARD,
              node: process.execPath,
              version: process.version,
              nodeTestContext: env.NODE_TEST_CONTEXT ?? null,
              args: observation.args,
              originalArgs: args,
              cwd: root,
              startedAt,
              completedAt: new Date().toISOString(),
              memoryBefore,
              memoryAfter: toolingMemoryObservation(),
              status: result.status,
              signal: result.signal,
              error: result.error
                ? {
                    name: result.error.name,
                    message: result.error.message,
                    code: result.error.code,
                  }
                : null,
            },
            null,
            2,
          ) + "\n",
        );
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
