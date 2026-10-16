import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

type WorkflowStep = {
  name?: string;
  run?: string;
  env?: Record<string, string>;
  with?: Record<string, string | number>;
};
type StudyWorkflow = {
  on: Record<string, unknown> & {
    workflow_dispatch: { inputs: Record<string, { required: boolean; type: string }> };
  };
  jobs: Record<string, { steps: WorkflowStep[] }>;
};

const studies = [
  "compact-collections",
  "doctor-filter-bench",
  "text-boundary-bench",
  "type-snapshot-bench",
  "davinci-canon-scaling",
];
const workflow = (name: string) =>
  parse(
    readFileSync(new URL(`../../.github/workflows/${name}.yml`, import.meta.url), "utf8"),
  ) as StudyWorkflow;

test("repeated performance studies stay outside automatic PR checks", () => {
  for (const name of studies) {
    const source = workflow(name);
    assert.ok(Object.hasOwn(source.on, "workflow_dispatch"), name);
    assert.equal(source.on.pull_request, undefined, name);
    assert.equal(source.on.pull_request_target, undefined, name);
    assert.equal(source.on.merge_group, undefined, name);
    assert.equal(source.on.push, undefined, name);
    if (name === "text-boundary-bench") {
      assert.deepEqual(source.on.schedule, [{ cron: "23 4 * * 0" }]);
    } else {
      assert.equal(source.on.schedule, undefined, `${name} needs an explicit comparison authority`);
    }
  }
});

test("paired comparisons require an exact ancestor baseline and the dispatch head", () => {
  for (const name of ["compact-collections", "doctor-filter-bench", "type-snapshot-bench"]) {
    const source = workflow(name);
    assert.equal(source.on.workflow_dispatch.inputs.base_sha.required, true, name);
    assert.equal(source.on.workflow_dispatch.inputs.base_sha.type, "string", name);
    for (const job of Object.values(source.jobs)) {
      const selection = job.steps.find((step) => step.env?.REQUESTED_BASE);
      assert.ok(selection, name);
      assert.match(selection.env?.REQUESTED_BASE ?? "", /inputs\.base_sha/);
      assert.match(selection.run ?? "", /\[\[ "\$REQUESTED_BASE" =~ \^\[0-9a-f\]\{40\}\$ \]\]/);
      assert.match(selection.run ?? "", /merge-base --is-ancestor "\$REQUESTED_BASE"/);
      const checkout = job.steps.find((step) => step.with?.path === "head");
      assert.ok(checkout);
      assert.match(String(checkout.with?.ref), /github\.sha/);
      assert.equal(checkout.with?.["fetch-depth"], 0);
      const build = job.steps.find((step) =>
        /Build exact base and head CLIs/.test(step.name ?? ""),
      );
      if (build) {
        assert.match(build.run ?? "", /git -C head diff --exit-code HEAD/);
        assert.match(build.run ?? "", /git -C base diff --exit-code HEAD/);
      }
    }
  }
});

test("manual qualification retains its independent CLI and original400 authorities", () => {
  const snapshot = workflow("type-snapshot-bench");
  const cli = snapshot.jobs["paired-cli"].steps;
  assert.ok(cli.some((step) => step.run?.includes("canon-alias-native.mjs")));
  assert.ok(cli.some((step) => step.run?.includes("type-snapshot-cli-paired.mjs")));
  const canon = workflow("davinci-canon-scaling");
  assert.deepEqual(Object.keys(canon.on.workflow_dispatch.inputs), [
    "original_control_sha",
    "original_after_sha",
    "original_after_tree",
    "original_manifest_sha256",
    "original_public_authority",
    "original_public_authority_sha256",
  ]);
  const commands = canon.jobs.compare.steps.map((step) => step.run ?? "").join("\n");
  assert.match(commands, /warm-type-backed-workflow\.ts prepare/);
  assert.match(commands, /warm-type-backed-build\.ts before/);
  assert.match(commands, /warm-type-backed-build\.ts after/);
  assert.match(commands, /warm-type-backed-requests\.ts/);
  assert.match(commands, /warm-type-backed-public\.ts measure/);
});
