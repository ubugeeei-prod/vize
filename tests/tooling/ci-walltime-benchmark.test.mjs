import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

void test("walltime comparisons use the same source and runner without publishing authority", () => {
  const workflow = parse(
    readFileSync(
      new URL("../../.github/workflows/ci-walltime-benchmark.yml", import.meta.url),
      "utf8",
    ),
  );
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.deepEqual(workflow.on.pull_request.types, ["opened", "reopened", "labeled"]);
  assert.deepEqual(Object.keys(workflow.jobs).sort(), [
    "docs",
    "js",
    "n8n-baseline",
    "native",
    "rust",
  ]);
  for (const key of ["docs", "js", "native", "rust"]) {
    const job = workflow.jobs[key];
    assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
    assert.match(job.if, /ci-walltime-benchmark/);
    assert.match(job.if, /startsWith\(github.head_ref, 'ci\/optimize-'\)/);
    assert.equal(job.steps[0].with.ref, "${{ github.sha }}");
    const upload = job.steps.at(-1);
    assert.equal(upload.if, "${{ always() }}");
    assert.equal(upload.with["if-no-files-found"], "error");
  }
  const steps = (job) => workflow.jobs[job].steps.map((step) => step.run ?? "").join("\n");
  assert.match(steps("js"), /--host-mode serial/);
  assert.match(steps("js"), /--host-mode concurrent/);
  assert.match(steps("docs"), /--workers 1/);
  assert.match(steps("docs"), /--workers 2/);
  assert.match(steps("rust"), /cargo test --locked --workspace --no-run/);
  assert.match(steps("rust"), /--runner cargo/);
  assert.match(steps("rust"), /--runner nextest/);
  assert.doesNotMatch(steps("rust"), /--filter|--partition|DISABLE_TSGO/);
  assert.match(steps("native"), /moonbit-publish-concurrency-benchmark\.test\.ts/);
});

void test("baseline input partitions are manual-only, build once and retain both strict controller arms", () => {
  const workflow = parse(
    readFileSync(
      new URL("../../.github/workflows/ci-walltime-benchmark.yml", import.meta.url),
      "utf8",
    ),
  );
  const input = workflow.on.workflow_dispatch.inputs.component;
  assert.equal(input.default, "all");
  assert.deepEqual(input.options, ["all", "js", "docs", "rust", "native", "n8n-baseline"]);
  assert.equal(workflow.concurrency["cancel-in-progress"], false);
  const job = workflow.jobs["n8n-baseline"];
  assert.equal(
    job.if,
    "${{ github.event_name == 'workflow_dispatch' && inputs.component == 'n8n-baseline' }}",
  );
  const admitted = (candidate, event, component) =>
    runInNewContext(candidate.if.slice(3, -3), {
      github: { event_name: event },
      inputs: { component },
    });
  assert.equal(admitted(job, "workflow_dispatch", "n8n-baseline"), true);
  for (const component of ["all", "js", "docs", "rust", "native"])
    assert.equal(admitted(job, "workflow_dispatch", component), false);
  assert.equal(admitted(job, "pull_request", "n8n-baseline"), false);
  for (const key of ["docs", "js", "native", "rust"]) {
    assert.equal(admitted(workflow.jobs[key], "workflow_dispatch", "all"), true);
    assert.equal(admitted(workflow.jobs[key], "workflow_dispatch", "n8n-baseline"), false);
  }
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job["timeout-minutes"], 60);
  assert.deepEqual(job.env, { CARGO_BUILD_JOBS: "12" });
  assert.equal(job.steps.length, 6);
  assert.equal(job.steps[0].with.ref, "${{ github.sha }}");
  assert.equal(job.steps[1].uses, "./.github/actions/setup-js-package-tests");
  assert.equal(
    job.steps[2].run,
    "vp run --workspace-root build:native:test\nvp run --filter './npm/oxlint' build\n",
  );
  const [serial, partitioned] = job.steps.slice(3, 5);
  assert.deepEqual(serial.env, { VIZE_N8N_BASELINE_MODE: "serial" });
  assert.deepEqual(partitioned.env, { VIZE_N8N_BASELINE_MODE: "partitioned" });
  for (const [arm, mode] of [
    [serial, "serial"],
    [partitioned, "partitioned"],
  ]) {
    assert.equal(
      arm.run,
      `node npm/oxlint/scripts/check-project-transport.mjs --host-mode concurrent --phase-walltime target/ci-walltime/n8n-baseline-${mode}.json`,
    );
  }
  assert.equal(serial.if, undefined);
  assert.equal(partitioned.if, "${{ !cancelled() }}");
  assert.equal(job["continue-on-error"], undefined);
  for (const step of job.steps) assert.equal(step["continue-on-error"], undefined);
  const upload = job.steps.at(-1);
  assert.equal(upload.if, "${{ always() }}");
  assert.match(upload.uses, /^actions\/upload-artifact@[a-f0-9]{40}$/);
  assert.equal(upload.with.name, "ci-walltime-n8n-baseline-${{ github.sha }}");
  assert.equal(upload.with["if-no-files-found"], "error");
  assert.equal(upload.with["retention-days"], 14);
  assert.equal(
    upload.with.path,
    "target/ci-walltime/\ntarget/oxlint-original-project-transport/\nnpm/native/.artifacts/native/formatter-history/build-receipt.json\nnpm/native/.artifacts/native/formatter-history/build-process.json\nnpm/native/.artifacts/native/formatter-history/cargo.stdout.bin\nnpm/native/.artifacts/native/formatter-history/cargo.stderr.bin\n",
  );
});
