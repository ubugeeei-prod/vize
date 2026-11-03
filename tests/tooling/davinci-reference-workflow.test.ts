import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...segments), "utf8");
}

type Step = {
  uses?: string;
  run?: string;
  if?: unknown;
  "continue-on-error"?: unknown;
  "working-directory"?: string;
  with?: Record<string, unknown>;
};
type Workflow = {
  on: { schedule: Array<{ cron: string }>; workflow_dispatch: unknown };
  jobs: Record<string, { steps: Step[]; if?: unknown; "continue-on-error"?: unknown }>;
};

const workflowPath = [".github", "workflows", "davinci-reference.yml"] as const;
const job = "reference-backends";

function assertReferenceWorkflow(workflow: Workflow): void {
  assert.equal(Object.hasOwn(workflow.on, "push"), false);
  assert.deepEqual(workflow.on.schedule, [{ cron: "11 5 * * *" }]);
  assert.ok(Object.hasOwn(workflow.on, "workflow_dispatch"));
  assert.ok(!Object.hasOwn(workflow.on, "pull_request"));
  const lane = workflow.jobs[job];
  assert.ok(lane);
  assert.equal(lane.if, undefined);
  assert.ok(lane["continue-on-error"] === undefined || lane["continue-on-error"] === false);
  const vp = lane.steps.find((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  assert.equal(vp?.uses, "voidzero-dev/setup-vp@ca1c46663915d6c1042ae23bd39ab85718bfb0fa");
  assert.equal(vp?.with?.["node-version-file"], "package.json");
  assert.equal(vp?.with?.["run-install"], false);
  for (const step of lane.steps) {
    assert.equal(step.if, undefined);
    assert.ok(step["continue-on-error"] === undefined || step["continue-on-error"] === false);
  }
  assert.deepEqual(
    lane.steps.flatMap((step) =>
      step.run ? [[step["working-directory"] ?? ".", step.run.trim()]] : [],
    ),
    [
      [".", "vp install --frozen-lockfile --prefer-offline"],
      [".", "cargo test -p vize_l3 --test lattice_reference_fixture"],
      [".", "cargo test -p vize_l2_to_l3 --test l3_reference_fixture"],
      [".", "cargo test -p vize_atelier_vapor --test davinci_l3_compiled_trace"],
      [".", "cargo test -p vize_atelier_vapor --test davinci_mounted_behavior"],
      [
        ".",
        "cargo test -p vize_atelier_vapor --test davinci_vapor_upstream_parity --test davinci_keep_alive_parity --test davinci_suspense_parity --test davinci_structural_slots_parity --test davinci_transition_parity",
      ],
      ["tests", "vize-ci-apt-retry vp exec playwright install --with-deps chromium"],
      [
        ".",
        "cargo test -p vize_atelier_vapor --test davinci_event_handlers -- --ignored --nocapture",
      ],
    ],
  );
}

test("TS-28 reference and mounted backends run nightly and on dispatch", () => {
  assert.equal(
    fs.existsSync(path.join(repoRoot, ".github", "workflows", "davinci-lean.yml")),
    false,
  );
  assertReferenceWorkflow(parseYaml(readRepoFile(...workflowPath)));
});

test("TS-28 rejects commented, moved and disabled workflow commands", () => {
  const source = readRepoFile(...workflowPath);
  const command = "cargo test -p vize_l2_to_l3 --test l3_reference_fixture";
  const commented = source.replace(`run: ${command}`, `run: "true" # ${command}`);
  assert.throws(() => assertReferenceWorkflow(parseYaml(commented)), assert.AssertionError);
  const moved: Workflow = parseYaml(source);
  const steps = moved.jobs[job].steps;
  const index = steps.findIndex((step) => step.run === command);
  assert.notEqual(index, -1);
  moved.jobs.unrelated = { steps: steps.splice(index, 1) };
  assert.throws(() => assertReferenceWorkflow(moved), assert.AssertionError);
  for (const field of ["if", "continue-on-error"] as const) {
    const disabled: Workflow = parseYaml(source);
    disabled.jobs[job].steps.find((step) => step.run === command)![field] = field !== "if";
    assert.throws(() => assertReferenceWorkflow(disabled), assert.AssertionError);
  }
});

test("TS-30 Chromium contract cannot silently skip its ignored Rust test", () => {
  const source = readRepoFile(...workflowPath);
  const command =
    "cargo test -p vize_atelier_vapor --test davinci_event_handlers -- --ignored --nocapture";
  const skipped = source.replace(command, command.replace(" -- --ignored --nocapture", ""));
  assert.throws(() => assertReferenceWorkflow(parseYaml(skipped)), assert.AssertionError);
});

test("TS-28 rejects failure-ignore settings at job and step scopes", () => {
  const source = readRepoFile(...workflowPath);
  for (const scope of ["job", "step"] as const) {
    for (const value of [true, "${{ true }}", "${{ false }}", "false", null]) {
      const workflow: Workflow = parseYaml(source);
      const lane = workflow.jobs[job];
      const target = scope === "job" ? lane : lane.steps.find((step) => step.run)!;
      target["continue-on-error"] = value;
      assert.throws(() => assertReferenceWorkflow(workflow), assert.AssertionError);
    }
  }
  const explicitFalse: Workflow = parseYaml(source);
  const lane = explicitFalse.jobs[job];
  lane["continue-on-error"] = false;
  for (const step of lane.steps) step["continue-on-error"] = false;
  assertReferenceWorkflow(explicitFalse);
});

function harnesses(...files: string[][]): string {
  return files.map((segments) => readRepoFile(...segments)).join("\n");
}

function assertHarness(source: string, name: string): void {
  assert.match(
    source,
    new RegExp(
      `^ *#\\[kani::proof\\]\\n(?: *#\\[kani::unwind\\(\\d+\\)\\]\\n)? *fn ${name}\\(\\)`,
      "mu",
    ),
    `missing Kani harness ${name}`,
  );
}

test("P3-15 laws are Kani harnesses that the Kani workflow verifies", () => {
  const lattice = harnesses(
    ["davinci", "vize_l3", "src", "lattice", "kani.rs"],
    ["davinci", "vize_l3", "src", "lattice", "classify_proofs.rs"],
  );
  for (const name of [
    "join_has_its_units_and_is_monotone",
    "the_effect_floor_is_the_join_of_the_rule_table",
    "classify_is_the_least_admissible_class",
    "classification_is_monotone",
    "the_provide_inject_cap_never_lowers_and_is_already_implied",
  ]) {
    assertHarness(lattice, name);
  }
  assert.match(
    readRepoFile("davinci", "vize_l3", "tests", "lattice.rs"),
    /^fn the_verdict_never_changes_the_class_and_only_proven_facts_fire\(\)/mu,
  );
  const root = ["tests", "davinci_test_support", "src", "l3_reference"];
  const reference = harnesses(
    [...root, "schedule_proofs.rs"],
    [...root, "checker_proofs.rs"],
    [...root, "incremental_proofs.rs"],
  );
  for (const name of [
    "accepted_schedule_orders_edges",
    "accepted_edges_stay_in_scope",
    "accepted_regrouping_preserves_edges",
    "the_checker_reports_exactly_the_violated_invariants",
    "the_patched_tree_displays_the_view",
    "an_update_is_recompute_and_reorders_allocate_nothing",
    "top_level_identities_are_retained_or_fresh",
    "allocation_is_exactly_the_inserted_delta",
  ]) {
    assertHarness(reference, name);
  }
  const kani = readRepoFile(".github", "workflows", "davinci-kani.yml");
  assert.match(kani, /args: davinci\/vize_l3\/src\/lattice\/kani\.rs /u);
  assert.match(kani, /args: tests\/davinci_test_support\/src\/l3_reference\/kani\.rs /u);
  const checker = readRepoFile(...root, "checker.rs");
  assert.doesNotMatch(
    checker,
    /\bloop\b|\bwhile\b|\bunsafe\b/u,
    "the graph checker must stay total",
  );
  const bridge = readRepoFile("davinci", "vize_l2_to_l3", "tests", "l3_reference_fixture.rs");
  assert.match(bridge, /^    mod schedule;$/mu);
  assert.match(bridge, /^    mod reference;$/mu);
  const lattices = readRepoFile("davinci", "vize_l3", "tests", "lattice_reference_fixture.rs");
  assert.match(lattices, /tests\/fixtures\/l3-reference\/reactivity-lattice\.folio/u);
  assert.match(lattices, /ReactivityPage::of\(&facts\)\.print_to_string\(DumpMode::Full\)/u);
  assert.match(lattices, /lattice::negative_tests\(&text\)/u);
});
