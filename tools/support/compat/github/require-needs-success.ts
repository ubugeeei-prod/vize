import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * All jobs in the PR report's dependency list must run. A skipped job is a
 * missing check and fails the report.
 */
export const PULL_REQUEST_SKIPPED_JOBS = Object.freeze({});

export type GateReport = { exitCode: 0 | 1; message: string };

function sortedEntries(needs: object): Array<[string, unknown]> {
  return Object.entries(needs).sort(([left], [right]) =>
    left < right ? -1 : left > right ? 1 : 0,
  );
}

function formatNames(names: string[]): string {
  return names.join(", ");
}

/**
 * Decides whether the `test-report` aggregator may report success.
 *
 * `test-report` runs with `if: always()` so that it reports failed dependencies.
 * Without this check its own steps succeed
 * and the required status check reports success while any of the jobs it
 * aggregates is red, which makes the whole check suite advisory.
 *
 * @param {Record<string, { result?: string }>} needs Parsed `toJSON(needs)` payload.
 * @param {Record<string, string>} skippableJobs Jobs allowed to report `skipped`.
 * @returns {{ exitCode: number, message: string }}
 */
export function aggregateNeedsResults(
  needs: unknown,
  skippableJobs: Readonly<Record<string, unknown>> = PULL_REQUEST_SKIPPED_JOBS,
): GateReport {
  if (needs === null || typeof needs !== "object" || Array.isArray(needs)) {
    throw new Error("The needs context must be an object of job results");
  }
  const entries = sortedEntries(needs);
  if (entries.length === 0) {
    throw new Error("The needs context is empty: test-report must depend on the jobs it gates");
  }

  const succeeded: string[] = [];
  const skippedByDesign: string[] = [];
  const unresolved: string[] = [];
  for (const [job, value] of entries) {
    const result =
      value === null || typeof value !== "object"
        ? undefined
        : (value as Record<string, unknown>).result;
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

/** Queue-only product qualifications must succeed before the required Check report. */
export function aggregateCheckNeedsResults(needs: unknown, eventName: unknown): GateReport {
  const nonqueue = ["pull_request", "push", "schedule", "workflow_dispatch"];
  if (
    eventName !== "merge_group" &&
    (typeof eventName !== "string" || !nonqueue.includes(eventName))
  ) {
    // Template interpolation rejects primitive symbols; String otherwise uses
    // the same string-hint coercion without introducing an unknown-value warning.
    if (typeof eventName === "symbol")
      throw new TypeError("Cannot convert a Symbol value to a string");
    throw new Error(`Unsupported Check event: ${String(eventName)}`);
  }
  const queueOnly = ["wit-contracts", "nuxt-module-qualification"];
  for (const job of queueOnly) {
    if (
      !needs ||
      (typeof needs !== "object" && typeof needs !== "function") ||
      !Object.hasOwn(needs, job)
    ) {
      throw new Error(`Check needs context is missing ${job}`);
    }
  }
  const skippable =
    eventName === "merge_group"
      ? {}
      : Object.fromEntries(queueOnly.map((job) => [job, "queue-only"]));
  return aggregateNeedsResults(needs, skippable);
}

/** The outer Check report requires measured ceilings in the queue and full qualification. */
export function aggregateCheckReportResults(needs: unknown, eventName: unknown): GateReport {
  if (
    !["pull_request", "merge_group", "schedule", "workflow_dispatch"].includes(eventName as string)
  ) {
    throw new Error("Unsupported Check report event");
  }
  if (!needs || typeof needs !== "object" || !Object.hasOwn(needs, "instruction-counts")) {
    throw new Error("Check report needs context is missing instruction-counts");
  }
  return aggregateNeedsResults(
    needs,
    eventName === "pull_request"
      ? { "instruction-counts": "queue and full qualification only" }
      : {},
  );
}

/** Full qualification requires both the exhaustive package matrix and both-host project replay. */
export function aggregateFullJsPackageResults(needs: unknown): GateReport {
  const expected = ["full-js-original-project", "full-js-package-tests"];
  if (
    !needs ||
    typeof needs !== "object" ||
    Array.isArray(needs) ||
    Object.keys(needs).length !== expected.length ||
    expected.some((job) => !Object.hasOwn(needs, job))
  ) {
    throw new Error(
      "Full JS qualification requires exactly the package matrix and original-project jobs",
    );
  }
  return aggregateNeedsResults(needs);
}

function main(): void {
  const args = process.argv.slice(2);
  if (
    args.length > 1 ||
    (args.length === 1 && !["--check", "--check-report", "--full-js-packages"].includes(args[0]))
  ) {
    throw new Error("Unknown report mode");
  }
  const raw = process.env.NEEDS_JSON;
  if (!raw) {
    console.error(
      "NEEDS_JSON is required: pass ${{ toJSON(needs) }} to tools/support/compat/github/require-needs-success.ts",
    );
    process.exitCode = 1;
    return;
  }
  const results: unknown = JSON.parse(raw);
  const { exitCode, message } =
    args[0] === "--full-js-packages"
      ? aggregateFullJsPackageResults(results)
      : args[0] === "--check-report"
        ? aggregateCheckReportResults(results, process.env.GITHUB_EVENT_NAME)
        : args[0] === "--check"
          ? aggregateCheckNeedsResults(results, process.env.GITHUB_EVENT_NAME)
          : aggregateNeedsResults(results);
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
