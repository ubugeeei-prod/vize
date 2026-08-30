import { fileURLToPath } from "node:url";

import { planAppE2eRows } from "./app-e2e-plan.mjs";

export function assertAppE2eAggregate({
  profile,
  suite,
  runRequired,
  planResult,
  producerResult,
  plannedCount,
}) {
  if (planResult !== "success") throw new Error(`App E2E planner is ${planResult}`);
  if (runRequired === false) {
    if (profile !== "readiness") throw new Error("Only readiness may be a successful no-op");
    if (producerResult !== "skipped") {
      throw new Error(`Irrelevant readiness producers must be skipped, got ${producerResult}`);
    }
    return { expectedCount: 0, outcome: "success" };
  }
  const expectedCount = planAppE2eRows(profile, suite).length;
  if (plannedCount !== expectedCount) {
    throw new Error(`App E2E planner emitted ${plannedCount} rows; expected ${expectedCount}`);
  }
  if (producerResult !== "success") {
    throw new Error(`App E2E producers are ${producerResult}; expected success`);
  }
  return { expectedCount, outcome: "success" };
}

function main(argv) {
  if (argv.length !== 6) {
    throw new Error(
      "Usage: app-e2e-aggregate.mjs <profile> <suite> <run-required> <plan-result> <producer-result> <planned-count>",
    );
  }
  const [profile, suite, runRequiredText, planResult, producerResult, plannedCountText] = argv;
  if (runRequiredText !== "true" && runRequiredText !== "false") {
    throw new Error(`run-required must be true or false, got ${runRequiredText}`);
  }
  const plannedCount = plannedCountText === "" ? 0 : Number(plannedCountText);
  if (!Number.isInteger(plannedCount) || plannedCount < 0) {
    throw new Error(`planned-count must be a non-negative integer, got ${plannedCountText}`);
  }
  const result = assertAppE2eAggregate({
    profile,
    suite,
    runRequired: runRequiredText === "true",
    planResult,
    producerResult,
    plannedCount,
  });
  process.stdout.write(`App E2E ${profile} aggregate: ${result.expectedCount} row(s) succeeded\n`);
}

if (process.argv[1] != null && fileURLToPath(import.meta.url) === process.argv[1]) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
