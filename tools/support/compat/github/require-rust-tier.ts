import assert from "node:assert/strict";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { aggregateNeedsResults } from "./require-needs-success.ts";
import type { GateReport } from "./require-needs-success.ts";

type Tier = "all" | "source" | "products";
const productJobs = ["pr-rust-shard"];
const sourceJobs = ["merge-rust-differential", "pr-rust-build"];

function context(event: unknown, runRust: unknown, needs: unknown) {
  if (typeof event !== "string" || !["pull_request", "merge_group"].includes(event))
    throw new Error("Unexpected Rust tier event");
  if (runRust !== "true")
    throw new Error("Rust source checks must run for every pull request and merge group");
  // Preserve the original default mode's null/undefined refusal and primitive
  // coercion: primitive contexts cannot own any of the required named jobs.
  if (needs === null || needs === undefined)
    throw new TypeError("Cannot convert undefined or null to object");
  return typeof needs === "object" || typeof needs === "function"
    ? (needs as Record<string, unknown>)
    : {};
}

export function requireRustTier(
  event: unknown,
  runRust: unknown,
  needs: unknown,
  tier: Tier = "all",
): GateReport {
  const jobs = context(event, runRust, needs);
  if (!["all", "source", "products"].includes(tier)) throw new Error("Unknown Rust tier mode");
  const required =
    tier === "source"
      ? sourceJobs
      : tier === "products"
        ? productJobs
        : event === "merge_group"
          ? ["merge-rust-differential", "pr-rust-build", "pr-rust-shard"]
          : ["pr-rust-build", "pr-rust-shard"];
  const selected: Record<string, unknown> = {};
  for (const job of required) {
    if (!Object.hasOwn(jobs, job)) throw new Error(`Missing Rust tier job: ${job}`);
    selected[job] = jobs[job];
  }
  if (tier !== "all") assert.deepEqual(Object.keys(jobs).sort(), [...required].sort());
  return aggregateNeedsResults(
    selected,
    tier === "source" && event === "pull_request"
      ? { "merge-rust-differential": "merge-group only" }
      : {},
  );
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  assert(args.length <= 1 && (args.length === 0 || ["--source", "--products"].includes(args[0])));
  const needs: unknown = JSON.parse(String(process.env.NEEDS_JSON));
  const result =
    args[0] === "--products"
      ? requireRustTier(process.env.GITHUB_EVENT_NAME, process.env.RUN_RUST, needs, "products")
      : requireRustTier(
          process.env.GITHUB_EVENT_NAME,
          process.env.RUN_RUST,
          needs,
          args[0] === "--source" ? "source" : "all",
        );
  process.stdout.write(`${result.message}\n`);
  process.exitCode = result.exitCode;
}
