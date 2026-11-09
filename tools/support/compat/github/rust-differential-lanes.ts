import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

type Step = {
  package: string;
  args: string[];
  env?: Record<string, string>;
  workspaceCorpus?: true;
};
type Lane = { name: string; steps: Step[]; navigation?: boolean };
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
    steps: [
      ...feature("experimental-source-folding", "source_folding::tests::"),
      test("vize_maestro", ["--lib", "source_project::", "--", "--nocapture"]),
      ...feature("experimental-source-project", "source_project::", true),
    ],
  },
  {
    name: "sfc-project",
    steps: [
      ...feature("experimental-sfc-folding", "source_sfc_folding::tests::"),
      ...feature(
        "experimental-source-project,experimental-sfc-folding",
        "source_sfc_folding::",
        true,
      ),
    ],
  },
  { name: "navigation", steps: [], navigation: true },
  { name: "surface", steps: [corpus("vize_l1", ["surface_corpus"])] },
  {
    name: "lowering",
    steps: [
      corpus("vize_l1_to_l2", [
        "level_lowering_corpus",
        "level_dom_corpus",
        "level_remarks_corpus",
        "level_pug_corpus",
      ]),
    ],
  },
  {
    name: "sfc",
    steps: [
      corpus("vize_atelier_sfc", ["level_production_reach"], "legacy-dom-differential"),
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
        ...corpus("vize_atelier_ssr", ["level_ssr_corpus"]),
        workspaceCorpus: true,
      },
      corpus("vize_atelier_ssr", ["level_ssr_production_cases", "slot_element_text_children"]),
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
      corpus("vize_atelier_core", ["level_l2_transform_vue2"], "legacy"),
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
    steps: [{ ...corpus("vize_patina", ["level_markup_differential"]), workspaceCorpus: true }],
  },
  {
    name: "vapor",
    steps: [
      test("vize_atelier_vapor", ["--test", "level_vapor_upstream_parity"], {
        VIZE_VUE_RUNTIME_PRODUCTION: "1",
      }),
    ],
  },
];

export function differentialMatrix(packages: readonly string[]) {
  const lanes = rustDifferentialLanes
    .filter((lane) =>
      lane.navigation
        ? packages.includes("vize_maestro")
        : lane.steps.some((step) => packages.includes(step.package)),
    )
    .map(({ name }) => name);
  return { lane: lanes.length ? lanes : ["none"] };
}

export function differentialSteps(lane: string, packages: readonly string[]): Step[] {
  const selected = rustDifferentialLanes.find(({ name }) => name === lane);
  if (!selected || selected.navigation) throw new Error(`unknown command lane: ${lane}`);
  return selected.steps.filter((step) => packages.includes(step.package));
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const args = process.argv.slice(2);
  let steps: Step[];
  if (args.length === 1 && args[0] === "--all") {
    steps = rustDifferentialLanes.flatMap((lane) => lane.steps);
  } else {
    if (args.length !== 2) throw new Error("expected --all or a lane and Rust plan path");
    const [lane, planPath] = args;
    const plan = JSON.parse(readFileSync(planPath, "utf8"));
    if (
      !Array.isArray(plan.packages) ||
      plan.packages.some((pkg: unknown) => typeof pkg !== "string")
    )
      throw new Error("invalid Rust package selection");
    steps = differentialSteps(lane, plan.packages);
  }
  if (!steps.length) throw new Error("selected differential lane has no affected package");
  for (const step of steps) {
    const env = {
      ...process.env,
      ...(step.workspaceCorpus ? { VIZE_LEVEL_DIFFERENTIAL_CORPUS: process.cwd() } : {}),
      ...step.env,
    };
    if (process.env.VIZE_TEST_CASES) {
      const { runRustCaseCommand } = await import("./rust-test-cases.ts");
      const status = await runRustCaseCommand({
        command: ["cargo", ...step.args],
        packages: [step.package],
        label: `Rust differential ${args[0]} › ${step.package} ${step.args[0]}`,
        output: process.env.VIZE_TEST_CASES,
        env,
      });
      if (status !== 0) {
        process.exitCode = status;
        break;
      }
      continue;
    }
    const result = spawnSync("cargo", step.args, {
      stdio: "inherit",
      shell: false,
      env,
    });
    if (result.error) throw result.error;
    if (result.status !== 0) {
      process.exitCode = result.status ?? 1;
      break;
    }
  }
}
