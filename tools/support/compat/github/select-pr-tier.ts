/** PRs run affected product checks; the queue retains canonical/browser coverage.
 * Full qualification can be requested explicitly with the `full-ci` label. */
import { appendFileSync } from "node:fs";

import { changedPaths } from "./plan-source-checks.ts";

export type Tier = "heavy" | "fast";

export interface TierRequest {
  eventName: string;
  paths: readonly string[];
  /** Labels on the PR; `full-ci` forces heavy, `hotfix` forces fast. */
  labels: readonly string[];
}

export function selectPrTier({ eventName, paths, labels }: TierRequest): Tier {
  if (eventName === "merge_group") return "heavy";
  if (eventName !== "pull_request") throw new Error("expected a PR or merge-group context");
  if (labels.includes("full-ci")) return "heavy";
  // A hotfix keeps every check: the queue it jumps still runs all of them.
  if (labels.includes("hotfix")) return "fast";
  // An unknown diff is never a reason to skip.
  if (paths.length === 0) return "heavy";
  return "fast";
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
