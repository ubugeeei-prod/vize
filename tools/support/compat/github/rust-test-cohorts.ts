import { workspaceGraph } from "./plan-affected-rust.ts";
import type { CargoMetadata } from "./plan-affected-rust.ts";

export const RUST_TEST_COHORT_IDS = [
  "compiler",
  "typechecker",
  "language-server",
  "lint-and-format",
  "cli-and-integrations",
] as const;
export type RustTestCohortId = (typeof RUST_TEST_COHORT_IDS)[number];

type Definition = {
  name: string;
  packages: readonly string[];
  cliPrefixes?: readonly string[];
};

// Every workspace package has one owner. The CLI's integration binaries follow
// their product feature; its remaining binaries belong to CLI and integrations.
const definitions: Record<RustTestCohortId, Definition> = {
  compiler: {
    name: "Compiler",
    packages: [
      "vize_l0",
      "vize_l0_derive",
      "vize_l1",
      "vize_l2",
      "vize_l3",
      "vize_l4",
      "vize_l1_to_l2",
      "vize_l2_to_l3",
      "vize_armature",
      "vize_relief",
      "vize_carton",
      "vize_croquis",
      "vize_atelier_core",
      "vize_atelier_dom",
      "vize_atelier_sfc",
      "vize_atelier_jsx",
      "vize_atelier_ssr",
      "vize_atelier_vapor",
      "vize_oxc_parser",
      "vize_lightningcss",
      "vize_parcel_sourcemap",
      "vize_incremental",
    ],
  },
  typechecker: {
    name: "Type checker",
    packages: ["vize_canon", "vize_croquis_cf"],
    cliPrefixes: ["check_", "typecheck_"],
  },
  "language-server": {
    name: "Language server",
    packages: ["vize_maestro"],
    cliPrefixes: ["lsp_"],
  },
  "lint-and-format": {
    name: "Lint and format",
    packages: ["vize_patina", "vize_glyph", "vize_oxc_formatter", "oxlint_html_renderer186"],
    cliPrefixes: ["lint_", "fmt_", "format_"],
  },
  "cli-and-integrations": {
    name: "CLI and integrations",
    packages: [
      "vize",
      "vize_vitrine",
      "vize_fresco",
      "vize_musea",
      "vize_marquette",
      "vize_doctor",
      "vize_curator",
      "davinci_harness",
      "davinci_test_support",
      "vize_benchmarks",
      "vize_test_runner",
    ],
  },
};

const cliPrefixes = Object.values(definitions).flatMap(
  (definition) => definition.cliPrefixes ?? [],
);
const cliBinaries = (prefixes: readonly string[]) => `binary(/^(${prefixes.join("|")})/)`;

export const RUST_TEST_COHORTS = RUST_TEST_COHORT_IDS.map((id) => {
  const { name, packages, cliPrefixes: prefixes } = definitions[id];
  const filters = packages.filter((pkg) => pkg !== "vize").map((pkg) => `package(=${pkg})`);
  if (prefixes) filters.push(`(package(=vize) & ${cliBinaries(prefixes)})`);
  if (packages.includes("vize")) {
    // A complement, rather than an enumerated target list, also covers every
    // future CLI binary that does not match a feature prefix.
    filters.push(`(package(=vize) & not ${cliBinaries(cliPrefixes)})`);
  }
  return { id, name, packages, filter: filters.join(" | ") };
});

export function isRustTestCohort(value: unknown): value is RustTestCohortId {
  return RUST_TEST_COHORT_IDS.some((id) => id === value);
}

export function rustTestMatrix(metadata: CargoMetadata, selectedPackages: readonly string[]) {
  const workspace = new Set(workspaceGraph(metadata).names);
  for (const pkg of workspace) {
    if (RUST_TEST_COHORTS.filter((cohort) => cohort.packages.includes(pkg)).length !== 1) {
      throw new Error(`workspace package must have exactly one Rust test cohort: ${pkg}`);
    }
  }
  const selected = new Set(selectedPackages);
  if (!selected.size) throw new Error("Rust test cohorts require at least one selected package");
  for (const pkg of selected) {
    if (!workspace.has(pkg))
      throw new Error(`selected Rust package is outside the workspace: ${pkg}`);
  }
  return {
    include: RUST_TEST_COHORTS.filter(
      (cohort) =>
        cohort.packages.some((pkg) => selected.has(pkg)) ||
        (selected.has("vize") && definitions[cohort.id].cliPrefixes),
    ).map(({ id: cohort, name, filter }) => ({ cohort, name, filter })),
  };
}
