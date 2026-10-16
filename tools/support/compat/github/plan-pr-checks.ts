import { appendFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { changedPaths, planSourceChecks } from "./plan-source-checks.ts";
import { planAffectedRust, readCargoMetadata } from "./plan-affected-rust.ts";
import type { CargoMetadata } from "./plan-affected-rust.ts";
import { planAffectedJs, readJsWorkspace } from "./plan-affected-js.ts";
import type { JsPackage } from "./plan-affected-js.ts";
import { planNativeRuntime } from "./plan-native-runtime.ts";
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

/** A single diff feeds the pure selectors; Rust's closure feeds both JS and native runtimes. */
export function planPrChecks({ paths, eventName, labels, workspace, cargoMetadata }: Request) {
  const source = sourceChecks(paths, eventName);
  if (source.rust && !cargoMetadata)
    throw new Error("Rust inputs require Cargo workspace metadata");
  const rust = source.rust ? planAffectedRust(cargoMetadata!, paths, eventName) : undefined;
  const js = planAffectedJs(workspace, paths, rust);
  const native = planNativeRuntime(paths, rust);
  const tier = selectPrTier({ paths, eventName, labels });
  return { source, rust, js, native, tier };
}

export function prCheckOutputs(plan: ReturnType<typeof planPrChecks>): Record<string, string> {
  return {
    rust: String(plan.source.rust),
    "rust-plan": plan.rust ? JSON.stringify(plan.rust) : "",
    js: String(plan.js.groups.length > 0 || plan.js.conformance.fresco || plan.js.conformance.ui),
    "js-plan": JSON.stringify(plan.js),
    tooling: String(plan.source.tooling),
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
  const source = sourceChecks(paths, eventName);
  const plan = planPrChecks({
    paths,
    eventName,
    labels,
    workspace: readJsWorkspace(),
    cargoMetadata: source.rust ? readCargoMetadata() : undefined,
  });
  const outputs = Object.entries(prCheckOutputs(plan))
    .map(([name, value]) => `${name}=${value}\n`)
    .join("");
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, outputs);
  const summary = `${paths.length} changed paths; Rust: ${plan.rust?.packages.length ?? 0} crates; JS: ${plan.js.groups.join(", ") || "none"}; native: ${plan.native.matrix.lane.join(", ")}; tier: ${plan.tier}.\n`;
  process.stdout.write(summary);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();
