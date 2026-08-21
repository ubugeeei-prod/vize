import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile, workflowJobRunsOn } from "./support/github-workflows.ts";

const BLACKSMITH_RUNNER = "blacksmith-32vcpu-ubuntu-2404";

type Step = {
  continueOnError?: boolean;
  env?: Record<string, string>;
  id?: string;
  if?: string;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, string | boolean>;
  "continue-on-error"?: boolean;
};
type Job = {
  if?: string;
  name?: string;
  needs?: string | string[];
  outputs?: Record<string, string>;
  permissions?: Record<string, string>;
  "runs-on"?: string;
  steps?: Step[];
  strategy?: { "fail-fast"?: boolean; matrix?: string };
  "timeout-minutes"?: number;
};
type Workflow = {
  concurrency?: Record<string, unknown>;
  env?: Record<string, string>;
  jobs?: Record<string, Job>;
  on?: Record<string, unknown>;
  permissions?: Record<string, string>;
  "run-name"?: string;
};

const loadWorkflow = (): Workflow =>
  parse(readRepoFile(".github", "workflows", "e2e.yml")) as Workflow;
const namedStep = (job: Job | undefined, name: string): Step => {
  const found = job?.steps?.find((step) => step.name === name);
  assert.ok(found, `missing step ${name}`);
  return found;
};

test("App E2E keeps PR, nightly, dispatch, and immutable run identity", () => {
  const workflow = loadWorkflow();
  const triggers = workflow.on ?? {};
  assert.ok(triggers.pull_request);
  assert.ok(triggers.schedule);
  assert.ok(triggers.workflow_dispatch);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.match(workflow["run-name"] ?? "", /readiness-pr-/);
  assert.match(workflow["run-name"] ?? "", /inputs\.target_sha \|\| github\.sha/);
  assert.equal(workflow.env?.E2E_TARGET_SHA, "${{ inputs.target_sha || github.sha }}");
  assert.equal(workflow.concurrency?.["cancel-in-progress"], true);
  assert.match(String(workflow.concurrency?.group), /inputs\.target_sha \|\| github\.sha/);
});

test("PR readiness plans six isolated rows behind one stable aggregator", () => {
  const jobs = loadWorkflow().jobs ?? {};
  const plan = jobs["app-readiness-plan"];
  const producer = jobs["app-readiness-producer"];
  const aggregate = jobs["app-readiness"];
  assert.ok(plan && producer && aggregate);
  assert.equal(plan.name, "Plan app readiness");
  assert.equal(plan["runs-on"], BLACKSMITH_RUNNER);
  assert.deepEqual(plan.permissions, { contents: "read", "pull-requests": "read" });
  assert.deepEqual(plan.outputs, {
    run: "${{ steps.changes.outputs.readiness }}",
    matrix: "${{ steps.plan.outputs.matrix }}",
    count: "${{ steps.plan.outputs.count }}",
  });
  const changes = namedStep(plan, "Detect app readiness changes");
  assert.match(changes.uses ?? "", /^dorny\/paths-filter@[0-9a-f]{40}$/);
  const filters = String(changes.with?.filters);
  for (const path of [
    ".github/actions/app-e2e-row/**",
    ".github/workflows/e2e.yml",
    "tests/package.json",
    "tests/_fixtures/_git/{elk,misskey,npmx.dev,nuxt-ui,reka-ui,vuefes-2025}",
    "tests/app/dev/{misskey,nuxt-ui}.spec.ts",
    "tests/app/dev/{misskey-hmr,nuxt-ui-dev-server,nuxt-ui-hmr,source-restore}.ts",
    "tools/github/app-e2e-*.mjs",
  ]) {
    assert.match(filters, new RegExp(path.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
  }
  for (const broad of ["npm/**", "tests/**", "tools/**"]) {
    assert.ok(!filters.includes(`- '${broad}'`), `unexpected broad readiness path ${broad}`);
  }
  const checkout = namedStep(plan, "Checkout readiness source");
  assert.equal(checkout.with?.ref, "${{ github.sha }}");
  assert.equal(checkout.with?.["persist-credentials"], false);
  assert.match(namedStep(plan, "Verify readiness merge SHA").run ?? "", /git rev-parse HEAD/);
  const planStep = namedStep(plan, "Plan readiness rows");
  assert.match(planStep.run ?? "", /--profile readiness/);
  const readinessEvidence = namedStep(plan, "Upload readiness plan evidence");
  assert.match(String(readinessEvidence.with?.name), /github\.run_id.*github\.run_attempt/);

  assert.equal(producer.name, "app-readiness (${{ matrix.shard }})");
  assert.equal(producer.needs, "app-readiness-plan");
  assert.equal(producer.if, "needs.app-readiness-plan.outputs.run == 'true'");
  assert.equal(producer["runs-on"], "${{ matrix.runner }}");
  assert.equal(producer.strategy?.["fail-fast"], false);
  assert.equal(
    producer.strategy?.matrix,
    "${{ fromJSON(needs.app-readiness-plan.outputs.matrix) }}",
  );
  const rowAction = producer.steps?.find((step) => step.uses === "./.github/actions/app-e2e-row");
  assert.ok(rowAction);
  for (const input of [
    "profile",
    "suite",
    "shard",
    "task",
    "timeout",
    "needs-playwright",
    "cache-key",
    "worktree-id",
    "artifact-stem",
  ]) {
    assert.ok(rowAction.with?.[input], `missing row input ${input}`);
  }

  assert.equal(aggregate.name, "app-readiness");
  assert.equal(aggregate["runs-on"], BLACKSMITH_RUNNER);
  assert.deepEqual(aggregate.needs, ["app-readiness-plan", "app-readiness-producer"]);
  assert.match(aggregate.if ?? "", /always\(\)/);
  assert.match(
    namedStep(aggregate, "Aggregate app readiness").run ?? "",
    /app-e2e-aggregate\.mjs readiness all/,
  );
  assert.equal(
    Object.values(jobs).filter((job) => job.name === "app-readiness").length,
    1,
    "only the aggregator may expose the required context",
  );
});

test("full App E2E uses a planner, isolated matrix producers, and stable release evidence", () => {
  const jobs = loadWorkflow().jobs ?? {};
  const plan = jobs["app-e2e-plan"];
  const producer = jobs["app-e2e-producer"];
  const aggregate = jobs["app-e2e"];
  assert.ok(plan && producer && aggregate);
  assert.match(plan.if ?? "", /github\.event_name != 'pull_request'/);
  assert.equal(plan["runs-on"], BLACKSMITH_RUNNER);
  assert.deepEqual(plan.outputs, {
    suite: "${{ steps.plan.outputs.suite }}",
    matrix: "${{ steps.plan.outputs.matrix }}",
    count: "${{ steps.plan.outputs.count }}",
  });
  const validation = namedStep(plan, "Validate target SHA");
  assert.match(validation.run ?? "", /target_sha is required when suite=all/);
  assert.match(validation.run ?? "", /\^\[0-9a-f\]\{40\}\$/);
  assert.match(validation.run ?? "", /RUN_HEAD_SHA/);
  const planning = namedStep(plan, "Plan full rows");
  assert.match(planning.run ?? "", /--profile full --suite/);
  assert.match(planning.run ?? "", /--field count/);
  const fullEvidence = namedStep(plan, "Upload full plan evidence");
  assert.match(String(fullEvidence.with?.name), /github\.run_id.*github\.run_attempt/);

  assert.equal(producer.name, "app-e2e (${{ matrix.suite }}:${{ matrix.shard }})");
  assert.equal(producer.needs, "app-e2e-plan");
  assert.equal(producer["runs-on"], "${{ matrix.runner }}");
  assert.equal(producer.strategy?.["fail-fast"], false);
  assert.equal(producer.strategy?.matrix, "${{ fromJSON(needs.app-e2e-plan.outputs.matrix) }}");
  assert.ok(producer.steps?.some((step) => step.uses === "./.github/actions/app-e2e-row"));
  assert.match(namedStep(producer, "Verify producer SHA").run ?? "", /git rev-parse HEAD/);

  assert.equal(aggregate.name, "app-e2e");
  assert.equal(aggregate["runs-on"], BLACKSMITH_RUNNER);
  assert.deepEqual(aggregate.needs, ["app-e2e-plan", "app-e2e-producer"]);
  assert.match(aggregate.if ?? "", /always\(\)/);
  assert.match(
    namedStep(aggregate, "Aggregate full App E2E").run ?? "",
    /app-e2e-aggregate\.mjs full/,
  );
  assert.equal(Object.values(jobs).filter((job) => job.name === "app-e2e").length, 1);
});

test("every producer and aggregator checks out the exact event target", () => {
  const jobs = loadWorkflow().jobs ?? {};
  for (const jobId of ["app-readiness-plan", "app-readiness-producer", "app-readiness"]) {
    const checkout = jobs[jobId]?.steps?.find((step) => step.uses?.startsWith("actions/checkout@"));
    assert.equal(checkout?.with?.ref, "${{ github.sha }}", jobId);
    assert.equal(checkout?.with?.["persist-credentials"], false, jobId);
  }
  for (const jobId of ["testbox", "app-e2e-plan", "app-e2e-producer", "app-e2e"]) {
    const checkout = jobs[jobId]?.steps?.find((step) => step.uses?.startsWith("actions/checkout@"));
    assert.equal(checkout?.with?.ref, "${{ env.E2E_TARGET_SHA }}", jobId);
    assert.equal(checkout?.with?.["persist-credentials"], false, jobId);
  }
  assert.match(
    namedStep(jobs.testbox, "Validate optional target SHA").run ?? "",
    /\^\[0-9a-f\]\{40\}\$/,
  );
});

test("App E2E producers take their runner from the plan; support jobs stay Blacksmith", () => {
  const source = readRepoFile(".github", "workflows", "e2e.yml");
  // Which label each row gets is the planner's call and is asserted in
  // app-e2e-plan.test.ts; the workflow only has to defer to it.
  for (const job of ["app-readiness-producer", "app-e2e-producer"]) {
    assert.equal(
      workflowJobRunsOn(source, job),
      "${{ matrix.runner }}",
      `${job} must take its runner from the planned row`,
    );
  }
  for (const job of ["app-readiness-plan", "app-readiness", "testbox", "app-e2e-plan", "app-e2e"]) {
    assert.equal(
      workflowJobRunsOn(source, job),
      BLACKSMITH_RUNNER,
      `${job} must run on Blacksmith`,
    );
  }
});

test("shared row action validates the plan and never parallelizes fixture processes", () => {
  const source = readRepoFile(".github", "actions", "app-e2e-row", "action.yml");
  const action = parse(source) as {
    inputs?: Record<string, unknown>;
    runs?: { steps?: Step[]; using?: string };
  };
  assert.equal(action.runs?.using, "composite");
  for (const input of [
    "profile",
    "suite",
    "shard",
    "task",
    "timeout",
    "needs-playwright",
    "cache-key",
    "worktree-id",
    "artifact-stem",
  ]) {
    assert.ok(action.inputs?.[input], input);
  }
  const hydration = namedStep(
    { steps: action.runs?.steps },
    "Validate and hydrate planned fixtures",
  );
  assert.match(hydration.run ?? "", /app-e2e-plan\.mjs/);
  assert.match(hydration.run ?? "", /mapfile -t fixture_paths/);
  assert.match(
    hydration.run ?? "",
    /git submodule update --init --recursive --depth 1 -- "\$\{fixture_paths\[@\]\}"/,
  );
  const run = namedStep({ steps: action.runs?.steps }, "Run planned App E2E row");
  assert.equal(run["continue-on-error"], true);
  assert.doesNotMatch(run.run ?? "", /VIZE_BATCH_CHECK_BUDGET_SCALE/);
  assert.equal(run.env?.VIZE_BATCH_CHECK_BUDGET_SCALE, undefined);
  assert.match(run.run ?? "", /timeout --signal=TERM --kill-after=15s "\$PLANNED_TIMEOUT"/);
  assert.match(run.run ?? "", /vp run --no-cache --filter '\.\/tests' "\$PLANNED_TASK"/);
  assert.doesNotMatch(run.run ?? "", /(?:^|\s)&(?:\s|$)/m);
  const upload = namedStep({ steps: action.runs?.steps }, "Upload App E2E row artifacts");
  assert.match(String(upload.if), /steps\.run\.outcome == 'failure'/);
  assert.match(String(upload.with?.name), /artifact-stem.*github\.run_id.*github\.run_attempt/);
  const browser = namedStep({ steps: action.runs?.steps }, "Cache Playwright browsers");
  assert.equal(browser.if, "inputs.needs-playwright == 'true'");
  assert.equal(
    action.runs?.steps?.find((step) => step.uses === "./.github/actions/setup-rust-sticky-cache")
      ?.with?.key,
    "${{ inputs.cache-key }}",
  );
});

test("Testbox still hydrates the exact 16-fixture App inventory", () => {
  const action = readRepoFile(".github", "actions", "hydrate-app-fixtures", "action.yml");
  const fixtures = [
    "ant-design-vue",
    "directus",
    "element-plus",
    "elk",
    "frontend-phpcon-do-website",
    "hoppscotch",
    "misskey",
    "naive-ui",
    "npmx.dev",
    "nuxt-ui",
    "primevue",
    "reka-ui",
    "voicevox",
    "vue-vben-admin",
    "vuefes-2025",
    "vuetify",
  ];
  for (const fixture of fixtures)
    assert.match(action, new RegExp(`tests/_fixtures/_git/${fixture.replace(".", "\\.")}`));
  assert.equal(action.match(/tests\/_fixtures\/_git\//g)?.length, fixtures.length);
  assert.equal(
    loadWorkflow().jobs?.testbox?.steps?.filter(
      (step) => step.uses === "./.github/actions/hydrate-app-fixtures",
    ).length,
    1,
  );
});
