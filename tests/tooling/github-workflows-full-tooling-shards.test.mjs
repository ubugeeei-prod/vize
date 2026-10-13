import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { rustCachePolicy } from "../../.github/actions/setup-rust-sticky-cache/cache-policy.mjs";
import {
  planToolingTests,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import { toolingTestCommand } from "../../tools/support/compat/github/run-tooling-tests.mjs";
import { toolingShardMatrix } from "../../tools/support/compat/github/tooling-test-shards.ts";
import { mergeOnlyToolingTests } from "../../tools/config/vite-plus/tooling-test-scopes.ts";
import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.ts";

const root = new URL("../../", import.meta.url);
const read = (file) => readFileSync(new URL(file, root), "utf8");
const yaml = (file) => parse(read(file));
const fullCondition =
  "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' || (github.event_name == 'push' && github.ref == 'refs/heads/main') }}";
const alwaysFullCondition = fullCondition
  .replace("${{ ", "${{ always() && (")
  .replace(" }}", ") }}");
const originalRecipe =
  "cargo build --profile ci -p vize && vp exec node tests/differential/build-receipt.ts && cat target/ci/vize.differential-build.json && cargo build --profile ci -p vize_atelier_sfc --example cli_build_result_oracle";
const check = () => yaml(".github/workflows/check.yml");
const action = () => yaml(".github/actions/test-full-tooling/action.yml");
const producer = () => yaml(".github/workflows/full-source-cli.yml");
const uses = (steps, name) => steps.filter((step) => step.uses?.startsWith(name));
const compareStrings = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
const workerIndexes = toolingShardMatrix(planToolingTests([], { tier: "merge" })).include.map(
  ({ index }) => index,
);

void test("full events require the original producer, planned isolated workers and the unchanged report deadline", () => {
  const jobs = check().jobs;
  assert.ok(jobs["full-source-cli"], "one original full-suite CLI producer is required");
  assert.equal(jobs["full-source-cli"].uses, "./.github/workflows/full-source-cli.yml");
  assert.equal(jobs["full-source-cli"].if, fullCondition);
  const worker = jobs["test-scripts"];
  assert.equal(worker.if, alwaysFullCondition);
  assert.deepEqual(worker.needs, ["full-source-cli", "tooling-plan"]);
  assert.deepEqual(worker.strategy, {
    "fail-fast": false,
    matrix: "${{ fromJSON(needs.tooling-plan.outputs.matrix) }}",
  });
  assert.equal(worker["timeout-minutes"], 30);
  assert.deepEqual(worker.env, {
    NODE_OPTIONS: "--disable-warning=DEP0040 --disable-warning=DEP0169",
    VIZE_SOURCE_CLI_RECEIPT_SHA256: "${{ needs.full-source-cli.outputs.receipt-sha256 }}",
  });
  assert.equal(worker.steps.length, 2);
  assert.ok(worker.steps[0].uses.startsWith("actions/checkout@"));
  assert.equal(worker.steps[0].with["fetch-depth"], 0, "complete actual shipping-cut ancestry");
  const prTooling = yaml(".github/workflows/pr-source-checks.yml").jobs["pr-tooling-scripts"];
  assert.equal(uses(prTooling.steps, "actions/checkout@")[0].with["fetch-depth"], 0);
  assert.equal(worker.steps[1].uses, "./.github/actions/test-full-tooling");
  assert.deepEqual(worker.steps[1].with, {
    producer: "${{ toJSON(needs.full-source-cli.outputs) }}",
    shard: "${{ matrix.index }}",
    total: "${{ matrix.total }}",
  });
  assert.equal(worker.steps[1].if, undefined);
  assert.equal(worker.steps[1]["continue-on-error"], undefined);
  const report = jobs["full-js-report"];
  assert.equal(report["timeout-minutes"], 5);
  assert.equal(report.if, alwaysFullCondition);
  assert.deepEqual(
    [...report.needs].sort(compareStrings),
    ["test-js-packages", "build-js-packages", "test-scripts", "full-source-cli", "check-js"].sort(
      compareStrings,
    ),
  );
  assert.deepEqual(report.steps.at(-1).with, { "needs-json": "${{ toJSON(needs) }}" });
  const green = Object.fromEntries(report.needs.map((name) => [name, { result: "success" }]));
  assert.equal(aggregateNeedsResults(green).exitCode, 0);
  for (const name of ["full-source-cli", "test-scripts"])
    for (const result of ["failure", "cancelled", "skipped"])
      assert.equal(aggregateNeedsResults({ ...green, [name]: { result } }).exitCode, 1);
});

void test("producer failure retains native failure evidence while strict restore and quorum remain fatal", () => {
  const jobs = check().jobs,
    steps = action().runs.steps;
  assert.equal(jobs["test-scripts"].if, alwaysFullCondition);
  assert.equal(jobs["test-scripts"].steps[1].if, undefined);
  const restore = steps.findIndex((step) => step.uses === "./.github/actions/restore-source-cli");
  assert.ok(restore >= 0);
  assert.equal(steps[restore].if, undefined);
  assert.equal(steps[restore]["continue-on-error"], undefined);
  for (const gate of ["test-native-js-setup", "test-native-attribute-values-7502"]) {
    const index = steps.findIndex((step) => step.uses === `./.github/actions/${gate}`);
    assert.ok(index > restore);
    assert.equal(steps[index].if, "${{ always() && inputs.shard == '1' }}");
    assert.equal(steps[index]["continue-on-error"], undefined);
  }
  for (const result of ["failure", "cancelled", "skipped"])
    assert.equal(
      aggregateNeedsResults({
        "full-source-cli": { result },
        "test-scripts": { result: "success" },
      }).exitCode,
      1,
    );
});

void test("the complete original glob vector is partitioned once with all deferred runtime tests", () => {
  const task = read("tools/config/vite-plus/tasks/test-benchmark.ts");
  const original = [
    ["tests/tooling", ".test.ts"],
    ["tests/tooling", ".test.mjs"],
    ["tests/tooling/davinci", ".test.ts"],
    ["tests/tooling/davinci", ".test.mjs"],
    ["tests/tooling/release", ".test.ts"],
  ].flatMap(([dir, suffix]) =>
    readdirSync(new URL(`${dir}/`, root))
      .filter((file) => file.endsWith(suffix))
      .sort(compareStrings)
      .map((file) => join(dir, file)),
  );
  for (const glob of [
    "tests/tooling/*.test.ts",
    "tests/tooling/*.test.mjs",
    "tests/tooling/davinci/*.test.ts",
    "tests/tooling/davinci/*.test.mjs",
    "tests/tooling/release/*.test.ts",
  ])
    assert.ok(task.includes(`"${glob}"`), `original discovery retained: ${glob}`);
  const files = toolingTestFiles();
  assert.deepEqual([...original].sort(compareStrings), files);
  const full = planToolingTests([], { tier: "merge" });
  assert.equal(full.deferredTests, 0);
  assert.equal(full.tests.length, files.length);
  const total = toolingShardMatrix(full).include.length;
  const shards = Array.from({ length: total }, (_, index) =>
    toolingTestCommand(full, files, `${index + 1}/${total}`),
  );
  for (const args of shards) assert.deepEqual(args.slice(0, 2), ["--test", "--test-concurrency=1"]);
  const union = shards.flatMap((args) => args.slice(2));
  assert.deepEqual([...union].sort(compareStrings), files);
  assert.equal(new Set(union).size, files.length);
  for (const file of mergeOnlyToolingTests) assert.ok(union.includes(file), file);
  for (const tests of [files.slice(1), [...files, files[0]]])
    assert.throws(() => toolingTestCommand({ ...full, tests }, files, `1/${total}`));
  assert.throws(() => toolingTestCommand(full, files, `1/${total + 1}`));
});

void test("the full runner retains serial execution, native preparation, source binding and required TSGO", () => {
  const steps = action().runs.steps;
  const plan = steps.findIndex((step) => step.run?.includes("plan-tooling-tests.mjs"));
  const run = steps.findIndex(
    (step) => step.run === "vp run --workspace-root test:scripts:planned",
  );
  const restore = steps.findIndex((step) => step.uses === "./.github/actions/restore-source-cli");
  assert.ok(restore >= 0 && restore < plan && plan < run);
  assert.deepEqual(steps[restore].with, { producer: "${{ inputs.producer }}" });
  assert.match(steps[plan].run, / 0{40} "\$GITHUB_SHA" --tier merge /);
  assert.deepEqual(steps[run].env, {
    VIZE_LSP_BIN: "${{ github.workspace }}/target/ci/vize",
    VIZE_LSP_REQUIRE_SOURCE_BUILD: "1",
    VIZE_TOOLING_TEST_TIER: "merge",
    VIZE_TOOLING_TEST_PLAN: "${{ runner.temp }}/tooling-plan.json",
    VIZE_TOOLING_TEST_SHARD: "${{ inputs.shard }}/${{ inputs.total }}",
  });
  assert.equal(steps[run]["continue-on-error"], undefined);
  assert.doesNotMatch(
    steps.map((step) => step.run ?? "").join("\n"),
    /cargo build --profile ci -p vize|build-receipt\.ts|ci-opt/,
  );
  const tasks = read("tools/config/vite-plus/tasks/test-benchmark.ts");
  assert.match(
    tasks,
    /"test:scripts": noCacheTask\(\s*`\$\{runTask\("build:native:test"\)\} && rust-script tools\/commands\/ci\/verify-tool-layout\.rs && VIZE_TEST_REQUIRE_TSGO=1 node --test --test-concurrency=1/,
  );
  assert.match(
    tasks,
    /"test:scripts:planned": noCacheTask\(\s*`\$\{runTask\("build:native:test"\)\} && rust-script tools\/commands\/ci\/verify-tool-layout\.rs && node tools\/support\/compat\/github\/run-tooling-tests\.mjs/,
  );
  assert.match(
    read("tools/support/compat/github/run-tooling-tests.mjs"),
    /VIZE_TEST_REQUIRE_TSGO: "1"/,
  );
  for (const name of [
    "./.github/actions/setup-moonbit",
    "./.github/actions/setup-rust-script",
    "./.github/actions/install-formatter-css-browser",
  ])
    assert.equal(uses(steps, name).length, 1);
  const install = steps.find((step) => step.run?.includes("vp install --frozen-lockfile"));
  for (const fixture of ["ant-design-vue", "create-vue", "vue-select", "vue-router"])
    assert.ok(install.run.includes(`tests/_fixtures/_git/${fixture}`));
  assert.ok(install.run.includes("prepare-vue-benchmarks.mjs"));
});

void test("one full-environment producer retains the original ci recipe and strict transport custody", () => {
  const workflow = producer(),
    cli = workflow.jobs.cli,
    steps = cli.steps;
  assert.equal(cli["timeout-minutes"], 30);
  assert.deepEqual(workflow.env, {
    CARGO_TERM_COLOR: "always",
    FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true,
    NODE_OPTIONS: "--disable-warning=DEP0040 --disable-warning=DEP0169",
  });
  assert.equal(cli.env, undefined, "the original default Cargo parallelism is retained");
  const build = steps.findIndex((step) => step.run === originalRecipe);
  const pack = steps.findIndex((step) => step.id === "pack");
  const upload = steps.findIndex((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.ok(build >= 0 && build < pack && pack < upload);
  for (const step of [steps[build], steps[pack], steps[upload]]) {
    assert.equal(step.if, undefined);
    assert.equal(step["continue-on-error"], undefined);
  }
  assert.equal(
    steps[pack].run,
    'node tools/support/compat/github/source-cli-artifact.ts pack "$RUNNER_TEMP/source-cli.tar.gz"',
  );
  assert.equal(
    steps[upload].with.name,
    "source-cli-${{ github.run_id }}-${{ steps.pack.outputs.producer-attempt }}",
  );
  assert.equal(steps[upload].with["if-no-files-found"], "error");
  for (const key of ["sha256", "receipt-sha256", "producer-attempt"]) {
    assert.equal(cli.outputs[key], `\${{ steps.pack.outputs.${key} }}`);
    assert.equal(workflow.on.workflow_call.outputs[key].value, `\${{ jobs.cli.outputs.${key} }}`);
  }
  for (const group of [steps, action().runs.steps]) {
    assert.deepEqual(uses(group, "dtolnay/rust-toolchain@")[0].with, { toolchain: "stable" });
    assert.deepEqual(uses(group, "wild-linker/action@")[0].with, { "wild-version": "0.9.0" });
  }
  const restore = yaml(".github/actions/restore-source-cli/action.yml").runs.steps;
  assert.equal(
    restore[1].with.name,
    "source-cli-${{ github.run_id }}-${{ steps.producer.outputs.producer-attempt }}",
  );
  assert.equal(restore[1].with["digest-mismatch"], "error");
  assert.equal(restore[1].with["run-id"], undefined);
});

void test("native gates and packet consumers retain one owner while every checkout prepares its runtime", () => {
  const steps = action().runs.steps;
  for (const gate of ["test-native-js-setup", "test-native-attribute-values-7502"]) {
    const owned = uses(steps, `./.github/actions/${gate}`);
    assert.equal(owned.length, 1);
    assert.equal(owned[0].if, "${{ always() && inputs.shard == '1' }}");
    assert.equal(owned[0]["continue-on-error"], undefined);
    assert.doesNotMatch(read(`.github/actions/${gate}/action.yml`), /GITHUB_ENV/);
  }
  assert.equal(
    uses(steps, "./.github/actions/check-router-page-types")[0].if,
    "${{ inputs.shard == '1' }}",
  );
  const other = steps.find((step) => step.run?.includes("plugin-sandbox-image.mjs"));
  assert.equal(other.if, "${{ always() && inputs.shard != '1' }}");
  const setup = read(".github/actions/test-native-js-setup/action.yml");
  for (const file of [
    "js-setup-reference",
    "primitive-setup-reference",
    "annotation-setup-reference",
    "strict-setup",
    "ordinary-empty-reference",
  ])
    assert.ok(setup.includes(`native-sfc-${file}.test.ts`));
  for (const nested of ["test-native-selected-sfc-dom", "test-native-handler-local-sfc-dom"]) {
    assert.ok(setup.includes(`./.github/actions/${nested}`));
    assert.doesNotMatch(read(`.github/actions/${nested}/action.yml`), /GITHUB_ENV/);
  }
  const attributes = read(".github/actions/test-native-attribute-values-7502/action.yml");
  for (const phase of ["history", "build", "judge"])
    assert.ok(attributes.includes(`native-attribute-values-7502-${phase}.ts`));
  assert.match(attributes, /history_status.*-ne 0.*build_status.*-ne 0.*judge_status.*-ne 0/);
  const retained = uses(steps, "./.github/actions/upload-formatter-api-corpus-evidence")[0];
  assert.equal(retained.if, "${{ always() }}");
  assert.deepEqual(retained.with, { shard: "${{ inputs.shard }}" });
  const template = yaml(".github/actions/upload-formatter-api-corpus-evidence/action.yml").runs
    .steps[0].with.name;
  const names = workerIndexes.map((index) =>
    template.replace("${{ inputs.shard }}", String(index)),
  );
  assert.equal(new Set(names).size, workerIndexes.length);
  assert.ok(names.every((name) => !name.includes("/")));
});

void test("producer and all planned workers own separate trusted sticky and fallback namespaces", () => {
  const main = "a".repeat(40),
    repository = "ubugeeei-prod/vize";
  const base = {
    event: { repository: { default_branch: "main", full_name: repository } },
    ref: "refs/heads/main",
    repository,
    sourceSha: main,
    checkoutSha: main,
    runnerEnvironment: "self-hosted",
    runnerOs: "Linux",
    runnerArch: "X64",
    role: "test-scripts",
    targetPath: "target",
  };
  const cache = uses(action().runs.steps, "./.github/actions/setup-rust-sticky-cache")[0].with;
  assert.equal(cache.key, "test-scripts");
  assert.equal(
    cache["cache-key-suffix"],
    "${{ runner.os }}-${{ runner.arch }}-shard-${{ inputs.shard }}",
  );
  assert.deepEqual(
    uses(producer().jobs.cli.steps, "./.github/actions/setup-rust-sticky-cache")[0].with,
    { key: "test-scripts", "cache-key-suffix": "${{ runner.os }}-${{ runner.arch }}" },
  );
  for (const eventName of ["push", "schedule", "workflow_dispatch"]) {
    const event =
      eventName === "push" ? { ...base.event, ref: base.ref, deleted: false } : base.event;
    const owners = ["Linux-X64", ...workerIndexes.map((index) => `Linux-X64-shard-${index}`)];
    const plans = owners.map((suffix) => rustCachePolicy({ ...base, eventName, event, suffix }));
    for (const plan of plans)
      assert.deepEqual([plan.trusted, plan.sticky, plan["target-save"]], ["true", "true", "true"]);
    for (const key of [
      "sticky-registry-key",
      "sticky-git-key",
      "sticky-target-key",
      "registry-key",
      "git-key",
      "target-key",
      "target-seed-key",
    ])
      assert.equal(new Set(plans.map((plan) => plan[key])).size, owners.length, key);
    for (const suffix of owners) {
      const untrusted = rustCachePolicy({
        ...base,
        eventName: "workflow_dispatch",
        ref: "refs/heads/ci/diagnostic",
        suffix,
      });
      assert.deepEqual(
        [untrusted.trusted, untrusted.sticky, untrusted["target-save"]],
        ["false", "false", "false"],
      );
      const mismatched = rustCachePolicy({
        ...base,
        eventName,
        event,
        suffix,
        checkoutSha: "b".repeat(40),
      });
      assert.deepEqual(
        [mismatched.trusted, mismatched.sticky, mismatched["target-save"]],
        ["false", "false", "false"],
      );
    }
  }
});
