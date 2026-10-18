import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { aggregateNeedsResults } from "./require-needs-success.ts";
import type { GateReport } from "./require-needs-success.ts";

type Tier = "all" | "source" | "products";
type ProductInput = { qualification: unknown; product: unknown };
type ProductVerifier = (input: ProductInput) => unknown;
const productJobs = ["merge-rust-source", "pr-rust-shard"];
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
          ? ["merge-rust-source", "merge-rust-differential", "pr-rust-build", "pr-rust-shard"]
          : ["pr-rust-build", "pr-rust-shard"];
  const selected: Record<string, unknown> = {};
  for (const job of required) {
    if (!Object.hasOwn(jobs, job)) throw new Error(`Missing Rust tier job: ${job}`);
    selected[job] = jobs[job];
  }
  if (tier !== "all") assert.deepEqual(Object.keys(jobs).sort(), [...required].sort());
  const inactive = tier === "source" ? "merge-rust-differential" : "merge-rust-source";
  return aggregateNeedsResults(
    selected,
    tier !== "all" && event === "pull_request" ? { [inactive]: "merge-group only" } : {},
  );
}

function verifyProduct(input: ProductInput): unknown {
  const result = spawnSync(
    process.execPath,
    [
      fileURLToPath(new URL("../../release/qualify_check/integration-product.ts", import.meta.url)),
      "--verify",
    ],
    { input: JSON.stringify(input), encoding: "utf8", maxBuffer: 1024 * 1024 },
  );
  if (result.stderr) process.stderr.write(result.stderr);
  assert.ifError(result.error);
  assert.equal(result.status, 0, "Live integration product verification failed");
  return JSON.parse(result.stdout);
}

/** The CLI always uses the real live verifier; injected providers are pure test controls. */
export function requireRustProducts(
  event: unknown,
  runRust: unknown,
  needs: unknown,
  productQualification: unknown,
  releaseQualification: unknown,
  verify: ProductVerifier = verifyProduct,
): GateReport {
  assert(typeof productQualification === "string");
  assert(typeof releaseQualification === "string");
  if (productQualification === "") return requireRustTier(event, runRust, needs, "products");
  const jobs = context(event, runRust, needs);
  assert.equal(event, "merge_group", "Product reuse is merge-group only");
  assert.deepEqual(Object.keys(jobs).sort(), [...productJobs].sort());
  for (const job of productJobs)
    assert.equal((jobs[job] as { result?: unknown } | null)?.result, "skipped");
  assert(releaseQualification !== "", "Original release qualification is required");
  assert(
    Buffer.byteLength(productQualification) + Buffer.byteLength(releaseQualification) <= 24_576,
  );
  const product: unknown = JSON.parse(productQualification);
  assert(product !== null && typeof product === "object" && !Array.isArray(product));
  const qualification: unknown = JSON.parse(releaseQualification);
  const verified = verify({ qualification, product });
  assert.deepEqual(verified, product, "Live integration product qualification changed");
  return {
    exitCode: 0,
    message:
      "Rust products reuse live-verified full qualification; no candidate worker execution claimed.",
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  assert(args.length <= 1 && (args.length === 0 || ["--source", "--products"].includes(args[0])));
  const needs: unknown = JSON.parse(String(process.env.NEEDS_JSON));
  const result =
    args[0] === "--products"
      ? requireRustProducts(
          process.env.GITHUB_EVENT_NAME,
          process.env.RUN_RUST,
          needs,
          process.env.PRODUCT_QUALIFICATION ?? "",
          process.env.RELEASE_QUALIFICATION ?? "",
        )
      : requireRustTier(
          process.env.GITHUB_EVENT_NAME,
          process.env.RUN_RUST,
          needs,
          args[0] === "--source" ? "source" : "all",
        );
  process.stdout.write(`${result.message}\n`);
  process.exitCode = result.exitCode;
}
