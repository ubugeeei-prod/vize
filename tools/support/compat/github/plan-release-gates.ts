// Selects which release-preflight gates a merge-queue candidate must pass, so
// main cannot merge a change that the exact-SHA release runs would reject.
import { appendFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { aggregateNeedsResults } from "./require-needs-success.ts";
import { changedPaths, isSharedRustInput } from "./plan-source-checks.ts";

export type ReleaseGatePlan = {
  "vue-parity": boolean;
  docs: boolean;
  miri: boolean;
  fuzz: boolean;
};

export const RELEASE_GATE_JOBS = {
  "vue-parity": "vue-parity",
  docs: "docs-build",
  miri: "miri",
  fuzz: "fuzz-replay",
} as const satisfies Record<keyof ReleaseGatePlan, string>;

const NONE: ReleaseGatePlan = { "vue-parity": false, docs: false, miri: false, fuzz: false };
const ALL: ReleaseGatePlan = { "vue-parity": true, docs: true, miri: true, fuzz: true };

// Editing the gate wiring must exercise every gate it wires.
const gateWiring = new Set([
  ".github/workflows/check.yml",
  "tools/support/compat/github/plan-release-gates.ts",
  "tools/support/compat/github/require-needs-success.ts",
]);

// Build-docs.yml's push filter, plus the package and dependency inputs that
// its docs and playground builds consume.
const docsInput =
  /^(docs\/|playground\/|examples\/vite-musea\/|npm\/ui\/|npm\/builder\/vite-musea\/|\.github\/workflows\/build-docs\.yml$|package\.json$|pnpm-(lock|workspace)\.yaml$)/;
const rustInput =
  /^(crates\/|davinci\/|\.cargo\/|Cargo\.(toml|lock)$|rust-toolchain\.toml$|\.github\/workflows\/miri\.yml$)/;
// Fuzz targets replay corpora seeded from repository fixtures.
const fuzzInput =
  /^(tests\/fuzz\/|tests\/fixtures\/|tests\/_fixtures\/_projects\/|tools\/commands\/ci\/fuzz\/|\.github\/workflows\/fuzz\.yml$)/;
// Vue parity builds the CLI and replays fixtures across the repository; only
// authored prose and surfaces it never reads may skip it.
const vueParityExempt = /^(docs\/|\.changeset\/|editors\/|playground\/|examples\/)/;

export function planReleaseGates(paths: readonly string[]): ReleaseGatePlan {
  if (paths.length === 0) return { ...ALL };
  const plan = { ...NONE };
  for (const file of paths) {
    if (gateWiring.has(file)) return { ...ALL };
    const rust = rustInput.test(file) || isSharedRustInput(file);
    if (rust) plan.miri = plan.fuzz = true;
    if (fuzzInput.test(file)) plan.fuzz = true;
    if (docsInput.test(file) && !file.startsWith("docs/davinci/decisions/")) plan.docs = true;
    if (rust || !(vueParityExempt.test(file) || /(^|\/)(README|AGENTS)\.md$/.test(file)))
      plan["vue-parity"] = true;
  }
  return plan;
}

/** A gate may skip only when the plan did not select it. */
export function checkReleaseGates(needs: Record<string, { result?: string; outputs?: object }>) {
  const plan = needs["release-gate-plan"];
  if (plan?.result !== "success") {
    return { exitCode: 1 as const, message: `release-gate-plan: ${plan?.result ?? "missing"}` };
  }
  const outputs = (plan.outputs ?? {}) as Record<string, string>;
  const skippable: Record<string, string> = {};
  for (const [gate, job] of Object.entries(RELEASE_GATE_JOBS)) {
    if (!Object.hasOwn(needs, job)) {
      return { exitCode: 1 as const, message: `release gates must depend on ${job}` };
    }
    if (outputs[gate] !== "true" && outputs[gate] !== "false") {
      return { exitCode: 1 as const, message: `release-gate-plan has no ${gate} decision` };
    }
    if (outputs[gate] === "false") skippable[job] = "unselected";
  }
  return aggregateNeedsResults(needs, skippable);
}

function main(args: string[]) {
  if (args[0] === "--check") {
    const { exitCode, message } = checkReleaseGates(JSON.parse(process.env.NEEDS_JSON ?? "null"));
    (exitCode === 0 ? console.log : console.error)(message);
    process.exitCode = exitCode;
    return;
  }
  const [event, base, head] = args;
  let plan: ReleaseGatePlan;
  if (event === "pull_request") {
    // Pull requests keep their fast lane; the queue runs these gates.
    plan = { ...NONE };
  } else if (event === "merge_group") {
    if (!/^[0-9a-f]{40}$/.test(base ?? "") || !/^[0-9a-f]{40}$/.test(head ?? "")) {
      throw new Error("expected full base and head commit SHAs");
    }
    plan = /^0+$/.test(base!) ? { ...ALL } : planReleaseGates(changedPaths(base!, head!));
  } else {
    throw new Error(`unsupported release gate planning event: ${event}`);
  }
  const lines = Object.entries(plan).map(([gate, run]) => `${gate}=${run}`);
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `${lines.join("\n")}\n`);
  console.log(`Release gates for ${event}: ${lines.join(", ")}`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  main(process.argv.slice(2));
}
