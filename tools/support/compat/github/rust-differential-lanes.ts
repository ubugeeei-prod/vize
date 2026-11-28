import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { fewestJobs } from "../../ci/job-packing.ts";

type Step = {
  package: string;
  args: string[];
  env?: Record<string, string>;
  workspaceCorpus?: true;
};
type Lane = { name: string; steps: Step[]; navigation?: boolean; elementPlus?: true };
const test = (pkg: string, args: string[], env?: Record<string, string>): Step => ({
  package: pkg,
  args: ["test", "--locked", "-p", pkg, ...args],
  env,
});
const feature = (name: string, filter: string, allTargets = false): Step[] => [
  test("vize_maestro", ["--features", name, "--lib", filter, "--", "--nocapture"]),
  {
    package: "vize_maestro",
    args: ["check", "--locked", "-p", "vize_maestro", "--no-default-features", "--features", name],
  },
  {
    package: "vize_maestro",
    args: [
      "clippy",
      "--locked",
      "-p",
      "vize_maestro",
      "--features",
      name,
      ...(allTargets ? ["--all-targets"] : ["--lib", "--tests"]),
      "--",
      "-D",
      "warnings",
    ],
  },
];
const corpus = (pkg: string, names: string[], features = "legacy-differential"): Step =>
  test(pkg, [
    "--features",
    features,
    ...names.flatMap((name) => ["--test", name]),
    "--",
    "--nocapture",
  ]);

// One recipe serves both the affected merge-queue lanes and scheduled full
// workspace checks. Every selected step runs with its own runtime environment.
export const rustDifferentialLanes: readonly Lane[] = [
  {
    name: "program",
    elementPlus: true,
    steps: [
      ...feature("experimental-source-folding", "source_folding::tests::"),
      test("vize_maestro", ["--lib", "source_project::", "--", "--nocapture"]),
      ...feature("experimental-source-project", "source_project::", true),
    ],
  },
  {
    name: "sfc-project",
    elementPlus: true,
    steps: [
      ...feature("experimental-sfc-folding", "source_sfc_folding::tests::"),
      ...feature(
        "experimental-source-project,experimental-sfc-folding",
        "source_sfc_folding::",
        true,
      ),
    ],
  },
  { name: "navigation", steps: [], navigation: true, elementPlus: true },
  { name: "surface", steps: [corpus("vize_l1", ["surface_corpus"])] },
  {
    name: "lowering",
    steps: [
      corpus("vize_l1_to_l2", [
        "davinci_lowering_corpus",
        "davinci_dom_corpus",
        "davinci_remarks_corpus",
        "davinci_pug_corpus",
      ]),
    ],
  },
  {
    name: "sfc",
    steps: [
      corpus("vize_atelier_sfc", ["davinci_production_reach"], "legacy-dom-differential"),
      test("vize_atelier_sfc", [
        "--features",
        "legacy-dom-differential",
        "--lib",
        "compile_template::duplicates::",
        "--",
        "--nocapture",
      ]),
      test("vize_atelier_sfc", ["--test", "vapor_runtime_contracts"], {
        VIZE_VUE_RUNTIME_PRODUCTION: "1",
      }),
    ],
  },
  {
    name: "ssr",
    steps: [
      {
        ...corpus("vize_atelier_ssr", ["davinci_ssr_corpus"]),
        workspaceCorpus: true,
      },
      corpus("vize_atelier_ssr", ["davinci_ssr_production_cases", "slot_element_text_children"]),
      {
        ...corpus("vize_atelier_ssr", ["vue_ssr_render"]),
        env: { VIZE_VUE_RUNTIME_PRODUCTION: "1" },
      },
    ],
  },
  {
    name: "compiler",
    steps: [
      test("vize_atelier_jsx", [
        "--features",
        "legacy-differential",
        "--lib",
        "vdom::l2_differential::l2_vdom_admitted_cases_match_relief_codegen",
        "--",
        "--exact",
      ]),
      corpus("vize_atelier_core", ["davinci_l2_transform_vue2"], "legacy"),
      corpus("vize_atelier_dom", [
        "v_pre_freezes_its_subtree",
        "component_v_once",
        "element_v_once",
      ]),
    ],
  },
  {
    name: "cli",
    steps: [
      test("vize", [
        "--features",
        "legacy",
        "--test",
        "integration",
        "--",
        "check_nuxt_tsconfig_isolation_cli::",
        "--nocapture",
      ]),
    ],
  },
  { name: "wasm", steps: [test("vize_vitrine", ["--no-default-features", "--features", "wasm"])] },
  {
    name: "formatter",
    steps: [{ ...corpus("vize_patina", ["davinci_markup_differential"]), workspaceCorpus: true }],
  },
  {
    name: "vapor",
    steps: [
      test("vize_atelier_vapor", ["--test", "davinci_vapor_upstream_parity"], {
        VIZE_VUE_RUNTIME_PRODUCTION: "1",
      }),
    ],
  },
];

/**
 * Separates the lanes of one job in the job name, on the `run` command line and
 * in the test-case packet and artifact names, so it avoids spaces and commas.
 */
export const DIFFERENTIAL_LANE_SEPARATOR = "+";

// Work per lane on the merge-queue runner: the 75th-percentile job duration over
// 33 merge-queue runs (2026-11-27/28), minus about 38 s of shared setup, rounded
// up to 5 s. A lane without an entry is costed at the whole budget.
export const differentialLaneSeconds: Readonly<Record<string, number>> = {
  program: 175,
  "sfc-project": 150,
  navigation: 130,
  sfc: 125,
  cli: 75,
  wasm: 60,
  formatter: 40,
  vapor: 40,
  ssr: 30,
  compiler: 35,
  lowering: 20,
  surface: 10,
};

// A packed job runs no longer than the longest single lane, so packing never
// lengthens the merge group's differential critical path.
const laneBudget = Math.max(...Object.values(differentialLaneSeconds));
const laneSeconds = (lane: Lane) => differentialLaneSeconds[lane.name] ?? laneBudget;

export interface DifferentialMatrix {
  readonly include: readonly { readonly lane: string; readonly elementPlus: boolean }[];
}

/**
 * The affected lanes, packed into the fewest jobs within the budget. Navigation
 * runs its own action steps, so it keeps a job of its own.
 */
export function differentialMatrix(packages: readonly string[]): DifferentialMatrix {
  const lanes = rustDifferentialLanes.filter((lane) =>
    lane.navigation
      ? packages.includes("vize_maestro")
      : lane.steps.some((step) => packages.includes(step.package)),
  );
  if (lanes.length === 0) return { include: [{ lane: "none", elementPlus: false }] };
  const jobs = [
    ...fewestJobs(
      lanes.filter((lane) => !lane.navigation),
      laneSeconds,
      laneBudget,
    ),
    ...lanes.filter((lane) => lane.navigation).map((lane) => [lane]),
  ];
  return {
    include: jobs.map((job) => ({
      lane: job.map(({ name }) => name).join(DIFFERENTIAL_LANE_SEPARATOR),
      elementPlus: job.some((lane) => lane.elementPlus === true),
    })),
  };
}

/** Lane names of one matrix job's `lane` value. */
export function differentialJobLanes(lane: string): string[] {
  return lane.split(DIFFERENTIAL_LANE_SEPARATOR);
}

export function differentialSteps(lane: string, packages: readonly string[]): Step[] {
  const selected = rustDifferentialLanes.find(({ name }) => name === lane);
  if (!selected || selected.navigation) throw new Error(`unknown command lane: ${lane}`);
  return selected.steps.filter((step) => packages.includes(step.package));
}

// Runs one lane's steps in order and stops at its first failing step.
async function runLane(label: string, steps: readonly Step[]): Promise<number> {
  for (const step of steps) {
    const env = {
      ...process.env,
      ...(step.workspaceCorpus ? { VIZE_DAVINCI_DIFFERENTIAL_CORPUS: process.cwd() } : {}),
      ...step.env,
    };
    if (process.env.VIZE_TEST_CASES) {
      const { runRustCaseCommand } = await import("./rust-test-cases.ts");
      const status = await runRustCaseCommand({
        command: ["cargo", ...step.args],
        packages: [step.package],
        label: `Rust differential ${label} › ${step.package} ${step.args[0]}`,
        output: process.env.VIZE_TEST_CASES,
        env,
      });
      if (status !== 0) return status;
      continue;
    }
    const result = spawnSync("cargo", step.args, {
      stdio: "inherit",
      shell: false,
      env,
    });
    if (result.error) throw result.error;
    if (result.status !== 0) return result.status ?? 1;
  }
  return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const args = process.argv.slice(2);
  let lanes: { label: string; steps: Step[] }[];
  if (args.length === 1 && args[0] === "--all") {
    lanes = [{ label: args[0], steps: rustDifferentialLanes.flatMap((lane) => lane.steps) }];
  } else {
    if (args.length !== 2) throw new Error("expected --all or a lane list and Rust plan path");
    const [job, planPath] = args;
    const plan = JSON.parse(readFileSync(planPath, "utf8"));
    if (
      !Array.isArray(plan.packages) ||
      plan.packages.some((pkg: unknown) => typeof pkg !== "string")
    )
      throw new Error("invalid Rust package selection");
    lanes = differentialJobLanes(job).map((lane) => ({
      label: lane,
      steps: differentialSteps(lane, plan.packages),
    }));
  }
  for (const { steps } of lanes)
    if (!steps.length) throw new Error("selected differential lane has no affected package");
  // A failing lane does not stop the job's later lanes, so every lane still
  // reports, and the job fails if any lane failed.
  const failed: string[] = [];
  for (const { label, steps } of lanes) {
    if (lanes.length > 1) console.log(`Differential lane ${label}`);
    const status = await runLane(label, steps);
    if (status === 0) continue;
    failed.push(label);
    process.exitCode = status;
  }
  if (lanes.length > 1 && failed.length > 0)
    console.log(`::error title=Rust differential lanes::Failed lanes: ${failed.join(", ")}`);
}
