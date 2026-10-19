import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { planNativeRuntime } from "../../tools/support/compat/github/plan-native-runtime.ts";
import { RUST_TEST_COHORT_IDS } from "../../tools/support/compat/github/rust-test-cohorts.ts";
import { readRepoFile, requireDailySourceContracts } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  env?: Record<string, string>;
  with?: Record<string, string | boolean>;
  "continue-on-error"?: boolean;
};
type Job = {
  uses?: string;
  "timeout-minutes"?: number;
  name?: string;
  permissions?: Record<string, string>;
  if?: string;
  needs?: string[] | string;
  steps?: Step[];
  outputs?: Record<string, string>;
  strategy?: { "fail-fast": boolean; matrix: { shard: number[] } | string };
  with?: Record<string, string>;
  "continue-on-error"?: boolean;
};
const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  jobs: Record<string, Job>;
};
const rust = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml")) as {
  jobs: Record<string, Job>;
  on: {
    workflow_call: {
      inputs: Record<string, { type: string; required: boolean; default?: boolean }>;
    };
  };
};
const products = parse(readRepoFile(".github", "workflows", "pr-rust-products.yml")) as {
  jobs: Record<string, Job>;
  on: {
    workflow_call: {
      inputs: Record<string, { type: string; required: boolean; default?: boolean }>;
    };
  };
};

test("source planning uses Node and offline metadata before selecting affected products", () => {
  const plan = source.jobs["pr-source-plan"];
  const steps = plan.steps ?? [];
  const setup = steps.findIndex((step) => step.uses?.startsWith("actions/setup-node@"));
  const sourcePlan = steps.findIndex((step) => step.name === "Plan source checks");
  assert.ok(setup >= 0 && setup < sourcePlan);
  assert.equal(steps[setup].with?.["node-version-file"], "package.json");
  assert.equal(steps[setup].with?.["package-manager-cache"], false);
  assert.equal(steps[setup].with?.cache, undefined);
  assert.ok(steps.every((step) => !step.uses?.includes("setup-vp@")));
  assert.ok(steps.every((step) => !step.uses?.includes("rust-toolchain@")));
  assert.ok(steps.every((step) => !/\b(?:vp|pnpm|npm)(?=\s|$)/.test(step.run ?? "")));
  assert.equal(steps.filter((step) => step.run?.includes("plan-pr-checks.ts")).length, 1);
  assert.match(steps[sourcePlan].run ?? "", /plan-pr-checks\.ts/);
  assert.ok(steps.every((step) => !step.run?.includes("rust-plan.json")));
  assert.equal(plan.outputs?.["comparison-base"], "${{ steps.comparison.outputs.base }}");
  assert.equal(steps[sourcePlan].env?.BASE_SHA, "${{ steps.comparison.outputs.base }}");
  assert.equal(
    source.jobs["pr-rust-source"].with?.["rust-plan"],
    "${{ needs.pr-source-plan.outputs.rust-plan }}",
  );
  assert.equal(
    source.jobs["pr-js-packages"].with?.["js-plan"],
    "${{ needs.pr-source-plan.outputs.js-plan }}",
  );
  assert.equal(plan.outputs?.tooling, "${{ steps.plan.outputs.tooling }}");
  assert.ok(steps.every((step) => step.name !== "Test CI selection when its inputs change"));
  const selectionJob = source.jobs["ci-selection-tests"];
  assert.equal(selectionJob.needs, "pr-source-plan");
  assert.ok(source.jobs["source-report"].needs?.includes("ci-selection-tests"));
  const ciSelection = selectionJob.steps?.find(
    (step) => step.name === "Test CI selection when its inputs change",
  );
  assert.equal(ciSelection?.if, "${{ needs.pr-source-plan.outputs.tooling == 'true' }}");
  assert.equal(
    ciSelection?.run,
    "node --test tests/tooling/ci-iteration-plans.test.ts tests/tooling/pr-required-rust.test.ts tests/tooling/rust-test-cohorts.test.ts tests/tooling/rust-build-evidence.test.mjs tests/tooling/pinned-fixture-hydration.test.ts tests/tooling/native-preparation-identity.test.ts tests/tooling/native-formatter-build-custody.test.mjs tests/tooling/native-ci-formatter-custody.test.mjs tests/tooling/native-history-profile.test.ts tests/tooling/publish-npm-package*.test.ts tests/tooling/davinci-canonical-corpus-kernel-prefix.test.ts",
  );
  const check = parse(readRepoFile(".github", "workflows", "check.yml")) as typeof source;
  const js = check.jobs["check-js"].steps ?? [];
  const install = js.findIndex((step) => step.name === "Install JS dependencies");
  const fast = js.findIndex((step) => step.name === "Check fast JS/TS");
  assert.ok(install >= 0 && fast > install, "The real source law needs installed YAML");
  assert.equal(
    js[fast].if,
    "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' || github.event_name == 'push' }}",
  );
  assert.equal(js[fast]["continue-on-error"], undefined);
  assert.equal(
    js[fast].run,
    "vp run --workspace-root check:iteration && node --test tests/tooling/github-workflows-source-selection.test.ts tests/tooling/vp-bootstrap-cache.test.mjs",
  );
});

test("the existing required Rust source job runs the original three contracts with strict defaults", () => {
  requireDailySourceContracts(source, rust);
  for (const mutation of [
    "default-false",
    "step-skip",
    "step-advisory",
    "job-skip",
    "job-advisory",
    "report-detach",
    "wrong-command",
  ]) {
    const changed = structuredClone(rust);
    const job = changed.jobs["pr-rust-build"];
    const step = job.steps!.find(
      (candidate) =>
        candidate.name === "Require current-tree source, API and reviewed-doc contracts",
    )!;
    if (mutation === "default-false")
      changed.on.workflow_call.inputs["run-source-contracts"].default = false;
    if (mutation === "step-skip") step.if = "${{ false }}";
    if (mutation === "step-advisory") step["continue-on-error"] = true;
    if (mutation === "job-skip") job.if = "${{ false }}";
    if (mutation === "job-advisory") job["continue-on-error"] = true;
    if (mutation === "report-detach") changed.jobs["rust-source-report"].needs = ["pr-rust-shard"];
    if (mutation === "wrong-command") step.run = "echo contracts passed";
    assert.throws(
      () => requireDailySourceContracts(source, changed),
      assert.AssertionError,
      mutation,
    );
  }
});

test("affected native runtime proofs execute independently of the scheduled tooling campaign", () => {
  const plan = source.jobs["pr-source-plan"];
  assert.equal(
    plan.outputs?.["native-runtime-matrix"],
    "${{ steps.plan.outputs.native-runtime-matrix }}",
  );
  assert.equal(source.jobs["pr-tooling-scripts"], undefined);
  const job = source.jobs["pr-native-runtime"];
  assert.equal(job.needs, "pr-source-plan");
  assert.equal(job.strategy?.["fail-fast"], false);
  assert.equal(
    job.strategy?.matrix,
    "${{ fromJSON(needs.pr-source-plan.outputs.native-runtime-matrix) }}",
  );
  const steps = job.steps ?? [];
  for (const [lane, action] of [
    ["ssr", "test-native-ssr"],
    ["scoped-css", "test-native-scoped-css"],
    ["setup", "test-native-js-setup"],
    ["vapor", "test-native-vapor"],
    ["attributes", "test-native-attribute-values-7502"],
    ["selected-template", "test-native-selected-template"],
  ]) {
    const callers = steps.filter((step) => step.uses === `./.github/actions/${action}`);
    assert.equal(callers.length, 1, lane);
    assert.equal(callers[0].if, `\${{ matrix.lane == '${lane}' }}`);
    assert.notEqual(callers[0]["continue-on-error"], true, lane);
  }
  const install = steps.find((step) => step.name === "Install native runtime oracle dependencies");
  assert.equal(install?.if, "${{ matrix.lane != 'none' }}");
  assert.match(install?.run ?? "", /vp-install\.ts.*npm\/native.*tests/);
  const skip = steps.find((step) => step.name === "Explain unrelated skip");
  assert.equal(skip?.if, "${{ matrix.lane == 'none' }}");
  const report = source.jobs["source-report"];
  assert.equal(report.if, "${{ always() && !contains(needs.*.result, 'cancelled') }}");
  assert.ok(report.needs?.includes("pr-native-runtime"));
  assert.equal(
    report.steps?.at(-1)?.run,
    "node tools/support/compat/github/canonical-corpus-selection.mjs --check",
  );
});

test("opted-in PR and merge browser qualification retain tests and enforce merge VRT outcomes", () => {
  const plan = source.jobs["pr-source-plan"];
  assert.equal(plan.outputs?.heavy, "${{ steps.plan.outputs.heavy }}");
  assert.equal(plan.outputs?.playground, "${{ steps.plan.outputs.playground }}");
  assert.equal(
    plan.outputs?.["canonical-corpus"],
    "${{ steps.plan.outputs.heavy == 'true' && steps.canonical-corpus-plan.outputs.canonical-corpus || 'false' }}",
  );
  const tier = plan.steps?.find((step) => step.name === "Plan source checks");
  assert.equal(tier?.env?.PR_LABELS, "${{ toJSON(github.event.pull_request.labels.*.name) }}");
  assert.match(tier?.run ?? "", /plan-pr-checks\.ts/);
  const steps = source.jobs["pr-playground-test"].steps ?? [];
  const browser = steps.find((step) => step.name === "Run playground tests");
  assert.equal(browser?.if, "${{ env.RUN_PLAYGROUND == 'true' }}");
  for (const name of [
    "Run VRT",
    "Upload VRT report",
    "Upload VRT diff artifacts",
    "Fail if VRT failed",
  ]) {
    const step = steps.find((candidate) => candidate.name === name);
    assert.ok(step?.if?.includes("github.event_name == 'merge_group'"), name);
  }
});

test("the Rust report waits for the builder and every named test cohort", () => {
  const shard = products.jobs["pr-rust-shard"];
  // PR and queue cohorts build in place beside the lanes.
  assert.equal(shard.needs, undefined);
  assert.equal(shard.strategy?.["fail-fast"], false);
  assert.equal(shard.name, "Rust tests (${{ matrix.name }})");
  assert.equal(shard.strategy?.matrix, "${{ fromJSON(inputs.rust-plan).testMatrix }}");
  const steps = shard.steps ?? [];
  const run = steps.findIndex(
    (step) => step.name === "Build and run the affected Rust test cohort",
  );
  assert.ok(run >= 0);
  assert.match(steps[run].run ?? "", /-E "\$NEXTEST_FILTER"/);
  assert.doesNotMatch(steps[run].run ?? "", /--partition/);
  assert.deepEqual(rust.jobs["rust-source-report"].needs, [
    "merge-rust-differential",
    "pr-rust-build",
  ]);
  assert.deepEqual(products.jobs["rust-product-report"].needs, ["pr-rust-shard"]);
  assert.equal(
    products.jobs["rust-product-report"].if,
    "${{ always() && !contains(needs.*.result, 'cancelled') }}",
  );
  const reportSteps = products.jobs["rust-product-report"].steps ?? [];
  const gate = reportSteps.findIndex((step) => step.name === "Require the complete Rust tier");
  const gateRuntime = reportSteps.findIndex((step) =>
    step.uses?.startsWith("voidzero-dev/setup-vp@"),
  );
  assert.ok(gateRuntime >= 0 && gateRuntime < gate);
  assert.equal(reportSteps[gateRuntime].if, undefined);
  assert.equal(
    reportSteps.filter((step) => step.uses?.startsWith("voidzero-dev/setup-vp@")).length,
    1,
  );
  assert.deepEqual(reportSteps[gateRuntime].with, {
    "node-version-file": "package.json",
    "run-install": false,
  });
  assert.equal(gate, reportSteps.length - 1, "no worker selection follows the tier gate");
  const check = parse(readRepoFile(".github", "workflows", "check.yml"));
  assert.deepEqual(check.jobs["pr-source-checks"].permissions, {
    contents: "read",
    actions: "read",
    checks: "read",
    "pull-requests": "read",
  });
  for (const job of [source.jobs["pr-rust-source"], rust.jobs["rust-source-report"]])
    assert.deepEqual(job.permissions, { contents: "read", actions: "read" });
  assert.equal(reportSteps[gate].if, undefined);
  assert.equal(
    reportSteps[gate].run,
    "node tools/support/compat/github/require-rust-tier.ts --products",
  );
  assert.deepEqual(reportSteps[gate].env, {
    RUN_RUST: "${{ inputs.run-rust }}",
    NEEDS_JSON: "${{ toJSON(needs) }}",
    GH_TOKEN: "${{ github.token }}",
  });
});

test("unchanged queue gate hydrates the same pinned fixture for all five full Rust cohorts", () => {
  const job = products.jobs["pr-rust-shard"];
  assert.ok(job.steps);
  const step = job.steps.find(
    (value) => value.name === "Hydrate the required ElementPlus projection fixture",
  )!;
  assert.equal(step.uses, "./.github/actions/hydrate-pinned-fixture");
  assert.equal(
    step.if,
    "${{ github.event_name == 'merge_group' && contains(fromJSON(inputs.rust-plan).packages, 'vize_canon') }}",
  );
  assert.deepEqual(step.with, { fixture: "tests/_fixtures/_git/element-plus" });
  assert.equal(step["continue-on-error"], undefined);
  assert.equal(job.needs, undefined);
  assert.deepEqual(job.strategy, {
    "fail-fast": false,
    matrix: "${{ fromJSON(inputs.rust-plan).testMatrix }}",
  });
  assert.deepEqual(RUST_TEST_COHORT_IDS, [
    "compiler",
    "typechecker",
    "language-server",
    "lint-and-format",
    "cli-and-integrations",
  ]);
  const action = parse(
    readRepoFile(".github", "actions", "hydrate-pinned-fixture", "action.yml"),
  ) as {
    runs: { steps: Record<string, unknown>[] };
  };
  const selection = source.jobs["ci-selection-tests"].steps?.find(
    (step) => step.name === "Test CI selection when its inputs change",
  );
  assert.ok(selection);
  assert.equal(selection.if, "${{ needs.pr-source-plan.outputs.tooling == 'true' }}");
  assert.equal(selection["continue-on-error"], undefined);
  assert.deepEqual(selection.run?.split(/\s+/), [
    "node",
    "--test",
    "tests/tooling/ci-iteration-plans.test.ts",
    "tests/tooling/pr-required-rust.test.ts",
    "tests/tooling/rust-test-cohorts.test.ts",
    "tests/tooling/rust-build-evidence.test.mjs",
    "tests/tooling/pinned-fixture-hydration.test.ts",
    "tests/tooling/native-preparation-identity.test.ts",
    "tests/tooling/native-formatter-build-custody.test.mjs",
    "tests/tooling/native-ci-formatter-custody.test.mjs",
    "tests/tooling/native-history-profile.test.ts",
    "tests/tooling/publish-npm-package*.test.ts",
    "tests/tooling/davinci-canonical-corpus-kernel-prefix.test.ts",
  ]);
  assert.equal(action.runs.steps.length, 1);
  assert.equal(
    action.runs.steps[0]!.run,
    'node "$GITHUB_ACTION_PATH/hydrate-pinned-fixture.ts" "$GITHUB_WORKSPACE" "$FIXTURE_PATH"',
  );
  assert.equal(action.runs.steps[0]!["continue-on-error"], undefined);
});

test("existing source CLI producer qualifies every formatter semantic case with strict custody", () => {
  const continuation = "tests/tooling/formatter-continuation-prefix-width.test.mjs";
  const buildRun =
    "cargo build --locked --profile ci -p vize\nnode tests/differential/build-receipt.ts\n";
  const dependenciesRun =
    "node tools/support/github/vp-install.ts --filter vize-workspace --filter './npm/native...' --filter './npm/ui...' --filter './tests...'";
  const lspName = "Test actual source LSP feature and project responses";
  const replayName = "Replay complete formatter CLI stock semantics with the source CLI";
  const uploadName = "Preserve complete source formatter semantic observations";
  type Action = { runs: { steps: Step[] } };
  const action = parse(readRepoFile(".github/actions/test-native-lsp/action.yml")) as Action;
  const check = parse(readRepoFile(".github", "workflows", "check.yml")) as typeof source;
  const requireReplay = (flow: typeof source, producer: typeof action, outer: typeof check) => {
    const job = flow.jobs["pr-native-runtime"];
    const steps = job.steps ?? [];
    const deps = steps.find((step) => step.name === "Install native runtime oracle dependencies");
    const lsp = steps.find((step) => step.name === lspName);
    const replay = steps.find((step) => step.name === replayName);
    const upload = steps.find((step) => step.name === uploadName);
    assert.ok(deps && lsp && replay && upload);
    assert.equal(deps.run, dependenciesRun);
    assert.equal(lsp.uses, "./.github/actions/test-native-lsp");
    assert.equal(lsp.if, "${{ matrix.lane == 'lsp' }}");
    assert.equal(lsp["continue-on-error"], undefined);
    assert.equal(producer.runs.steps[0].if, undefined);
    assert.equal(producer.runs.steps[0]["continue-on-error"], undefined);
    assert.equal(job.if, undefined);
    assert.equal(producer.runs.steps[0].run, buildRun);
    assert.ok(steps.indexOf(deps) < steps.indexOf(lsp));
    assert.ok(steps.indexOf(lsp) < steps.indexOf(replay));
    assert.ok(steps.indexOf(replay) < steps.indexOf(upload));
    assert.equal(replay.if, "${{ success() && matrix.lane == 'lsp' }}");
    const recipe = replay.run ?? "";
    // These closed commands have no quoted arguments; never normalize shell literals.
    assert.doesNotMatch(recipe, /^[ \t]*#.*\\\r?$/m);
    assert.doesNotMatch(recipe.replace(/^[ \t]*#.*$/gm, ""), /['"`]/);
    assert.deepEqual(
      recipe
        .replace(/\\\r?\n/g, "")
        .split(/\r?\n/)
        .filter((line) => /[^ \t]/.test(line) && !/^[ \t]*#/.test(line))
        .map((line) => line.replace(/[ \t]+/g, " ").replace(/^ | $/g, "")),
      [
        "set -euo pipefail",
        "mkdir -p target/differential",
        "node --test tests/tooling/formatter-expression-width-semantics.test.mjs tests/tooling/formatter-continuation-prefix-width.test.mjs 2>&1 | tee target/differential/formatter-source.log",
      ],
    );
    assert.equal(replay.env, undefined);
    assert.equal(replay["continue-on-error"], undefined);
    assert.equal(Object.hasOwn(replay, "timeout-minutes"), false);
    assert.equal(job["timeout-minutes"], 15);
    assert.equal(upload.if, "${{ always() && matrix.lane == 'lsp' }}");
    assert.equal(upload.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
    assert.deepEqual(upload.with, {
      name: "source-formatter-semantics-${{ github.run_id }}-${{ github.run_attempt }}-${{ github.sha }}",
      path: "target/differential/formatter-expression-width-semantics.json\ntarget/differential/formatter-continuation-prefix-width.json\ntarget/differential/formatter-source.log\ntarget/ci/vize.differential-build.json\n",
      "if-no-files-found": "error",
      "retention-days": 7,
    });
    assert.equal(upload["continue-on-error"], undefined);
    assert.equal(job["continue-on-error"], undefined);
    assert.equal(job.needs, "pr-source-plan");
    assert.ok(flow.jobs["source-report"].needs?.includes("pr-native-runtime"));
    assert.equal(outer.jobs["pr-source-checks"].uses, "./.github/workflows/pr-source-checks.yml");
    assert.ok(outer.jobs["test-report"].needs?.includes("pr-source-checks"));
  };
  requireReplay(source, action, check);
  for (const path of [
    ".github/workflows/pr-source-checks.yml",
    "tests/tooling/formatter-expression-width-semantics.test.mjs",
    "tests/tooling/formatter-continuation-prefix-width.test.mjs",
    "tests/tooling/support/formatter-expression-width-semantics.mjs",
    "tests/tooling/support/formatter-continuation-semantics.mjs",
    "tests/tooling/support/sfc-equivalence.ts",
    "tests/tooling/support/babel-expression-signature.ts",
    "tests/tooling/support/sfc-equivalence/blocks.ts",
    "tests/tooling/support/sfc-opening-tag.ts",
    "tests/_fixtures/differential/formatter-regressions/expression-print-width-7876/corpus.json",
    "tests/_fixtures/differential/formatter-regressions/continuation-prefix-width-7876/original-example.input",
  ])
    assert.ok(planNativeRuntime([path]).matrix.lane.includes("lsp"), path);
  for (const path of [
    "README.md",
    "tests/tooling/formatter-continuation-reference.test.mjs",
    "tests/tooling/support/sfc-equivalence.ts.map",
    "tests/tooling/support/babel-expression-signature.ts.map",
    "tests/tooling/support/sfc-equivalence/blocks.ts.map",
    "tests/tooling/support/sfc-opening-tag.ts.map",
    "tests/_fixtures/differential/formatter-regressions/continuation-prefix-width-78760/corpus.json",
  ])
    assert.deepEqual(planNativeRuntime([path]).matrix.lane, ["none"], path);
  for (const kind of "producer producer-skip job-skip dependency caller skip partial advisory provider upload detached formatting quoted inline continued-comment heredoc comment".split(
    " ",
  )) {
    const flow = structuredClone(source),
      producer = structuredClone(action),
      outer = structuredClone(check);
    const steps = flow.jobs["pr-native-runtime"].steps!;
    const replay = steps.find((step) => step.name === replayName)!;
    const upload = steps.find((step) => step.name === uploadName)!;
    if (kind === "producer")
      producer.runs.steps[0].run = "node tests/differential/build-receipt.ts\n";
    if (kind === "producer-skip") producer.runs.steps[0].if = "${{ false }}";
    if (kind === "job-skip") flow.jobs["pr-native-runtime"].if = "${{ false }}";
    if (kind === "dependency")
      steps.find((step) => step.name === "Install native runtime oracle dependencies")!.run =
        "vp install --filter './tests...'";
    if (kind === "caller")
      steps.find((step) => step.name === lspName)!.uses = "./.github/actions/borrowed-cli";
    if (kind === "skip") replay.if = "${{ false }}";
    if (kind === "partial")
      replay.run = replay.run!.replace(continuation, "--test-name-pattern partial");
    if (kind === "advisory") replay["continue-on-error"] = true;
    if (kind === "provider") replay.env = { PUBLIC_CONSUMER: "/borrowed" };
    if (kind === "upload") upload.if = "${{ success() }}";
    if (kind === "detached") outer.jobs["test-report"].needs = [];
    if (kind === "formatting")
      replay.run = `# "comment"\n\n${replay.run!.replace("--test ", "--test \\\n ")}`;
    if (kind === "quoted")
      replay.run = replay.run!.replace("node --test ", 'node --test "  literal whitespace  " ');
    if (kind === "inline")
      replay.run = replay.run!.replace("--test ", "--test # inline changes argv ");
    if (kind === "continued-comment")
      replay.run = replay.run!.replace("--test ", "--test \\\n# comment\n ");
    if (kind === "heredoc")
      replay.run = replay.run!.replace("--test ", "--test <<EOF\npayload\nEOF\n");
    if (kind === "comment") replay.run = "# \\\nUNEXPECTED_COMMAND\n" + replay.run;
    if (kind === "formatting") requireReplay(flow, producer, outer);
    else assert.throws(() => requireReplay(flow, producer, outer), assert.AssertionError, kind);
  }
});
