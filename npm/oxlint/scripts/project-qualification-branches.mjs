import assert from "node:assert/strict";
import fs from "node:fs";
import { performance } from "node:perf_hooks";
import { inspect } from "node:util";

// Both branches already own real children. Join every terminal/raw outcome before
// allowing the existing always-upload step to inspect their evidence directory.
export async function runQualificationBranches(branches, output) {
  assert.equal(branches.length, 2, "the two original qualification branches only");
  assert.equal(new Set(branches.map(({ id }) => id)).size, branches.length);
  let interrupted;
  const interrupt = (signal) => (interrupted ??= signal);
  const onInterrupt = () => interrupt("SIGINT");
  const onTerminate = () => interrupt("SIGTERM");
  process.once("SIGINT", onInterrupt);
  process.once("SIGTERM", onTerminate);
  try {
    const elapsedMs = [];
    const outcomes = await Promise.allSettled(
      branches.map(async ({ run }, index) => {
        const started = performance.now();
        try {
          return await run();
        } finally {
          elapsedMs[index] = performance.now() - started;
        }
      }),
    );
    const records = outcomes.map((outcome, index) => ({
      id: branches[index].id,
      status: outcome.status,
      elapsedMs: elapsedMs[index],
      ...(outcome.status === "rejected"
        ? {
            error: inspect(outcome.reason, {
              depth: null,
              maxArrayLength: null,
              maxStringLength: null,
            }),
          }
        : {}),
    }));
    fs.writeFileSync(
      output,
      JSON.stringify({ interrupted: interrupted ?? null, branches: records }, null, 2) + "\n",
    );
    const failures = outcomes.flatMap((outcome) =>
      outcome.status === "rejected" ? [outcome.reason] : [],
    );
    if (interrupted)
      failures.push(new Error(`qualification interrupted by ${String(interrupted)}`));
    if (failures.length)
      throw new AggregateError(
        failures,
        "project qualification failed; all branch evidence retained",
      );
    return outcomes.map(({ value }) => value);
  } finally {
    process.removeListener("SIGINT", onInterrupt);
    process.removeListener("SIGTERM", onTerminate);
  }
}
