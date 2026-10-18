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
  const native = planNativeRuntime(paths, affectedRust);
  const tier = selectPrTier({ paths, eventName, labels });
  // Preserve the real-tree source/API and reviewed-doc contracts in daily CI.
  // Unknown Rust/tooling inputs remain conservative; ordinary package-only
  // edits do not pay for these independent repository scans.
  const sourceContracts =
    source.rust || source.tooling || paths.some((path) => path.startsWith("docs/"));
  return { source, rust, js, native, tier, sourceContracts };
}

export function prCheckOutputs(plan: ReturnType<typeof planPrChecks>): Record<string, string> {
  return {
    rust: "true",
    "rust-plan": JSON.stringify(plan.rust),
    js: String(plan.js.groups.length > 0 || plan.js.conformance.fresco || plan.js.conformance.ui),
    "js-plan": JSON.stringify(plan.js),
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
  const summary = `${paths.length} changed paths; Rust: ${plan.rust.packages.length} crates; JS: ${plan.js.groups.join(", ") || "none"}; native: ${plan.native.matrix.lane.join(", ")}; tier: ${plan.tier}.\n`;
  process.stdout.write(summary);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
