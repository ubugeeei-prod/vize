import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { aggregateNeedsResults } from "./require-needs-success.mjs";

export function requireRustTier(event, runRust, needs, fast = "false") {
  if (!["pull_request", "merge_group"].includes(event))
    throw new Error("Unexpected Rust tier event");
  if (!["true", "false"].includes(runRust)) throw new Error("Missing Rust source plan");
  if (!["true", "false"].includes(fast)) throw new Error("Missing Rust fast-tier plan");
  if (event === "merge_group" && runRust !== "true")
    throw new Error("Merge queue must select the complete Rust workspace");
  if (fast === "true" && (event !== "pull_request" || runRust !== "true"))
    throw new Error("Fast Rust tier requires a selected pull request");
  const required =
    event === "merge_group"
      ? ["merge-rust-source", "pr-rust-shard"]
      : runRust === "true"
        ? fast === "true"
          ? ["pr-rust-fast-clippy", "pr-rust-fast-tests"]
          : ["pr-rust-build", "pr-rust-shard"]
        : [];
  if (required.length === 0) {
    return {
      exitCode: 0,
      message: "Rust source plan selects no crates; archive and shards skipped.",
    };
  }
  const selected = {};
  for (const job of required) {
    if (!Object.hasOwn(needs, job)) throw new Error(`Missing Rust tier job: ${job}`);
    selected[job] = needs[job];
  }
  return aggregateNeedsResults(selected);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = requireRustTier(
    process.env.GITHUB_EVENT_NAME,
    process.env.RUN_RUST,
    JSON.parse(process.env.NEEDS_JSON),
    process.env.RUST_FAST,
  );
  process.stdout.write(`${result.message}\n`);
  process.exitCode = result.exitCode;
}
