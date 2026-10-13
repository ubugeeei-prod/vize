/**
 * Decides whether a PR run also executes the heavy lanes that the merge queue
 * always executes: the canonical corpus observers, the Oxlint original project
 * qualification and the playground browser tests.
 *
 * Nothing reaches main without the queue running every lane on the actual
 * merge candidate, so skipping a lane on the PR removes a duplicate, not a
 * guarantee. A lane that fails only in the queue ejects its candidate and makes
 * every later candidate rebuild, so the automatic choice errs towards heavy.
 */
import { appendFileSync } from "node:fs";

import { changedPaths } from "./plan-source-checks.ts";

export type Tier = "heavy" | "fast";

export interface TierRequest {
  eventName: string;
  paths: readonly string[];
  /** Labels on the PR; `full-ci` forces heavy, `hotfix` forces fast. */
  labels: readonly string[];
}

// Changes most likely to break a heavy lane: every compiler, checker and
// linter source, the native and Oxlint packages, the playground, the build
// graph, and the workflows and tooling those lanes run.
const heavyInputs = [
  /^(?:crates|davinci|vendor)\//,
  /^npm\/(?:oxlint|native|builder|cli)\//,
  /^playground\//,
  /^(?:Cargo\.(?:toml|lock)|rust-toolchain\.toml|package\.json|pnpm-lock\.yaml|pnpm-workspace\.yaml)$/,
  /^\.cargo\//,
  /^\.github\/(?:workflows|actions)\//,
  /^tools\/support\/compat\/github\//,
  /^tools\/config\//,
  /^tests\/(?:davinci_test_support|_fixtures)\//,
];

export function selectPrTier({ eventName, paths, labels }: TierRequest): Tier {
  if (eventName === "merge_group") return "heavy";
  if (eventName !== "pull_request") throw new Error("expected a PR or merge-group context");
  if (labels.includes("full-ci")) return "heavy";
  // A hotfix keeps every check: the queue it jumps still runs all of them.
  if (labels.includes("hotfix")) return "fast";
  // An unknown diff is never a reason to skip.
  if (paths.length === 0) return "heavy";
  return paths.some((path) => heavyInputs.some((pattern) => pattern.test(path))) ? "heavy" : "fast";
}

if (process.argv[1]?.endsWith("/select-pr-tier.ts")) {
  const [base, head, eventName = "pull_request"] = process.argv.slice(2);
  if (!/^[0-9a-f]{40}$/.test(base ?? "") || !/^[0-9a-f]{40}$/.test(head ?? "")) {
    throw new Error("expected full base and head commit SHAs");
  }
  const labels = JSON.parse(process.env.PR_LABELS || "[]") as unknown;
  if (!Array.isArray(labels) || labels.some((label) => typeof label !== "string")) {
    throw new Error("expected PR_LABELS to be a JSON array of label names");
  }
  const paths = /^0+$/.test(base!) ? [] : changedPaths(base!, head!);
  const tier = selectPrTier({ eventName, paths, labels: labels as string[] });
  const output = process.env.GITHUB_OUTPUT;
  if (output) appendFileSync(output, `heavy=${tier === "heavy"}\n`);
  process.stdout.write(`PR tier: ${tier} (${eventName}, ${paths.length} changed paths)\n`);
}
