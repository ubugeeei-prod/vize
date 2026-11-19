import { appendFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { changedPaths, planSourceChecks } from "./plan-source-checks.ts";
import { planAffectedRust, readCargoMetadata } from "./plan-affected-rust.ts";
import type { CargoMetadata } from "./plan-affected-rust.ts";
import { planAffectedJs, readJsWorkspace } from "./plan-affected-js.ts";
import type { JsPackage } from "./plan-affected-js.ts";
import { planNativeRuntime } from "./plan-native-runtime.ts";
import { differentialMatrix } from "./rust-differential-lanes.ts";
import { rustTestMatrix } from "./rust-test-cohorts.ts";
import { selectPrTier } from "./select-pr-tier.ts";

type Request = {
  paths: readonly string[];
  eventName: string;
  labels: readonly string[];
  workspace: readonly JsPackage[];
  cargoMetadata?: CargoMetadata;
};

function sourceChecks(paths: readonly string[], eventName: string) {
  const selected = planSourceChecks(paths, eventName);
  // An unknown predecessor or empty diff cannot prove a product is unaffected.
  return paths.length ? selected : { rust: true, js: true, tooling: true, playground: true };
}

function requiredRustPlan(
  metadata: CargoMetadata,
  affected: ReturnType<typeof planAffectedRust> | undefined,
  eventName: string,
) {
  if (eventName === "pull_request" && affected?.packages.length) return affected;
  const full = planAffectedRust(metadata, [], eventName);
  if (eventName === "merge_group") {
    return { ...full, reasons: ["merge queue validates the entire Rust workspace"] };
  }
  // Relief's AST, binding and diagnostic tests provide a real compiler baseline
  // for docs/JS-only PRs, without running higher-level dev-oracle suites.
  const packages = ["vize_relief"];
  if (!full.packages.includes(packages[0]))
    throw new Error("missing required vize_relief baseline");
  return {
    ...full,
    scope: "affected",
    packages,
    cargoArgs: packages.flatMap((name) => ["--package", name]),
    reasons: ["required PR compiler baseline: vize_relief"],
    differentialMatrix: differentialMatrix(packages),
  };
}

/** Actual changed dependencies select JS/native work; Rust checks also enforce the PR/MQ policy. */
export function planPrChecks({ paths, eventName, labels, workspace, cargoMetadata }: Request) {
  const source = sourceChecks(paths, eventName);
  if (!cargoMetadata) throw new Error("Rust checks require Cargo workspace metadata");
  const affectedRust = source.rust ? planAffectedRust(cargoMetadata, paths, eventName) : undefined;
  const requiredRust = requiredRustPlan(cargoMetadata, affectedRust, eventName);
  const rust = {
    ...requiredRust,
    testMatrix: rustTestMatrix(cargoMetadata, requiredRust.packages),
  };
  const js = planAffectedJs(workspace, paths, affectedRust);
  const tier = selectPrTier({ paths, eventName, labels });
  // #6830 T1: native runtime lanes execute real Vue and tsgo oracles. The merge
  // queue plans the same affected lanes from the same diff; `full-ci` PRs and
  // unknown diffs (heavy tier) keep them on the PR.
  const affectedNative = planNativeRuntime(paths, affectedRust);
  const native =
    tier === "heavy" || !affectedNative.run
      ? { ...affectedNative, deferred: [] as string[] }
      : { run: false, matrix: { lane: ["none"] }, deferred: affectedNative.matrix.lane };
  // Preserve the real-tree source/API and reviewed-doc contracts in daily CI.
  // Unknown Rust/tooling inputs remain conservative; ordinary package-only
  // edits do not pay for these independent repository scans.
  const sourceContracts =
    source.rust || source.tooling || paths.some((path) => path.startsWith("docs/"));
  return { source, rust, js, native, tier, sourceContracts };
}

// Reasons explain a plan; they select nothing. Every needs output travels in
// one environment string to the source report, and Linux caps one string at
// 128 KiB, so a PR touching hundreds of fixtures must not list every path.
export const MAX_OUTPUT_REASONS = 40;

function boundReasons<T extends { reasons: string[] }>(plan: T): T {
  if (plan.reasons.length <= MAX_OUTPUT_REASONS) return plan;
  const omitted = plan.reasons.length - MAX_OUTPUT_REASONS;
  return {
    ...plan,
    reasons: [...plan.reasons.slice(0, MAX_OUTPUT_REASONS), `... and ${omitted} more`],
  };
}

export function prCheckOutputs(plan: ReturnType<typeof planPrChecks>): Record<string, string> {
  return {
    rust: "true",
    "rust-plan": JSON.stringify(boundReasons(plan.rust)),
    js: String(plan.js.groups.length > 0 || plan.js.conformance.fresco || plan.js.conformance.ui),
    "js-plan": JSON.stringify(boundReasons(plan.js)),
    tooling: String(plan.source.tooling),
    "source-contracts": String(plan.sourceContracts),
    heavy: String(plan.tier === "heavy"),
    playground: String(plan.tier === "heavy" && plan.source.playground),
    "native-runtime-matrix": JSON.stringify(plan.native.matrix),
  };
}

export function main(argv = process.argv.slice(2)) {
  const [base, head, eventName] = argv;
  if (argv.length !== 3 || ![base, head].every((sha) => /^[0-9a-f]{40}$/.test(sha ?? ""))) {
    throw new Error("expected full base and head SHAs and an event context");
  }
  const labels: unknown = JSON.parse(process.env.PR_LABELS || "[]");
  if (!Array.isArray(labels) || labels.some((label) => typeof label !== "string")) {
    throw new Error("expected PR_LABELS to contain label names");
  }
  const paths = /^0+$/.test(base) ? [] : changedPaths(base, head);
  const plan = planPrChecks({
    paths,
    eventName,
    labels,
    workspace: readJsWorkspace(),
    cargoMetadata: readCargoMetadata(),
  });
  const outputs = Object.entries(prCheckOutputs(plan))
    .map(([name, value]) => `${name}=${value}\n`)
    .join("");
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, outputs);
  const summary = `${paths.length} changed paths; Rust: ${plan.rust.packages.length} crates; JS: ${plan.js.groups.join(", ") || "none"}; native: ${plan.native.matrix.lane.join(", ")}${plan.native.deferred.length ? ` (T1: ${plan.native.deferred.join(", ")})` : ""}; tier: ${plan.tier}.\n`;
  process.stdout.write(summary);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
