import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import { stageProjectNative } from "../../npm/oxlint/scripts/project-native-staging.ts";

void test("walltime comparisons use the same source and runner without publishing authority", () => {
  const workflow = parse(
    readFileSync(
      new URL("../../.github/workflows/ci-walltime-benchmark.yml", import.meta.url),
      "utf8",
    ),
  );
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.deepEqual(workflow.on.pull_request.types, ["opened", "reopened", "labeled"]);
  for (const job of Object.values(workflow.jobs)) {
    assert.ok(
      Object.values(job.permissions ?? {}).every((value) => ["read", "none"].includes(value)),
    );
  }
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
  assert.match(steps("native"), /publish-npm-package-dirs-benchmark\.test\.ts/);
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
  for (const component of [
    "all",
    "js",
    "docs",
    "rust",
    "native",
    "release-builds",
    "nix",
    "n8n-baseline",
    "n8n-preparation",
    "artifact-payload",
    "owned-worker-fault",
    "owned-worker-controls",
  ])
    assert.ok(input.options.includes(component));
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
  for (const component of ["release-builds", "nix"]) {
    const candidate = workflow.jobs[component];
    assert.equal(admitted(candidate, "workflow_dispatch", component), true);
    assert.equal(admitted(candidate, "workflow_dispatch", "all"), false);
    assert.equal(admitted(candidate, "pull_request", component), false);
  }
  assert.equal(admitted(job, "workflow_dispatch", "n8n-baseline"), true);
  for (const component of ["all", "js", "docs", "rust", "native", "n8n-preparation"])
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

void test("preparation overlap is an opt-in complete same-source comparison with unchanged baseline and host modes", () => {
  const workflow = parse(
    readFileSync(
      new URL("../../.github/workflows/ci-walltime-benchmark.yml", import.meta.url),
      "utf8",
    ),
  );
  const job = workflow.jobs["n8n-preparation"];
  const admitted = (event, component) =>
    runInNewContext(job.if.slice(3, -3), {
      github: { event_name: event },
      inputs: { component },
    });
  assert.equal(admitted("workflow_dispatch", "n8n-preparation"), true);
  for (const component of ["all", "js", "docs", "rust", "native", "n8n-baseline"])
    assert.equal(admitted("workflow_dispatch", component), false);
  assert.equal(admitted("pull_request", "n8n-preparation"), false);
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job["timeout-minutes"], 60);
  assert.deepEqual(job.env, { CARGO_BUILD_JOBS: "12", VIZE_N8N_BASELINE_MODE: "partitioned" });
  assert.equal(job.steps.length, 6);
  assert.equal(job.steps[0].with.ref, "${{ github.sha }}");
  assert.equal(job.steps[1].uses, "./.github/actions/setup-js-package-tests");
  assert.equal(job.steps[2].run, workflow.jobs["n8n-baseline"].steps[2].run);
  const [sequential, overlap] = job.steps.slice(3, 5);
  for (const [arm, mode] of [
    [sequential, "sequential"],
    [overlap, "overlap"],
  ]) {
    assert.equal(arm.env, undefined);
    assert.equal(
      arm.run,
      `node npm/oxlint/scripts/check-project-transport.mjs --host-mode concurrent --preparation-mode ${mode} --phase-walltime target/ci-walltime/n8n-preparation-${mode}.json`,
    );
  }
  assert.equal(sequential.if, undefined);
  assert.equal(overlap.if, "${{ !cancelled() }}");
  assert.equal(job["continue-on-error"], undefined);
  for (const step of job.steps) assert.equal(step["continue-on-error"], undefined);
  const upload = job.steps.at(-1);
  assert.equal(upload.if, "${{ always() }}");
  assert.equal(upload.uses, workflow.jobs["n8n-baseline"].steps.at(-1).uses);
  assert.equal(upload.with.name, "ci-walltime-n8n-preparation-${{ github.sha }}");
  assert.equal(upload.with.path, workflow.jobs["n8n-baseline"].steps.at(-1).with.path);
  assert.equal(upload.with["if-no-files-found"], "error");
  assert.equal(upload.with["retention-days"], 14);
});

void test("both benchmark arms stage the same source bytes into distinct retained campaign directories", (t) => {
  // Exercise the actual staging operation; controlled bytes prove artifact
  // separation only and do not qualify an addon or a performance result.
  const root = mkdtempSync(path.join(os.tmpdir(), "vize-preparation-arms-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const binary = path.join(root, "controlled-source.bin"),
    receipt = path.join(root, "build-receipt.json");
  const bytes = Buffer.from("same controlled source bytes\\r\\n");
  const sha256 = createHash("sha256").update(bytes).digest("hex");
  writeFileSync(binary, bytes);
  const source = { head: "a".repeat(40), tree: "b".repeat(40), workingDiff: "controlled" };
  writeFileSync(receipt, JSON.stringify({ source, frozen: { sha256 } }));
  const arms = ["sequential", "overlap"].map((mode) => {
    const staged = stageProjectNative(path.join(root, "campaigns"), binary, receipt, sha256);
    writeFileSync(path.join(staged.artifacts, "phase-walltime.json"), JSON.stringify({ mode }));
    return { mode, ...staged };
  });
  assert.notEqual(arms[0].artifacts, arms[1].artifacts);
  for (const { mode, artifacts, binary: staged } of arms) {
    assert.ok(readFileSync(staged).equals(bytes));
    assert.ok(
      readFileSync(path.join(artifacts, "build-receipt.json")).equals(readFileSync(receipt)),
    );
    assert.deepEqual(JSON.parse(readFileSync(path.join(artifacts, "phase-walltime.json"))), {
      mode,
    });
  }
});

void test("whole artifact payload trial is manual-only with a separate complete consumer", () => {
  const workflow = parse(
    readFileSync(
      new URL("../../.github/workflows/ci-walltime-benchmark.yml", import.meta.url),
      "utf8",
    ),
  );
  const producer = workflow.jobs["artifact-payload"];
  const consumer = workflow.jobs["artifact-payload-restore"];
  for (const job of [producer, consumer]) {
    assert.equal(
      job.if,
      "${{ github.event_name == 'workflow_dispatch' && inputs.component == 'artifact-payload' }}",
    );
    assert.equal(job.steps[0].with.ref, "${{ github.sha }}");
    assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
    assert.equal(job["continue-on-error"], undefined);
    for (const step of job.steps) assert.equal(step["continue-on-error"], undefined);
  }
  assert.deepEqual(producer.permissions, { contents: "read", actions: "read" });
  assert.equal(consumer.needs, "artifact-payload");
  assert.equal(producer.steps.at(-1).uses, "./.github/actions/measure-artifact-payload");
  const action = parse(
    readFileSync(
      new URL("../../.github/actions/measure-artifact-payload/action.yml", import.meta.url),
      "utf8",
    ),
  );
  assert.equal(action.runs.using, "composite");
  for (const step of action.runs.steps) {
    assert.equal(step["continue-on-error"], undefined);
    if (step.run) assert.equal(step.shell, "bash");
  }
  const upload = action.runs.steps.find((step) => step.with?.["compression-level"] === 0);
  const restoreAction = parse(
    readFileSync(
      new URL("../../.github/actions/restore-artifact-payload/action.yml", import.meta.url),
      "utf8",
    ),
  );
  assert.equal(consumer.steps[1].uses, "./.github/actions/restore-artifact-payload");
  assert.deepEqual(consumer.steps[1].with, {
    artifact_id: "${{ inputs.artifact_id }}",
    artifact_source_sha: "${{ inputs.artifact_source_sha }}",
  });
  assert.equal(restoreAction.runs.using, "composite");
  for (const step of restoreAction.runs.steps) {
    assert.equal(step["continue-on-error"], undefined);
    if (step.run) assert.equal(step.shell, "bash");
  }
  const download = restoreAction.runs.steps.find((step) =>
    step.uses?.startsWith("actions/download-artifact@"),
  );
  assert.equal(upload.with.name, download.with.name);
  assert.equal(upload.with["if-no-files-found"], "error");
  assert.equal(download.with.path, "${{ runner.temp }}/payload-envelope");
  const restore = restoreAction.runs.steps.find((step) =>
    step.run?.includes("artifact_payload.py restore"),
  );
  assert.match(
    restore.run,
    /--source-sha "\$PAYLOAD_SOURCE_SHA" --driver-sha "\$GITHUB_SHA" --artifact-id "\$PAYLOAD_ARTIFACT_ID"/,
  );
  assert.deepEqual(restore.env, {
    PAYLOAD_ARTIFACT_ID: "${{ inputs.artifact_id }}",
    PAYLOAD_SOURCE_SHA: "${{ inputs.artifact_source_sha }}",
  });
  const receiptAction = parse(
    readFileSync(
      new URL("../../.github/actions/upload-artifact-payload-receipt/action.yml", import.meta.url),
      "utf8",
    ),
  );
  assert.equal(consumer.steps.at(-1).uses, "./.github/actions/upload-artifact-payload-receipt");
  assert.equal(consumer.steps.at(-1).if, "${{ always() }}");
  assert.equal(receiptAction.runs.using, "composite");
  assert.equal(receiptAction.runs.steps.length, 1);
  const receipt = receiptAction.runs.steps[0];
  assert.equal(receipt.if, "${{ always() }}");
  assert.equal(
    receipt.with.name,
    "artifact-payload-consumer-${{ github.sha }}-${{ github.run_attempt }}",
  );
  assert.equal(
    receipt.with.path,
    "${{ runner.temp }}/payload-restore.json\n${{ runner.temp }}/payload-restore-resources.txt\n",
  );
  assert.equal(receipt.with["if-no-files-found"], "error");
  assert.equal(receipt.with["retention-days"], 14);
  assert.equal(workflow.on.workflow_dispatch.inputs.component.default, "all");
});
