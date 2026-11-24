import { readAutomationPrograms } from "./support/automation-source.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import { stageProjectNative } from "../../npm/linter/oxlint/scripts/project-native-staging.ts";

void test("walltime comparisons use the same source and runner without publishing authority", () => {
  const workflow = parse(
    readAutomationPrograms(
      new URL("../../.github/workflows/ci-walltime-benchmark.yml", import.meta.url),
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

void test("manual components are dispatch-only and the default run keeps every same-source arm", () => {
  const workflow = parse(
    readAutomationPrograms(
      new URL("../../.github/workflows/ci-walltime-benchmark.yml", import.meta.url),
    ),
  );
  const input = workflow.on.workflow_dispatch.inputs.component;
  assert.equal(input.default, "all");
  assert.deepEqual(input.options, [
    "all",
    "js",
    "docs",
    "rust",
    "native",
    "release-builds",
    "nix",
    "artifact-payload",
  ]);
  assert.equal(workflow.concurrency["cancel-in-progress"], false);
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
  for (const key of ["docs", "js", "native", "rust"])
    assert.equal(admitted(workflow.jobs[key], "workflow_dispatch", "all"), true);
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
    readAutomationPrograms(
      new URL("../../.github/workflows/ci-walltime-benchmark.yml", import.meta.url),
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
    readAutomationPrograms(
      new URL("../../.github/actions/measure-artifact-payload/action.yml", import.meta.url),
    ),
  );
  assert.equal(action.runs.using, "composite");
  for (const step of action.runs.steps) {
    assert.equal(step["continue-on-error"], undefined);
    if (step.run) assert.equal(step.shell, "bash");
  }
  const upload = action.runs.steps.find((step) => step.with?.["compression-level"] === 0);
  const restoreAction = parse(
    readAutomationPrograms(
      new URL("../../.github/actions/restore-artifact-payload/action.yml", import.meta.url),
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
    step.run?.includes("artifact-payload.ts restore"),
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
    readAutomationPrograms(
      new URL("../../.github/actions/upload-artifact-payload-receipt/action.yml", import.meta.url),
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
