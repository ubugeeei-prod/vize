import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

const readRepoFile = (...parts: string[]): string =>
  readFileSync(new URL("../../../" + parts.join("/"), import.meta.url), "utf8");

type Step = {
  name?: string;
  if?: string;
  run?: string;
  env?: Record<string, string>;
  with?: { path?: string };
};
type Job = { if?: string; steps: Step[] };
type Workflow = {
  on: { workflow_dispatch: { inputs: Record<string, unknown> } };
  jobs: Record<string, Job>;
};
const files = ["public-native-events.yml", "release-contracts.yml"] as const;
const iconStep = "Qualify independent published icon class packets";
const iconProof = "Prove the candidate contains the required icon source";
const sdkStep = "Qualify independent published canonical SDK";
const testPath = "tests/tooling/release/public-observers.test.ts";
const guardSuffix =
  " || (github.repository == 'ubugeeei-prod/vize' && github.ref == 'refs/heads/main' && inputs.phase == 'after' && inputs.additional_observers == true)";
const evidence =
  "${{ runner.temp }}/public-custom-properties-consumer/public-canonical-sdk-evidence/";
const workflows = files.map((file) => ({
  file,
  current: parse(readRepoFile(".github", "workflows", file)) as Workflow,
  before: parse(
    readRepoFile(
      "tests",
      "tooling",
      "fixtures",
      "release-public-observers",
      file.replace(".yml", ".before.yml.txt"),
    ),
  ) as Workflow,
}));

function job(workflow: Workflow, name: string): Job {
  const result = workflow.jobs[name];
  assert.ok(result, name);
  return result;
}

function withoutSDK(source: Job): Job {
  const result = structuredClone(source);
  result.steps = result.steps.filter((step) => step.name !== sdkStep);
  const upload = result.steps.find((step) => step.with?.path?.includes(evidence));
  assert.ok(upload?.with?.path);
  upload.with.path = upload.with.path.replace(evidence + "\n", "");
  return result;
}

test("optional observers retain every original core workflow contract", () => {
  for (const { file, current, before } of workflows) {
    const retained = structuredClone(current);
    delete retained.on.workflow_dispatch.inputs.additional_observers;
    const standard = job(retained, "public-release-contracts");
    assert.ok(standard.if);
    assert.ok(standard.if.endsWith(guardSuffix));
    standard.if = standard.if.slice(0, -guardSuffix.length);
    standard.steps = standard.steps.filter(
      (step) => step.name !== iconStep && step.name !== iconProof,
    );
    if (file === "release-contracts.yml") {
      delete retained.on.workflow_dispatch.inputs.custom_properties;
      delete retained.jobs["public-custom-properties"];
      const unit = job(retained, "contracts").steps.find((step) => step.run?.includes(testPath));
      assert.ok(unit?.run);
      unit.run = unit.run.replace(testPath + " ", "");
    } else {
      retained.jobs["public-custom-properties"] = withoutSDK(
        job(retained, "public-custom-properties"),
      );
    }
    assert.deepEqual(retained, before, file);
  }
  const wrapper = workflows[1];
  const events = workflows[0];
  assert.ok(wrapper && events);
  assert.deepEqual(
    withoutSDK(job(wrapper.current, "public-custom-properties")),
    job(events.before, "public-custom-properties"),
  );
});

test("both independent observers use the existing fresh consumers after core checks", () => {
  for (const { current, file } of workflows) {
    assert.deepEqual(current.on.workflow_dispatch.inputs.additional_observers, {
      description:
        "Qualify independent published icon and canonical SDK observers after core contracts",
      required: false,
      type: "boolean",
      default: false,
    });
    const standard = job(current, "public-release-contracts");
    const icons = standard.steps.findIndex((step) => step.name === iconStep);
    const core = standard.steps.findIndex(
      (step) =>
        step.name === "Verify complete packets, runtime controls, and native public custody",
    );
    assert.ok(icons > core && core >= 0, file);
    const observer = standard.steps[icons];
    assert.ok(observer?.run);
    assert.equal(observer.env?.GH_TOKEN, undefined, "published code receives no proof token");
    const proofIndex = standard.steps.findIndex((step) => step.name === iconProof);
    assert.ok(proofIndex > core && proofIndex < icons);
    const proof = standard.steps[proofIndex];
    assert.ok(proof?.run);
    assert.equal(proof.env?.REQUIRED_ICON_SOURCE, "d38ab7e48514566f1776f25439e8777a9856b5a6");
    assert.equal(proof.env?.GH_TOKEN, "${{ github.token }}");
    assert.doesNotMatch(proof.run, /tools\/support\/release\//);
    assert.deepEqual(observer.run.trim().split(/\s+/), [
      "node",
      "tools/support/release/public_icons/verify.ts",
      '"$RUNNER_TEMP/public-release-consumer"',
      '"$PUBLIC_VERSION"',
      '"$RUNNER_TEMP/public-release-proof/icons-10928.json"',
    ]);
    const cpp = job(current, "public-custom-properties");
    const sdk = cpp.steps.findIndex((step) => step.name === sdkStep);
    const restore = cpp.steps.findIndex(
      (step) =>
        step.name === "Qualify fresh public custom properties and authored directive ownership",
    );
    assert.ok(sdk > restore && restore >= 0, file);
    assert.deepEqual(cpp.steps[sdk]?.run?.trim().split(/\s+/), [
      "node",
      "tools/support/release/public_canonical_sdk/verify.ts",
      '"$RUNNER_TEMP/public-custom-properties-consumer"',
      '"$PUBLIC_VERSION"',
      '"$RUNNER_TEMP/public-custom-properties-consumer/public-canonical-sdk.json"',
    ]);
    for (const step of [observer, cpp.steps[sdk]]) {
      assert.equal(step?.if, "inputs.additional_observers == true && inputs.phase == 'after'");
      assert.doesNotMatch(step?.run ?? "", /npm install|pnpm|cargo|NATIVE_OVERRIDE/);
    }
  }
});

test("actual guards retain defaults and fail closed for explicitly selected observers", () => {
  for (const { current, before, file } of workflows) {
    const selected = job(current, "public-release-contracts");
    for (const phase of ["", "before", "after", "release-contracts"]) {
      for (const candidate of ["", "bad", "9".repeat(40)]) {
        for (const repository of ["ubugeeei-prod/vize", "example/fork"]) {
          for (const ref of ["refs/heads/main", "refs/heads/topic"]) {
            const context = {
              github: { repository, ref, event_name: "workflow_dispatch" },
              inputs: { phase, candidate, custom_properties: false, additional_observers: false },
            };
            const original: unknown = runInNewContext(
              job(before, "public-release-contracts").if ?? "",
              context,
            );
            const retained: unknown = runInNewContext(selected.if ?? "", context);
            assert.equal(retained, original, file);
            context.inputs.additional_observers = true;
            const requested: unknown = runInNewContext(selected.if ?? "", context);
            const explicit =
              repository === "ubugeeei-prod/vize" && ref === "refs/heads/main" && phase === "after";
            assert.equal(requested, Boolean(original) || explicit, file);
          }
        }
      }
    }
    const validator = selected.steps.find(
      (step) => step.name === "Validate the explicit public candidate",
    );
    assert.deepEqual(
      validator,
      job(before, "public-release-contracts").steps.find((step) => step.name === validator?.name),
    );
  }
});

test("the actual embedded icon-source validator rejects unrelated comparisons", () => {
  for (const { current } of workflows) {
    const proof = job(current, "public-release-contracts").steps.find(
      (step) => step.name === iconProof,
    );
    assert.ok(proof?.run);
    const match = /node --input-type=module <<'NODE'\n([\s\S]+?)\nNODE/.exec(proof.run);
    assert.ok(match?.[1]);
    const script = match[1]
      .replace('import assert from "node:assert/strict";', "")
      .replace('import { readFileSync } from "node:fs";', "");
    assert.doesNotMatch(script, /^import /m);
    const required = proof.env?.REQUIRED_ICON_SOURCE;
    assert.ok(required);
    for (const [status, base, accepted] of [
      ["ahead", required, true],
      ["identical", required, true],
      ["behind", required, false],
      ["diverged", required, false],
      ["ahead", "0".repeat(40), false],
    ] as const) {
      const reads: unknown[] = [];
      const evaluate = (): unknown =>
        runInNewContext(
          script,
          {
            assert,
            process: { env: { RUNNER_TEMP: "/proof", REQUIRED_ICON_SOURCE: required } },
            readFileSync: (filename: unknown, encoding: unknown): string => {
              reads.push([filename, encoding]);
              return JSON.stringify({ status, base_commit: { sha: base } });
            },
          },
          { timeout: 500 },
        );
      if (accepted) assert.doesNotThrow(evaluate);
      else assert.throws(evaluate);
      assert.deepEqual(reads, [
        ["/proof/public-release-proof/icons-source-comparison.json", "utf8"],
      ]);
    }
  }
});
