import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  providerClosureRole,
  providerClosureTests,
} from "../../tools/support/compat/content-mapper/provider-closure.ts";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  if?: string;
  run?: string;
  uses?: string;
  shell?: string;
  env?: Record<string, string>;
  with?: Record<string, string>;
};

test("the exact Content Mapper provider closure retains all six original checks", () => {
  assert.equal(providerClosureRole, "standard-go-d6-vue35");
  assert.equal(providerClosureTests.length, 6);
  assert.equal(new Set(providerClosureTests).size, providerClosureTests.length);
  assert.deepEqual(providerClosureTests, [
    "content_mapper_importer_scoped_packages::standard_tsgo_content_mapper_matches_duplicate_package_identities",
    "content_mapper_tsgo_cli::standard_tsgo_checks_vue_project_and_emits_consumable_declarations",
    "content_mapper_tsgo_cli::reported_dynamic_targets::originals_and_error_repair_restore_keep_component_target_types",
    "content_mapper_tsgo_watch::standard_tsgo_watch_enters_idle_with_authored_mapper_diagnostics",
    "content_mapper_tsgo_watch::standard_tsgo_watch_revalidates_authored_vue_edits_and_repairs",
    "content_mapper_tsgo_cli::listener_anchor_11657::checks_anchored_at_wrapping_parentheses_stay_reported",
  ]);
});

test("scheduled conformance always runs and retains the provider closure", () => {
  const workflow = parse(readRepoFile(".github/workflows/content-mapper-conformance.yml")) as {
    jobs: Record<string, { steps: Step[] }>;
  };
  const action = parse(
    readRepoFile(".github/actions/qualify-content-mapper-provider-closure/action.yml"),
  ) as { runs: { using: string; steps: Step[] } };
  const steps = workflow.jobs["exact-tsgo-project"].steps;
  const invocation = steps.find(
    (step) => step.uses === "./.github/actions/qualify-content-mapper-provider-closure",
  );
  const closure = action.runs.steps.find(
    (step) => step.name === "Qualify the complete provider closure",
  );
  const upload = steps.find(
    (step) => step.name === "Preserve whole owner calibration observations",
  );
  assert.equal(action.runs.using, "composite");
  assert.equal(invocation?.if, undefined);
  assert.equal(closure?.if, "${{ github.event.inputs['targeted-test'] != 'generic-body-error' }}");
  assert.equal(closure?.shell, "bash");
  assert.equal(closure?.run, "node tools/support/compat/content-mapper/provider-closure.ts");
  assert.deepEqual(closure?.env, {
    VIZE_CONTENT_MAPPER_PROVIDER_CLOSURE_DIR:
      "${{ runner.temp }}/content-mapper-editor-stress/provider-closure",
    VIZE_TEST_CONTENT_MAPPER_TSGO: "${{ runner.temp }}/tsgo",
    VIZE_TEST_CONTENT_MAPPER_JAVASCRIPT_TSC:
      "${{ github.workspace }}/npm/cli/node_modules/.bin/tsc",
    VIZE_TEST_CONTENT_MAPPER_VUE: "${{ github.workspace }}/npm/cli/node_modules/vue",
    VIZE_TEST_REPORTED_DYNAMIC_TARGETS_PROVIDER_ROLE: "standard-go-d6-vue35",
    VIZE_TEST_REPORTED_DYNAMIC_TARGETS_REQUIRED: "1",
    VIZE_TEST_REPORTED_DYNAMIC_TARGETS_CAPTURE_DIR:
      "${{ runner.temp }}/content-mapper-editor-stress/provider-closure/dynamic-targets",
  });
  assert.match(upload?.with?.path ?? "", /content-mapper-editor-stress/u);
  assert.ok(steps.indexOf(invocation ?? {}) < steps.indexOf(upload ?? {}));
});
