import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";
import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.ts";
import { sourceInventoryLanes } from "../../tools/support/compat/davinci/source-inventories.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";
const CORE = ["fmt-rust", "check-js", "security-audit", "node-engine-compat", "check-vize-apps"];
const PR_JOBS = [
  "iteration-plan",
  ...CORE,
  "pr-source-checks",
  "instruction-counts",
  "level-dependency-direction",
  "release-integration-catalog",
];
const FULL_SUITE_JOBS = [
  "nix-flake",
  "vue-parity",
  "test-scripts",
  "editor-extensions",
  "editor-host-smoke",
  "build-js-packages",
  "test-js-packages",
  "full-js-package-tests",
  "full-js-original-project",
  "clippy-and-test",
  "coverage",
  "source-coverage",
  "branch-coverage",
  "playground-test",
];
type Job = {
  if?: string;
  needs?: string[] | string;
  outputs?: Record<string, string>;
  steps?: Array<{
    name?: string;
    "continue-on-error"?: boolean;
    if?: string;
    run?: string;
    env?: Record<string, string>;
    uses?: string;
    with?: Record<string, string>;
  }>;
  "timeout-minutes"?: number | string;
  uses?: string;
};
const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as {
  on?: Record<string, unknown>;
  jobs?: Record<string, Job>;
  concurrency?: { group?: string; "cancel-in-progress"?: boolean };
};
function needs(results: Record<string, string> = {}): Record<string, { result: string }> {
  return Object.fromEntries(PR_JOBS.map((job) => [job, { result: results[job] ?? "success" }]));
}
test("obsolete main validation is cancelled when the branch advances", () => {
  for (const candidate of [workflow]) {
    assert.equal(candidate.concurrency?.["cancel-in-progress"], true);
    assert.match(
      candidate.concurrency?.group ?? "",
      /github\.event\.pull_request\.number \|\| github\.ref/,
    );
    if (candidate === workflow) {
      assert.match(candidate.concurrency?.group ?? "", /format\('full-\{0\}', github\.sha\)/);
    } else {
      assert.doesNotMatch(candidate.concurrency?.group ?? "", /github\.sha/);
    }
  }
});
test("source gates cover iteration events while full qualification runs on schedule or dispatch", () => {
  assert.ok(workflow.on?.pull_request);
  assert.deepEqual(workflow.on?.merge_group, {
    types: ["checks_requested"],
    branches: ["main"],
  });
  assert.ok(workflow.on?.push);
  assert.ok(workflow.on?.schedule);
  assert.ok(Object.hasOwn(workflow.on ?? {}, "workflow_dispatch"));
  assert.deepEqual(workflow.jobs?.["test-report"]?.needs, PR_JOBS);
  assert.equal(
    workflow.jobs?.["test-report"]?.if,
    "${{ always() && (github.event_name == 'pull_request' || github.event_name == 'merge_group') }}",
  );
  for (const job of CORE) {
    assert.equal(workflow.jobs?.[job]?.if, undefined, `${job} must run on pull requests`);
  }
  for (const job of FULL_SUITE_JOBS) {
    assert.equal(
      workflow.jobs?.[job]?.if,
      job === "source-coverage"
        ? "${{ github.event_name == 'schedule' || (github.event_name == 'workflow_dispatch' && !(inputs.release-qualification && (startsWith(github.ref, 'refs/heads/release/v') || startsWith(github.ref, 'refs/tags/v')))) }}"
        : job === "clippy-and-test" || job === "test-js-packages"
          ? "${{ always() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}"
          : job === "test-scripts"
            ? "${{ !cancelled() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}"
            : "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
    );
  }
  assert.equal(
    workflow.jobs?.["nix-flake"]?.if,
    "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  assert.equal(
    workflow.jobs?.["semver-checks"]?.if,
    "${{ github.event_name == 'workflow_dispatch' && (startsWith(github.ref, 'refs/heads/release/v') || startsWith(github.ref, 'refs/tags/v')) }}",
  );
  const reportSteps = workflow.jobs?.["test-report"]?.steps ?? [];
  const aggregate = reportSteps.find((step) => step.name === "Require every needed job to succeed");
  const select = reportSteps.find(
    (step) => step.name === "Re-select the current integration before the required final gate",
  );
  const final = reportSteps.find(
    (step) =>
      step.name === "Freshly verify the same exact qualification at the required final gate",
  );
  const custody = reportSteps.find(
    (step) => step.name === "Retain the required final qualification observations",
  );
  assert.ok(aggregate && select && final && custody);
  assert.equal(
    aggregate.run,
    "node tools/support/compat/github/require-needs-success.ts --check-report",
  );
  assert.ok(reportSteps.indexOf(aggregate) < reportSteps.indexOf(select));
  assert.ok(reportSteps.indexOf(select) < reportSteps.indexOf(final));
  assert.equal(select.if, "${{ github.event_name == 'merge_group' }}");
  assert.equal(select.run, "node tools/support/compat/github/release-integration-event.ts");
  assert.equal(final.if, "${{ github.event_name == 'merge_group' }}");
  assert.ok(
    final.run?.includes(
      'integration-release.ts --verify < "$directory/qualification.json" > "$directory/final-qualification.json"',
    ),
  );
  for (const step of [aggregate, select, final]) assert.equal(step["continue-on-error"], undefined);
  assert.equal(reportSteps.at(-2), final);
  assert.equal(reportSteps.at(-1), custody);
  assert.equal(custody.if, "${{ always() && github.event_name == 'merge_group' }}");
  assert.equal(custody.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
  assert.equal(custody.with?.path, "${{ runner.temp }}/vize-release-integration-final/");
  assert.equal(custody.with?.["if-no-files-found"], "error");
  const commands = (job: string) =>
    (workflow.jobs?.[job]?.steps ?? []).map((step) => step.run ?? "").join("\n");
  const checkSteps = workflow.jobs?.["check-js"]?.steps ?? [];
  assert.equal(
    checkSteps.find((step) => step.name === "Check fast JS/TS")?.if,
    "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' || github.event_name == 'push' }}",
  );
  assert.equal(
    checkSteps.find((step) => step.name === "Check JS/TS")?.if,
    "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  const inventory = checkSteps.find((step) => step.name === "Check Davinci source inventories");
  assert.ok(inventory);
  assert.equal(
    inventory.if,
    "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  assert.equal(inventory.uses, "./.github/actions/check-level-inventories");
  const inventoryAction = parse(
    readRepoFile(".github", "actions", "check-level-inventories", "action.yml"),
  ) as {
    runs: { using: string; steps: NonNullable<Job["steps"]> };
  };
  assert.equal(inventoryAction.runs.using, "composite");
  const validate = inventoryAction.runs.steps.find(
    (step) => step.name === "Check Davinci source inventories",
  );
  assert.equal(validate?.run, "node tools/support/compat/davinci/source-inventories.ts");
  // The same five checks, in the same order within each dependent lane.
  assert.deepEqual(
    sourceInventoryLanes.flat().map((args) => `node ${args.join(" ")}`),
    [
      "node tools/support/compat/davinci/croquis-consumers.mjs --check",
      "node tools/support/compat/davinci/consumer-migration-surfaces.mjs --check",
      "node --test tests/tooling/davinci-storage-policy.test.ts",
      "node tools/support/compat/davinci/generated-ledgers.mjs --write",
      "node tools/support/compat/davinci/generated-ledgers.mjs --check",
    ],
  );
  assert.ok(
    checkSteps.indexOf(inventory) <
      checkSteps.findIndex((step) => step.uses === "./.github/actions/setup-js-check-runtime"),
    "inventory checks must fail before the heavy JS check setup",
  );
  const ledgerUpload = inventoryAction.runs.steps.find(
    (step) => step.name === "Upload generated Davinci ledgers",
  );
  assert.equal(ledgerUpload?.with?.name, "davinci-generated-ledgers");
  assert.equal(ledgerUpload?.with?.path, "artifacts/davinci-ledgers/");
  assert.equal(ledgerUpload?.with?.["if-no-files-found"], "error");
  assert.equal(ledgerUpload?.with?.["retention-days"], 14);
  assert.equal(ledgerUpload?.if, "${{ always() }}");
  assert.match(commands("check-js"), /vp run --workspace-root check:iteration/);
  assert.match(commands("check-js"), /vp run --workspace-root check:ci/);
  const appAction = parse(readRepoFile(".github", "actions", "check-vize-apps", "action.yml")) as {
    runs: { using: string; steps: NonNullable<Job["steps"]> };
  };
  assert.equal(appAction.runs.using, "composite");
  const appSteps = appAction.runs.steps;
  assert.ok(appSteps.some((step) => /cargo build --profile ci -p vize/.test(step.run ?? "")));
  const configExports = appSteps.findIndex((step) => step.run?.includes("(cd npm/cli && vp pack)"));
  assert.ok(
    configExports >= 0 &&
      configExports < appSteps.findIndex((step) => step.name === "Check Vize app fixtures"),
    "workspace config imports need built public JS exports before the source CLI checks apps",
  );
  assert.match(
    readRepoFile(".github", "actions", "test-full-tooling", "action.yml"),
    /vp run --workspace-root test:scripts/,
  );
  const history = parse(
    readRepoFile(".github", "actions", "test-js-packages-with-history", "action.yml"),
  ) as { runs: { steps: Array<{ run?: string }> } };
  const jsTestCommands = history.runs.steps.map((step) => step.run ?? "").join("\n");
  assert.match(jsTestCommands, /vp run --workspace-root test:js/);
  assert.match(jsTestCommands, /node npm\/oxlint\/scripts\/check-project-transport\.mjs/);
  assert.match(commands("rust-clippy-portability"), /cargo clippy --workspace/);
  assert.equal(
    workflow.jobs?.["rust-native-differential"]?.steps?.find((step) => step.name === "Test")?.uses,
    "./.github/actions/test-rust-workspace-differential",
  );
  assert.match(commands("rust-davinci-allocations"), /cargo bench -p vize_l1_to_l2/);
  assert.match(commands("branch-coverage"), /coverage:source:branch/);
  assert.match(commands("playground-test"), /test:browser/);
  assert.deepEqual(workflow.jobs?.["playground-test"]?.needs, ["build-js-packages"]);
});
test("input selection gates work inside required jobs and keeps the planner dependency", () => {
  const planner = workflow.jobs?.["iteration-plan"];
  assert.ok(planner);
  assert.equal(planner.steps?.at(-1)?.run, "node tools/support/compat/github/plan-check-inputs.ts");
  assert.ok(planner.steps?.some((step) => step.uses?.startsWith("actions/setup-node@")));
  for (const [jobName, output, command] of [
    ["security-audit", "dependencies", "node tools/commands/ci/npm-audit-retry.ts"],
    ["node-engine-compat", "engines", "node --test tests/tooling/node-engine-matrix.test.ts"],
    ["check-vize-apps", "apps", "./.github/actions/check-vize-apps"],
  ]) {
    assert.equal(planner.outputs?.[output], `\${{ steps.plan.outputs.${output} }}`);
    const job = workflow.jobs?.[jobName];
    assert.ok(job);
    assert.equal(job.if, undefined, `${jobName} must always report its selected-input result`);
    assert.equal(job.needs, "iteration-plan", `${jobName} requires a successful planner`);
    const check = job.steps?.find((step) => (step.run ?? step.uses ?? "").startsWith(command));
    assert.ok(check, `${jobName} must keep its product check`);
    assert.equal(check.if, `\${{ needs.iteration-plan.outputs.${output} == 'true' }}`);
    assert.ok(
      job.steps?.some(
        (step) => step.if === `\${{ needs.iteration-plan.outputs.${output} != 'true' }}`,
      ),
      `${jobName} must report unchanged inputs`,
    );
  }
});
test("report fails closed when any required check or its planner fails or skips", () => {
  assert.equal(aggregateNeedsResults(needs()).exitCode, 0);
  for (const job of PR_JOBS) {
    for (const result of ["failure", "cancelled", "skipped", "unknown"]) {
      const decision = aggregateNeedsResults(needs({ [job]: result }));
      assert.equal(decision.exitCode, 1);
      assert.ok(decision.message.includes(`${job}: ${result}`));
    }
    assert.throws(() => aggregateNeedsResults({ ...needs(), [job]: {} }), /reported no result/);
  }
  assert.throws(() => aggregateNeedsResults({}), /needs context is empty/);
});
test("report CLI propagates required check and planner failures", () => {
  const script = "tools/support/compat/github/require-needs-success.ts";
  const run = (results: Record<string, string>) =>
    spawnSync(process.execPath, [script], {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, NEEDS_JSON: JSON.stringify(needs(results)) },
    });
  assert.equal(run({}).status, 0);
  for (const job of ["iteration-plan", "check-vize-apps"]) {
    const command = run({ [job]: "failure" });
    assert.equal(command.status, 1);
    assert.ok(command.stderr.includes(`${job}: failure`));
  }
});
test("full JS qualification retains independent native builds and strictly gates all full events", () => {
  const full = workflow.jobs?.["full-js-report"];
  assert.ok(full);
  assert.equal(
    full.if,
    "${{ always() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
  );
  const fullJobs = [
    "test-js-packages",
    "build-js-packages",
    "test-scripts",
    "full-source-cli",
    "check-js",
    "instruction-counts",
  ];
  assert.deepEqual(full.needs, fullJobs);
  const caller = full.steps?.at(-1);
  assert.equal(caller?.uses, "./.github/actions/report-full-js-qualification");
  assert.deepEqual(caller?.with, { "needs-json": "${{ toJSON(needs) }}" });
  const report = parse(
    readRepoFile(".github", "actions", "report-full-js-qualification", "action.yml"),
  ) as {
    inputs: Record<string, { required?: boolean }>;
    runs: { using: string; steps: NonNullable<Job["steps"]> };
  };
  assert.equal(report.inputs["needs-json"].required, true);
  assert.equal(report.runs.using, "composite");
  const gate = report.runs.steps.at(-1);
  assert.ok(gate?.run);
  assert.deepEqual(gate.env, { NEEDS_JSON: "${{ inputs.needs-json }}" });
  assert.equal(gate?.run, "node tools/support/compat/github/require-needs-success.ts");
  const setup = report.runs.steps.find((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  assert.ok(setup);
  assert.deepEqual(setup.with, {
    "node-version-file": "package.json",
    cache: false,
    "run-install": false,
  });
  assert.ok(report.runs.steps.indexOf(setup) < report.runs.steps.indexOf(gate));
  const testJob = workflow.jobs?.["test-js-packages"];
  const producers = ["full-js-package-tests", "full-js-original-project"];
  assert.deepEqual(testJob?.needs, producers);
  const packageGate = testJob?.steps?.at(-1);
  assert.equal(
    packageGate?.run,
    "node tools/support/compat/github/require-needs-success.ts --full-js-packages",
  );
  assert.deepEqual(packageGate?.env, { NEEDS_JSON: "${{ toJSON(needs) }}" });
  for (const [id, partition] of [
    ["full-js-package-tests", "${{ matrix.group }}"],
    ["full-js-original-project", "original-project"],
  ]) {
    const history = workflow.jobs?.[id]?.steps?.find(
      (step) => step.uses === "./.github/actions/test-js-packages-with-history",
    );
    assert.equal(history?.if, "${{ !cancelled() }}");
    assert.deepEqual(history?.with, { partition });
    assert.equal(history?.run, undefined);
  }
  for (const job of [
    ...producers.map((id) => workflow.jobs?.[id]),
    workflow.jobs?.["build-js-packages"],
  ]) {
    assert.ok(job);
    assert.equal(job.needs, undefined, "each exact-source job owns its physical native build");
    assert.ok(job.steps?.some((step) => step.uses?.startsWith("actions/checkout@")));
    assert.equal(
      job.steps?.some((step) => step.uses?.startsWith("actions/download-artifact@")),
      false,
      "a shared CI addon cannot substitute for the debug history producer",
    );
  }
  const allSucceeded = Object.fromEntries(fullJobs.map((job) => [job, { result: "success" }]));
  assert.equal(aggregateNeedsResults(allSucceeded).exitCode, 0);
  for (const job of fullJobs) {
    for (const result of ["failure", "cancelled", "skipped", "unknown"]) {
      const observed = { ...allSucceeded, [job]: { result } };
      assert.equal(aggregateNeedsResults(observed).exitCode, 1, `${job}: ${result}`);
      const command: SpawnSyncReturns<string> = spawnSync(
        process.execPath,
        [gate.run.split(" ")[1]],
        {
          cwd: root,
          encoding: "utf8",
          env: { ...process.env, NEEDS_JSON: JSON.stringify(observed) },
        },
      );
      assert.equal(command.status, 1, `${command.stderr}\n${command.stdout}`);
      assert.ok(command.stderr.includes(`${job}: ${result}`));
    }
  }
});
test("slow suites use schedules or explicit dispatch without starting on PRs", () => {
  const pushOrDispatch = ["davinci-html-content-model.yml", "davinci-incremental.yml"];
  const scheduledOrDispatch = [
    "benchmark.yml",
    "check-bench.yml",
    "content-mapper-conformance.yml",
    "davinci-lean.yml",
    "davinci-resource-budgets.yml",
    "e2e.yml",
    "editor-conformance.yml",
    "fresco.yml",
    "fuzz.yml",
    "miri.yml",
    "tool-benchmark.yml",
    "vue-benchmarks-replay.yml",
  ];
  for (const name of [
    ...pushOrDispatch,
    ...scheduledOrDispatch,
    "criterion-bench.yml",
    "pkg-pr-new.yml",
  ]) {
    const events =
      (parse(readRepoFile(".github", "workflows", name)) as { on?: Record<string, unknown> }).on ??
      {};
    assert.equal(Object.hasOwn(events, "pull_request"), false, name);
    assert.equal(Object.hasOwn(events, "workflow_dispatch"), true, name);
    if (pushOrDispatch.includes(name)) assert.equal(Object.hasOwn(events, "push"), true, name);
    if (scheduledOrDispatch.includes(name))
      assert.equal(Object.hasOwn(events, "schedule"), true, name);
    if (scheduledOrDispatch.includes(name) || name === "pkg-pr-new.yml")
      assert.equal(Object.hasOwn(events, "push"), false, name);
  }
});

test("release candidate readiness joins all seven original gates before promotion without a cycle", () => {
  const release = parse(readRepoFile(".github/workflows/release.yml")) as {
    jobs: Record<string, Job>;
  };
  const expected = {
    name: "Release candidate ready",
    needs: [
      "candidate-preflight",
      "build-cli",
      "build-editor-extensions",
      "build-release-packages",
      "build-wasm-package",
      "build-native-all",
      "smoke-release-packages",
    ],
    "runs-on": "ubuntu-24.04",
    "timeout-minutes": 5,
    steps: [{ run: 'echo "All release artifacts and preflight gates passed for this candidate."' }],
  };
  const assertReady = (jobs: Record<string, Job>) => {
    assert.deepEqual(jobs["candidate-ready"], expected);
    const seen = new Set<string>();
    const visit = (name: string, pending = new Set<string>()) => {
      assert.ok(!pending.has(name), "release readiness dependency cycle");
      assert.doesNotMatch(
        name,
        /^(release-preflight|release-npm-|release-crates|release-jsr|create-github-release)/,
      );
      const job = jobs[name];
      assert.ok(job, name);
      assert.notEqual(job.uses, "./.github/workflows/release-promotion.yml");
      const dependencies = typeof job.needs === "string" ? [job.needs] : (job.needs ?? []);
      for (const dependency of dependencies) visit(dependency, new Set([...pending, name]));
      seen.add(name);
    };
    visit("candidate-ready");
    assert.deepEqual(
      [...seen].sort(),
      [...expected.needs, "candidate-ready", "candidate", "plan-release-platforms"].sort(),
    );
    assert.equal(jobs["candidate-preflight"].uses, "./.github/workflows/release-preflight.yml");
    assert.equal(jobs["release-preflight"].needs, "candidate-ready");
  };
  assertReady(release.jobs);
  for (const name of expected.needs) {
    const changed = structuredClone(release.jobs);
    changed["candidate-ready"].needs = expected.needs.filter((value) => value !== name);
    assert.throws(() => assertReady(changed));
  }
  for (const dependency of [
    "release-preflight",
    "candidate-ready",
    "release-npm-cli",
    "create-github-release",
  ]) {
    const changed = structuredClone(release.jobs);
    changed.candidate.needs = dependency;
    assert.throws(() => assertReady(changed));
  }
  const preflight = parse(readRepoFile(".github/workflows/release-preflight.yml"));
  assert.equal(preflight.jobs.verify.needs, "wait-check");
  assert.ok(
    preflight.jobs["wait-check"].steps.some((step: { run?: string }) =>
      step.run?.includes("main.ts --wait-existing"),
    ),
  );
  assert.ok(
    preflight.jobs.verify.steps.some(
      (step: { run?: string }) =>
        step.run === "node tools/support/compat/github/release-preflight.ts --verify-only",
    ),
  );
  const catalog = parse(readRepoFile(".github/workflows/release-integration-catalog.yml"));
  const wait = catalog.jobs["release-integration-catalog"].steps.find(
    (step: { id?: string }) => step.id === "qualification",
  );
  const q =
    'integration.ts --wait < "$directory/candidate-context.json" > "$directory/qualification-q.json"';
  const r =
    'integration-release.ts --wait < "$directory/qualification-q.json" > "$directory/qualification.json"';
  assert.ok(wait.run.includes(q) && wait.run.includes(r));
  assert.ok(wait.run.indexOf(q) < wait.run.indexOf(r));
  assert.doesNotMatch(
    workflow.jobs?.["test-report"]?.steps?.at(-2)?.run ?? "",
    /rust-script|cargo |--wait/,
  );
});
