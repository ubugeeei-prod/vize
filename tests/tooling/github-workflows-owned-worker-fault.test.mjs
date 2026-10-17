import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

const root = new URL("../..", import.meta.url);
const read = (path) => readFileSync(new URL(path, root), "utf8");

void test("worker ptrace is explicit manual-only and every original gate stays unchanged", () => {
  const workflow = parse(read(".github/workflows/ci-walltime-benchmark.yml"));
  const job = workflow.jobs["owned-worker-fault"];
  const admitted = (event, component) =>
    runInNewContext(job.if.slice(3, -3), {
      github: { event_name: event },
      inputs: { component },
    });
  assert.equal(admitted("workflow_dispatch", "owned-worker-fault"), true);
  assert.equal(admitted("workflow_dispatch", "owned-worker-controls"), true);
  for (const component of workflow.on.workflow_dispatch.inputs.component.options)
    if (!["owned-worker-fault", "owned-worker-controls"].includes(component))
      assert.equal(admitted("workflow_dispatch", component), false);
  for (const event of ["pull_request", "merge_group", "push", "schedule"])
    assert.equal(admitted(event, "owned-worker-fault"), false);
  assert.equal(workflow.on.workflow_dispatch.inputs.component.default, "all");
  assert.equal(job.uses, "./.github/workflows/owned-worker-fault.yml");
  const diagnostic = parse(read(".github/workflows/owned-worker-fault.yml"));
  assert.deepEqual(Object.keys(diagnostic.on), ["workflow_call"]);
  assert.deepEqual(diagnostic.env, workflow.env);
  assert.equal(diagnostic.on.workflow_call.inputs["controls-only"].type, "boolean");
  assert.equal(diagnostic.on.workflow_call.inputs["controls-only"].default, false);
  const execution = diagnostic.jobs["owned-worker-fault"];
  assert.equal(execution["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(execution["timeout-minutes"], 35);
  assert.deepEqual(execution.env, {
    NODE_OPTIONS: "--disable-warning=DEP0040 --disable-warning=DEP0169",
    CARGO_BUILD_JOBS: "12",
  });
  const steps = execution.steps;
  const commands = steps.map((step) => step.run ?? "").join("\n");
  assert.equal((commands.match(/-- vp run --workspace-root test:js/g) ?? []).length, 1);
  assert.doesNotMatch(
    commands,
    /retry|test-name-pattern|continue-on-error|DISABLE_TSGO|core_pattern|sysctl/,
  );
  assert.equal(steps[0].with.ref, "${{ github.sha }}");
  assert.equal(steps[2].with.ref, "${{ steps.custody.outputs.source-sha }}");
  assert.equal(steps[4].uses, "./.github/actions/setup-js-package-tests");
  assert.equal(steps[5].if, "${{ inputs.controls-only }}");
  assert.match(steps[5].run, /owned_worker_capture\.py" controls$/);
  assert.equal(steps[6].if, "${{ !inputs.controls-only }}");
  assert.equal(job.with["controls-only"], "${{ inputs.component == 'owned-worker-controls' }}");
  assert.equal(steps.at(-1).if, "${{ always() }}");
  assert.equal(steps.at(-1).with["if-no-files-found"], "error");
  assert.match(steps.at(-1).with.path, /\/public\/\*\.json/);
  assert.doesNotMatch(steps.at(-1).with.path, /private|probe|core|\.node/);
  const original = parse(read(".github/actions/test-js-packages-with-history/action.yml"));
  assert.equal(
    original.runs.steps.find((step) => step.name === "Test JS packages").run,
    "node .github/actions/test-js-packages-with-history/native-fault-evidence.ts -- vp run --workspace-root test:js",
  );
  const debuggerSource = read("tools/support/compat/github/owned_worker_gdb.py");
  for (const command of [
    "set auto-load off",
    "set debuginfod enabled off",
    "set print frame-arguments none",
    "set print entry-values no",
    "handle SIGSEGV stop print pass",
  ])
    assert.ok(debuggerSource.includes(command));
});

for (const fixture of [
  "owned_worker_laws.py",
  "owned_worker_gdb_laws.py",
  "owned_worker_supervisor_laws.py",
  "owned_worker_runner_laws.py",
]) {
  void test(`${fixture} preserves actor identity, cleanup, privacy and original failures`, () => {
    const result = spawnSync(
      "python3",
      [
        fileURLToPath(new URL(`tests/tooling/fixtures/${fixture}`, root)),
        fileURLToPath(new URL("tools/support/compat/github/", root)),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 0, result.stderr);
  });
}
