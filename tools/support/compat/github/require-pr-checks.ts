import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { canonicalSourceReport } from "./canonical-corpus-selection.mjs";
import type { GateReport } from "./require-needs-success.ts";
import { requireRustTier } from "./require-rust-tier.ts";

// Match the queue source report's dependencies. Workflow contracts test this
// list so that removing a job from the exported context cannot silently pass.
export const PR_SOURCE_GATES = Object.freeze([
  "pr-source-plan",
  "pr-rust-source",
  "pr-js-packages",
  "pr-native-runtime",
  "pr-playground-test",
  "wit-contracts",
  "nuxt-module-qualification",
  "canonical-corpus",
]);

function object(value: unknown, label: string): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new Error(`${label} must be an object of actual workflow results`);
  return value as Record<string, unknown>;
}

export function requirePrChecks(sourceJobs: unknown): GateReport {
  const source = object(sourceJobs, "PR source jobs");
  for (const job of PR_SOURCE_GATES) {
    if (!Object.hasOwn(source, job)) throw new Error(`Missing PR source job: ${job}`);
  }
  // Only this report is intentionally retired on PRs. Keep every other entry
  // so that unexpected failures or skips remain fatal in the existing policy.
  const sourceNeeds = Object.fromEntries(
    Object.entries(source).filter(([job]) => job !== "source-report"),
  );
  const sourceReport = canonicalSourceReport(sourceNeeds, "pull_request");
  if (sourceReport.exitCode !== 0) return sourceReport;

  const rust = object(source["pr-rust-source"], "Rust workflow result");
  const outputs = object(rust.outputs, "Rust workflow outputs");
  const rawRustJobs = outputs["jobs-json"];
  if (typeof rawRustJobs !== "string" || !rawRustJobs.trim())
    throw new Error("Missing actual Rust workflow job results");
  const rustJobs = object(JSON.parse(rawRustJobs) as unknown, "Rust jobs");
  const rustReport = requireRustTier("pull_request", outputs["run-rust"], rustJobs);
  if (rustReport.exitCode !== 0) return rustReport;
  return {
    exitCode: 0,
    message: `PR Rust results: ${rustReport.message}\nPR source results: ${sourceReport.message}`,
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.env.GITHUB_EVENT_NAME !== "pull_request")
    throw new Error("The flattened source report only accepts pull_request events");
  const raw = process.env.PR_SOURCE_JOBS_JSON;
  if (!raw?.trim()) throw new Error("PR_SOURCE_JOBS_JSON is required");
  const report = requirePrChecks(JSON.parse(raw) as unknown);
  process.stdout.write(`${report.message}\n`);
  process.exitCode = report.exitCode;
}
