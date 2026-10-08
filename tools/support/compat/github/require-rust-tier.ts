import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { aggregateNeedsResults } from "./require-needs-success.ts";
import type { GateReport } from "./require-needs-success.ts";

export function requireRustTier(event: unknown, runRust: unknown, needs: unknown): GateReport {
  if (typeof event !== "string" || !["pull_request", "merge_group"].includes(event))
    throw new Error("Unexpected Rust tier event");
  if (typeof runRust !== "string" || !["true", "false"].includes(runRust))
    throw new Error("Missing Rust source plan");
  if (event === "merge_group" && runRust !== "true")
    throw new Error("Merge queue must select the complete Rust workspace");
  const required =
    event === "merge_group"
      ? ["merge-rust-source", "merge-rust-differential", "pr-rust-shard"]
      : runRust === "true"
        ? ["pr-rust-build", "pr-rust-shard"]
        : [];
  if (required.length === 0) {
    return {
      exitCode: 0,
      message: "Rust source plan selects no crates; archive and shards skipped.",
    };
  }
  // Object.hasOwn previously coerced primitive contexts; none can own these
  // named jobs. Preserve its null/undefined refusal and the no-Rust early return.
  if (needs === null || needs === undefined)
    throw new TypeError("Cannot convert undefined or null to object");
  const context: Record<string, unknown> =
    typeof needs === "object" || typeof needs === "function"
      ? (needs as Record<string, unknown>)
      : {};
  const selected: Record<string, unknown> = {};
  for (const job of required) {
    if (!Object.hasOwn(context, job)) throw new Error(`Missing Rust tier job: ${job}`);
    selected[job] = context[job];
  }
  return aggregateNeedsResults(selected);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = requireRustTier(
    process.env.GITHUB_EVENT_NAME,
    process.env.RUN_RUST,
    JSON.parse(String(process.env.NEEDS_JSON)) as unknown,
  );
  process.stdout.write(`${result.message}\n`);
  process.exitCode = result.exitCode;
}
