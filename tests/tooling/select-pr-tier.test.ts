import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { selectPrTier } from "../../tools/support/compat/github/select-pr-tier.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const pr = (paths: string[], labels: string[] = []) =>
  selectPrTier({ eventName: "pull_request", paths, labels });

test("the merge queue always runs the heavy lanes", () => {
  for (const paths of [[], ["README.md"], ["crates/vize_canon/src/lib.rs"]]) {
    assert.equal(selectPrTier({ eventName: "merge_group", paths, labels: [] }), "heavy");
    assert.equal(selectPrTier({ eventName: "merge_group", paths, labels: ["hotfix"] }), "heavy");
  }
  assert.throws(() => selectPrTier({ eventName: "push", paths: [], labels: [] }));
});

test("a PR runs affected checks and explicitly requests duplicate heavy qualification", () => {
  for (const path of [
    "crates/vize_canon/src/lib.rs",
    "crates/vize_patina/src/linter/engine/rule_sets.rs",
    "davinci/vize_l2/src/lib.rs",
    "vendor/lightningcss/src/lib.rs",
    "npm/oxlint/src/index.ts",
    "npm/native/index.d.ts",
    "playground/src/App.vue",
    "Cargo.lock",
    "pnpm-lock.yaml",
    ".github/workflows/pr-js-checks.yml",
    ".github/actions/setup-rust-sticky-cache/action.yml",
    "tools/support/compat/github/canonical-corpus-selection.mjs",
    "tests/davinci_test_support/src/corpus.rs",
    "tests/_fixtures/differential/lint/n8n-cli-config/cases.json",
  ]) {
    assert.equal(pr([path]), "fast", path);
    assert.equal(pr(["docs/content/guide/index.md", path]), "fast", path);
  }
  assert.equal(pr([]), "heavy");
  assert.equal(pr(["docs/content/guide/index.md"], ["full-ci"]), "heavy");
});

test("other PRs and hotfixes leave the heavy lanes to the queue", () => {
  for (const path of [
    "docs/content/guide/index.md",
    "AGENTS.md",
    "tests/tooling/select-pr-tier.test.ts",
    "tools/support/ci/actions-cost-report.ts",
    "npm/ui/src/button.vue",
    "editors/vscode/package.json",
  ])
    assert.equal(pr([path]), "fast", path);
  assert.equal(pr(["crates/vize_atelier_sfc/src/lib.rs"], ["hotfix"]), "fast");
  assert.equal(pr(["crates/vize_atelier_sfc/src/lib.rs"], ["hotfix", "full-ci"]), "heavy");
});

type Job = { if?: string; with?: Record<string, string>; outputs?: Record<string, string> };

test("the three heavy lanes follow the tier and the report still checks every lane", () => {
  const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
    jobs: Record<
      string,
      Job & { steps?: { id?: string; run?: string; env?: Record<string, string> }[] }
    >;
  };
  const plan = source.jobs["pr-source-plan"];
  const tier = plan.steps?.find((step) => step.id === "plan");
  assert.equal(
    tier?.run,
    'node tools/support/compat/github/plan-pr-checks.ts "$BASE_SHA" "$GITHUB_SHA" "$GITHUB_EVENT_NAME"',
  );
  assert.equal(tier?.env?.PR_LABELS, "${{ toJSON(github.event.pull_request.labels.*.name) }}");
  assert.equal(plan.outputs?.heavy, "${{ steps.plan.outputs.heavy }}");
  assert.equal(
    plan.outputs?.["canonical-corpus"],
    "${{ steps.plan.outputs.heavy == 'true' && steps.canonical-corpus-plan.outputs.canonical-corpus || 'false' }}",
  );
  assert.equal(plan.outputs?.playground, "${{ steps.plan.outputs.playground }}");
  assert.equal(source.jobs["pr-js-packages"].with?.["run-original-project"], false);
  const js = parse(readRepoFile(".github", "workflows", "pr-js-checks.yml")) as {
    on: { workflow_call: { inputs: Record<string, { required: boolean; type: string }> } };
    jobs: Record<string, Job>;
  };
  assert.deepEqual(js.on.workflow_call.inputs["run-original-project"], {
    required: true,
    type: "boolean",
  });
  assert.equal(js.jobs["pr-oxlint-project-qualification"].if, "${{ inputs.run-original-project }}");
});
