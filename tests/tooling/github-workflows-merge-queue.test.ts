import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.ts";
import { rustDifferentialLanes } from "../../tools/support/compat/github/rust-differential-lanes.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";
type Step = {
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  env?: Record<string, string>;
  with?: Record<string, string>;
  "continue-on-error"?: boolean;
};
type Job = {
  if?: string;
  needs?: string[] | string;
  steps?: Step[];
  uses?: string;
  env?: Record<string, string>;
  strategy?: { matrix?: unknown };
};
const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  jobs: Record<string, Job>;
};
const check = parse(readRepoFile(".github", "workflows", "check.yml")) as {
  jobs: Record<string, Job>;
};
const jsChecks = parse(readRepoFile(".github", "workflows", "pr-js-checks.yml")) as {
  jobs: Record<string, Job>;
};
const recipe = parse(
  readRepoFile(".github", "actions", "test-rust-workspace-differential", "action.yml"),
) as {
  inputs: Record<string, { default: string }>;
  runs: { using: string; steps: Step[] };
};
const rust = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml")) as {
  jobs: Record<string, Job>;
};
const differential = parse(readRepoFile(".github", "workflows", "pr-rust-differential.yml")) as {
  jobs: Record<string, Job>;
};
const actionPath = "./.github/actions/test-rust-workspace-differential";
const lanes = [
  "pr-source-plan",
  "pr-rust-source",
  "pr-js-packages",
  "pr-native-runtime",
  "pr-playground-test",
];
const tailCommands = [
  "cargo test -p vize_l1 --features legacy-differential --test surface_corpus -- --nocapture",
  "cargo test -p vize_l1_to_l2 --features legacy-differential --test davinci_lowering_corpus -- --nocapture",
  "cargo test -p vize_l1_to_l2 --features legacy-differential --test davinci_dom_corpus -- --nocapture",
  "cargo test -p vize_l1_to_l2 --features legacy-differential --test davinci_remarks_corpus -- --nocapture",
  "cargo test -p vize_l1_to_l2 --features legacy-differential --test davinci_pug_corpus -- --nocapture",
  "cargo test -p vize_atelier_sfc --features legacy-dom-differential --test davinci_production_reach -- --nocapture",
  'VIZE_DAVINCI_DIFFERENTIAL_CORPUS="$PWD" cargo test -p vize_atelier_ssr --features legacy-differential --test davinci_ssr_corpus -- --nocapture',
  "cargo test -p vize_atelier_jsx --features legacy-differential --lib vdom::l2_differential::l2_vdom_admitted_cases_match_relief_codegen -- --exact",
  "cargo test -p vize --features legacy --test check_nuxt_tsconfig_isolation_cli",
  "cargo test -p vize_atelier_core --features legacy --test davinci_l2_transform_vue2",
  "cargo test -p vize_vitrine --no-default-features --features wasm",
  'VIZE_DAVINCI_DIFFERENTIAL_CORPUS="$PWD" cargo test -p vize_patina --features legacy-differential --test davinci_markup_differential -- --nocapture',
];
const jsHistoryPath = "./.github/actions/test-js-packages-with-history";
const jsHistory = parse(readRepoFile(jsHistoryPath, "action.yml")) as {
  runs: { using: string; steps: Step[] };
};
const commands = (job: Job) =>
  (job.steps ?? [])
    .flatMap((step) => (step.uses === jsHistoryPath ? jsHistory.runs.steps : [step]))
    .map((step) => step.run ?? "")
    .join("\n");
test("queue scope reaches the planner and every selected source lane remains required", () => {
  assert.equal(
    source.jobs["pr-source-plan"].steps?.find((step) => step.name === "Plan source checks")?.run,
    'node tools/support/compat/github/plan-pr-checks.ts "$BASE_SHA" "$GITHUB_SHA" "$GITHUB_EVENT_NAME"',
  );
  assert.deepEqual(source.jobs["source-report"].needs, [
    ...lanes,
    "wit-contracts",
    "canonical-corpus",
  ]);
  assert.equal(
    source.jobs["source-report"].if,
    "${{ always() && !contains(needs.*.result, 'cancelled') }}",
  );
  assert.match(commands(source.jobs["source-report"]), /canonical-corpus-selection\.mjs/);
  assert.ok(check.jobs["test-report"].needs?.includes("pr-source-checks"));
  for (const required of [
    "iteration-plan",
    "fmt-rust",
    "check-js",
    "security-audit",
    "node-engine-compat",
    "check-vize-apps",
  ]) {
    assert.ok(check.jobs["test-report"].needs?.includes(required), `${required} remains mandatory`);
  }
  assert.equal(check.jobs["pr-source-checks"].uses, "./.github/workflows/pr-source-checks.yml");
  assert.equal(source.jobs["pr-js-packages"].uses, "./.github/workflows/pr-js-checks.yml");
  const jsCommands = Object.values(jsChecks.jobs).map(commands).join("\n");
  assert.match(jsCommands, /vp run --workspace-root "test:js:\$VIZE_JS_PACKAGE_GROUP"/);
  assert.match(jsCommands, /node npm\/oxlint\/scripts\/check-project-transport\.mjs/);
  assert.match(jsCommands, /vp run --filter '\.\/npm\/ui' check/);
  assert.equal(jsHistory.runs.using, "composite");
  assert.equal(jsHistory.runs.steps[0].if, "${{ success() && job.status == 'success' }}");
  assert.ok(check.jobs["test-js-packages"].steps?.some((step) => step.uses === jsHistoryPath));
  const nativeRuntime = source.jobs["pr-native-runtime"];
  assert.equal(nativeRuntime.needs, "pr-source-plan");
  assert.equal(nativeRuntime.if, undefined, "unaffected runtime inputs still report success");
  for (const [lane, action] of [
    ["canon", "test-native-canon"],
    ["lsp", "test-native-lsp"],
    ["ssr", "test-native-ssr"],
    ["scoped-css", "test-native-scoped-css"],
    ["setup", "test-native-js-setup"],
    ["vapor", "test-native-vapor"],
    ["attributes", "test-native-attribute-values-7502"],
    ["selected-template", "test-native-selected-template"],
  ]) {
    const step = nativeRuntime.steps?.find((step) => step.uses === `./.github/actions/${action}`);
    assert.ok(step, lane);
    assert.equal(step.if, `\${{ matrix.lane == '${lane}' }}`);
    assert.notEqual(step["continue-on-error"], true);
  }
  assert.match(commands(source.jobs["pr-playground-test"]), /test:browser/);
  assert.match(commands(source.jobs["pr-playground-test"]), /test:vrt/);
  const vrtFailure = source.jobs["pr-playground-test"].steps?.find(
    (step) => step.name === "Fail if VRT failed",
  );
  assert.match(vrtFailure?.if ?? "", /steps\.vrt\.outcome == 'failure'/);
  assert.equal(vrtFailure?.run, "exit 1");
});
test("queue Rust keeps affected doctests beside archive builds and the differential feature tail", () => {
  assert.equal(source.jobs["pr-rust-source"].uses, "./.github/workflows/pr-rust-checks.yml");
  const build = rust.jobs["pr-rust-build"];
  assert.equal(build.needs, undefined, "Clippy and doctests must start beside the test archive");
  assert.equal(build.if, "${{ inputs.run-rust }}");
  assert.match(commands(build), /run-affected-rust\.ts[^\n]*cargo clippy @packages@/);
  assert.match(
    commands(build),
    /run-affected-rust\.ts[^\n]*cargo test @packages@ --profile ci --doc/,
  );
  const buildSteps = build.steps ?? [];
  const runtime = buildSteps.findIndex(
    (step) => step.name === "Select Clippy and doctest runtime envelope",
  );
  assert.ok(
    runtime >= 0 &&
      runtime < buildSteps.findIndex((step) => step.name === "Clippy affected crates"),
  );
  assert.match(buildSteps[runtime].run ?? "", /merge_group\) printf 'VIZE_TEST_REQUIRE_TSGO=1/);
  assert.ok(rust.jobs["rust-source-report"].needs?.includes("pr-rust-build"));
  const differentialJob = differential.jobs["differential"];
  const sibling = differentialJob.steps ?? [];
  const selected = sibling.find(
    (step) => step.name === "Run affected feature and production corpora",
  );
  assert.equal(
    differentialJob.strategy?.matrix,
    "${{ fromJSON(inputs.rust-plan).differentialMatrix }}",
  );
  assert.equal(differentialJob.if, "${{ github.event_name == 'merge_group' && inputs.run-rust }}");
  assert.equal(differentialJob.env?.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(selected?.if, "${{ matrix.lane != 'navigation' && matrix.lane != 'none' }}");
  assert.equal(
    selected?.run,
    'node tools/support/compat/github/rust-differential-lanes.ts "$DIFFERENTIAL_LANE" "$RUNNER_TEMP/rust-plan.json"',
  );
  assert.notEqual(selected?.["continue-on-error"], true);
  assert.equal(rust.jobs["merge-rust-differential"].needs, undefined);
  assert.equal(
    rust.jobs["merge-rust-differential"].uses,
    "./.github/workflows/pr-rust-differential.yml",
  );
  const manual = check.jobs["clippy-and-test"].steps?.find((step) => step.name === "Test");
  assert.equal(manual?.uses, actionPath);
  assert.equal(manual?.with?.["workspace-already-tested"], undefined);
  assert.equal(recipe.inputs["workspace-already-tested"].default, "false");
  assert.equal(recipe.runs.using, "composite");
  const workspaceStep = recipe.runs.steps.find((step) => step.name === "Test Rust workspace");
  assert.ok(workspaceStep);
  assert.equal(workspaceStep.if, "${{ inputs.workspace-already-tested != 'true' }}");
  assert.equal(workspaceStep.run, "node tools/support/compat/github/run-rust-workspace-tests.mjs");
  const hydration = recipe.runs.steps[0];
  assert.equal(
    hydration.run,
    "git submodule update --init --depth 1 -- tests/_fixtures/_git/element-plus",
  );
  assert.equal(hydration.if, undefined);
  assert.notEqual(hydration["continue-on-error"], true);
  for (const step of recipe.runs.steps.slice(1)) {
    assert.equal(step.env?.VIZE_TEST_REQUIRE_TSGO, "1");
    assert.notEqual(step["continue-on-error"], true);
  }
  // Batched --test flags may combine processes, but every prior feature target
  // must remain selected once with its package, feature set and test arguments.
  const partitioned = rustDifferentialLanes.flatMap((lane) => lane.steps);
  for (const command of tailCommands) {
    const expected = command.slice(command.indexOf("cargo ") + 6).split(" ");
    const matches = partitioned.filter((step) => {
      let position = 0;
      for (const argument of step.args) if (argument === expected[position]) position += 1;
      return position === expected.length;
    });
    assert.equal(matches.length, 1, command);
  }
});
test("both queue reports reject failed, cancelled, skipped and missing actual dependencies", () => {
  for (const job of [source.jobs["source-report"], check.jobs["test-report"]]) {
    const dependencies = job.needs;
    assert.ok(Array.isArray(dependencies) && dependencies.length > 0);
    const needs = () =>
      Object.fromEntries(dependencies.map((lane) => [lane, { result: "success" }]));
    assert.equal(aggregateNeedsResults(needs()).exitCode, 0);
    for (const lane of dependencies) {
      for (const result of ["failure", "cancelled", "skipped", "unknown"]) {
        assert.equal(aggregateNeedsResults({ ...needs(), [lane]: { result } }).exitCode, 1);
      }
      assert.throws(() => aggregateNeedsResults({ ...needs(), [lane]: {} }), /reported no result/);
    }
  }
});
test("the shared bash recipe stops at a failed feature command (simulated cargo)", () => {
  const differentialStep = recipe.runs.steps.find(
    (step) => step.name === "Test feature-enabled differential corpora",
  );
  assert.ok(differentialStep);
  const cwd = mkdtempSync(join(tmpdir(), "vize-queue-recipe-"));
  try {
    const log = join(cwd, "commands.log");
    const argvLog = join(cwd, "arguments.log");
    writeFileSync(
      join(cwd, "cargo"),
      '#!/bin/sh\nprintf "%s|%s|%s\\n" "$*" "$VIZE_TEST_REQUIRE_TSGO" "${VIZE_DAVINCI_DIFFERENTIAL_CORPUS-}" >> "$VIZE_GATE_TEST_LOG"\nprintf "%s\\0" "$@" >> "$VIZE_GATE_TEST_ARGV_LOG"\nprintf "\\n" >> "$VIZE_GATE_TEST_ARGV_LOG"\ncase "$*" in *"$VIZE_GATE_TEST_FAIL"*) exit 42;; esac\n',
      { mode: 0o755 },
    );
    for (const [failure, count, status] of [
      ["surface_corpus", 1, 42],
      ["davinci_remarks_corpus", 4, 42],
      ["no-such-command", 12, 0],
    ] as const) {
      writeFileSync(log, "");
      writeFileSync(argvLog, "");
      const run = spawnSync(
        "/bin/bash",
        ["--noprofile", "--norc", "-eo", "pipefail", "-c", differentialStep.run!],
        {
          cwd: root,
          encoding: "utf8",
          env: {
            PATH: cwd,
            VIZE_TEST_REQUIRE_TSGO: "1",
            VIZE_GATE_TEST_LOG: log,
            VIZE_GATE_TEST_ARGV_LOG: argvLog,
            VIZE_GATE_TEST_FAIL: failure,
          },
        },
      );
      assert.equal(run.status, status, run.stderr);
      const lines = readFileSync(log, "utf8").trim().split("\n");
      assert.equal(lines.length, count);
      const actualArguments = readFileSync(argvLog, "utf8")
        .trimEnd()
        .split("\n")
        .map((line) => line.split("\0").slice(0, -1));
      const expectedArguments = tailCommands
        .slice(0, count)
        .map((command) => command.slice(command.indexOf("cargo ") + 6).split(" "));
      assert.deepEqual(actualArguments, expectedArguments);
      assert.ok(lines.every((line) => line.includes("|1|")));
      if (status === 0) {
        assert.ok(lines[6].endsWith(`|${root}`));
        assert.ok(lines[11].endsWith(`|${root}`));
      }
    }
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
