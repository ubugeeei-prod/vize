import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";
import {
  PR_SOURCE_GATES,
  requirePrChecks,
} from "../../tools/support/compat/github/require-pr-checks.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Job = { result: string; outputs?: Record<string, string> };
type Jobs = Record<string, Job>;
const rustJobs = (): Jobs => ({
  "merge-rust-source": { result: "skipped" },
  "merge-rust-differential": { result: "skipped" },
  "pr-rust-build": { result: "success" },
  "pr-rust-shard": { result: "success" },
  "rust-source-report": { result: "skipped" },
});
const sourceJobs = (rust = rustJobs()): Jobs => ({
  "pr-source-plan": { result: "success", outputs: { "canonical-corpus": "false" } },
  "pr-rust-source": {
    result: "success",
    outputs: { "jobs-json": JSON.stringify(rust), "run-rust": "true" },
  },
  "pr-js-packages": { result: "success" },
  "pr-native-runtime": { result: "success" },
  "pr-playground-test": { result: "success" },
  "wit-contracts": { result: "skipped" },
  "nuxt-module-qualification": { result: "skipped" },
  "canonical-corpus": { result: "skipped" },
  "source-report": { result: "skipped" },
});
function refuses(value: unknown): boolean {
  try {
    return requirePrChecks(value).exitCode === 1;
  } catch {
    return true;
  }
}

test("the same final required job validates raw PR results without two report runners", () => {
  const workflow = (name: string) => parse(readRepoFile(`.github/workflows/${name}.yml`));
  const check = workflow("check");
  const source = workflow("pr-source-checks");
  const rust = workflow("pr-rust-checks");
  for (const child of [source, rust]) {
    assert.equal(child.on.workflow_call.outputs["jobs-json"].value, "${{ toJSON(jobs) }}");
  }
  assert.equal(rust.on.workflow_call.outputs["run-rust"].value, "${{ inputs.run-rust }}");
  assert.deepEqual(source.jobs["source-report"].needs, PR_SOURCE_GATES);
  for (const report of [source.jobs["source-report"], rust.jobs["rust-source-report"]]) {
    assert.equal(
      report.if,
      "${{ always() && github.event_name == 'merge_group' && !contains(needs.*.result, 'cancelled') }}",
    );
  }
  const final = check.jobs["test-report"];
  assert.equal(final.name, "test-report");
  assert.equal(
    final.if,
    "${{ always() && (github.event_name == 'pull_request' || github.event_name == 'merge_group') }}",
  );
  assert.equal(final.needs.includes("pr-source-checks"), true);
  assert.deepEqual(final.steps.at(-2), {
    name: "Require every PR source and Rust result",
    if: "${{ github.event_name == 'pull_request' }}",
    env: { PR_SOURCE_JOBS_JSON: "${{ needs.pr-source-checks.outputs.jobs-json }}" },
    run: "node tools/support/compat/github/require-pr-checks.ts",
  });
  assert.equal(final.steps.at(-1).name, "Require every needed job to succeed");
  assert.deepEqual(final.steps.at(-1).env, { NEEDS_JSON: "${{ toJSON(needs) }}" });
});

test("ordinary and opted-in PRs use the existing canonical and queue-only skip policy", () => {
  assert.equal(requirePrChecks(sourceJobs()).exitCode, 0);
  const selected = sourceJobs();
  selected["pr-source-plan"].outputs = { "canonical-corpus": "true" };
  selected["canonical-corpus"].result = "success";
  assert.equal(requirePrChecks(selected).exitCode, 0);
  for (const result of ["skipped", "failure", "cancelled", "unknown", ""]) {
    selected["canonical-corpus"].result = result;
    assert.equal(refuses(selected), true, result);
  }
});

test("every named source job is required even when the reusable workflow reports success", () => {
  for (const job of PR_SOURCE_GATES) {
    const omitted = sourceJobs();
    delete omitted[job];
    assert.throws(() => requirePrChecks(omitted), new RegExp(`Missing PR source job: ${job}$`));
    for (const result of ["failure", "cancelled", "unknown", ""]) {
      assert.equal(refuses({ ...sourceJobs(), [job]: { result } }), true, `${job}: ${result}`);
    }
  }
  for (const job of [
    "pr-source-plan",
    "pr-rust-source",
    "pr-js-packages",
    "pr-native-runtime",
    "pr-playground-test",
  ]) {
    assert.equal(refuses({ ...sourceJobs(), [job]: { result: "skipped" } }), true, job);
  }
  const selections: Array<Record<string, string> | undefined> = [
    undefined,
    {},
    { "canonical-corpus": "" },
  ];
  for (const selection of selections) {
    const source = sourceJobs();
    source["pr-source-plan"].outputs = selection;
    assert.throws(() => requirePrChecks(source), /missing canonical corpus selection/);
  }
});

test("successful source and Rust rollups cannot mask missing, failed or skipped Rust leaves", () => {
  for (const job of ["pr-rust-build", "pr-rust-shard"]) {
    const omitted = rustJobs();
    delete omitted[job];
    assert.throws(
      () => requirePrChecks(sourceJobs(omitted)),
      new RegExp(`Missing Rust tier job: ${job}$`),
    );
    for (const result of ["failure", "cancelled", "skipped", "unknown", ""]) {
      const rust = { ...rustJobs(), [job]: { result } };
      assert.equal(refuses(sourceJobs(rust)), true, `${job}: ${result}`);
    }
  }
  for (const enabled of [undefined, "", "false", "TRUE"]) {
    const source = sourceJobs();
    const outputs = source["pr-rust-source"].outputs!;
    if (enabled === undefined) delete outputs["run-rust"];
    else outputs["run-rust"] = enabled;
    assert.throws(() => requirePrChecks(source), /Rust source checks must run/);
  }
});

test("absent and malformed reusable outputs fail instead of trusting their rollup", () => {
  for (const value of [undefined, null, [], "success", 1, {}]) {
    assert.equal(refuses(value), true);
  }
  const absentOutputs: Array<Record<string, string> | undefined> = [
    undefined,
    {},
    { "run-rust": "true" },
  ];
  for (const outputs of absentOutputs) {
    const source = sourceJobs();
    source["pr-rust-source"].outputs = outputs;
    assert.equal(refuses(source), true);
  }
  for (const raw of ["", " ", "{", "null", "[]", '"success"', "{}"]) {
    const source = sourceJobs();
    source["pr-rust-source"].outputs!["jobs-json"] = raw;
    assert.equal(refuses(source), true, raw);
  }
});

test("unrecognized source jobs are still gated; only the retired report is removed", () => {
  assert.equal(requirePrChecks({ ...sourceJobs(), additional: { result: "success" } }).exitCode, 0);
  for (const result of ["failure", "cancelled", "skipped", "unknown", ""]) {
    assert.equal(refuses({ ...sourceJobs(), additional: { result } }), true, result);
  }
});

test("the final report CLI accepts only complete PR output packets", () => {
  const run = (raw: string | undefined, event = "pull_request") =>
    spawnSync(process.execPath, ["tools/support/compat/github/require-pr-checks.ts"], {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, PR_SOURCE_JOBS_JSON: raw, GITHUB_EVENT_NAME: event },
    });
  const packet = JSON.stringify(sourceJobs());
  assert.equal(run(packet).status, 0);
  for (const raw of [undefined, "", "{", "null", "[]", "{}"]) {
    assert.equal(run(raw).status, 1, String(raw));
  }
  for (const event of ["merge_group", "push", "", "pull_request_target"]) {
    assert.equal(run(packet, event).status, 1, event);
  }
  const cancelled = sourceJobs({ ...rustJobs(), "pr-rust-shard": { result: "cancelled" } });
  assert.equal(run(JSON.stringify(cancelled)).status, 1);
});
