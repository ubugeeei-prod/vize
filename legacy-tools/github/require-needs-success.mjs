import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * Jobs in the `test-report` dependency list that skip on every pull request by
 * design, mapped to the reason they skip.
 *
 * `test-report` itself only runs on `pull_request`, so this is the complete set
 * of legitimate skips: both entries are gated by
 * `if: ${{ github.event_name != 'pull_request' }}` in `.github/workflows/check.yml`
 * and therefore never run in the event that `test-report` aggregates.
 *
 * Every other needed job runs on pull requests, so a `skipped` result there means
 * the job never ran at all — `playground-test`, for instance, is skipped when the
 * `build-js-packages` job it needs fails. Those skips are failures for gating
 * purposes: the work they were meant to gate was not done.
 *
 * `tests/tooling/github-workflows-check-gate.test.ts` re-derives this list from
 * the workflow's `if:` guards, so adding a pull-request-skipping job to
 * `test-report`'s `needs:` fails that test until the job is classified here.
 */
export const PULL_REQUEST_SKIPPED_JOBS = Object.freeze({
  "nix-flake": "runs on push and schedule only",
  "source-coverage": "runs on push and schedule only",
});

function sortedEntries(needs) {
  return Object.entries(needs).sort(([left], [right]) =>
    left < right ? -1 : left > right ? 1 : 0,
  );
}

function formatNames(names) {
  return names.join(", ");
}

/**
 * Decides whether the `test-report` aggregator may report success.
 *
 * `test-report` runs with `if: always()` so that it still collects the test
 * inventory when a dependency fails. Without this check its own steps succeed
 * and the required status check reports success while any of the jobs it
 * aggregates is red, which makes the whole check suite advisory.
 *
 * @param {Record<string, { result?: string }>} needs Parsed `toJSON(needs)` payload.
 * @param {Record<string, string>} skippableJobs Jobs allowed to report `skipped`.
 * @returns {{ exitCode: number, message: string }}
 */
export function aggregateNeedsResults(needs, skippableJobs = PULL_REQUEST_SKIPPED_JOBS) {
  if (needs === null || typeof needs !== "object" || Array.isArray(needs)) {
    throw new Error("The needs context must be an object of job results");
  }
  const entries = sortedEntries(needs);
  if (entries.length === 0) {
    throw new Error("The needs context is empty: test-report must depend on the jobs it gates");
  }

  const succeeded = [];
  const skippedByDesign = [];
  const unresolved = [];
  for (const [job, value] of entries) {
    const result = value === null || typeof value !== "object" ? undefined : value.result;
    if (typeof result !== "string" || result === "") {
      throw new Error(`Job ${job} reported no result in the needs context`);
    }
    if (result === "success") {
      succeeded.push(job);
    } else if (result === "skipped" && Object.hasOwn(skippableJobs, job)) {
      skippedByDesign.push(job);
    } else {
      unresolved.push(`  - ${job}: ${result}`);
    }
  }

  const allowed = formatNames(Object.keys(skippableJobs).sort());
  if (unresolved.length > 0) {
    return {
      exitCode: 1,
      message: [
        `test-report gate: ${unresolved.length} of ${entries.length} needed jobs did not succeed.`,
        ...unresolved,
        "test-report is a required status check, so it must not pass while a job it aggregates is red.",
        `Only these jobs may skip on a pull request: ${allowed}.`,
      ].join("\n"),
    };
  }

  const skipped =
    skippedByDesign.length === 0
      ? "0 skipped on a pull request by design."
      : `${skippedByDesign.length} skipped on a pull request by design: ${formatNames(skippedByDesign)}.`;
  return {
    exitCode: 0,
    message: `test-report gate: all ${entries.length} needed jobs are accounted for. ${succeeded.length} succeeded; ${skipped}`,
  };
}

function main() {
  const raw = process.env.NEEDS_JSON;
  if (!raw) {
    console.error(
      "NEEDS_JSON is required: pass ${{ toJSON(needs) }} to tools/commands/ci/github/require-needs-success.rs",
    );
    process.exitCode = 1;
    return;
  }
  const { exitCode, message } = aggregateNeedsResults(JSON.parse(raw));
  if (exitCode === 0) {
    console.log(message);
    return;
  }
  console.error(message);
  process.exitCode = exitCode;
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  main();
}
