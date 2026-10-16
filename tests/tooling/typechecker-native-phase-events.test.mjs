import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { matchesGlob } from "node:path";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

const workflow = parse(
  readFileSync(
    new URL("../../.github/workflows/typechecker-native-phases.yml", import.meta.url),
    "utf8",
  ),
);
const job = workflow.jobs["native-phases"];
const evidenceName = "Actual CLI shard native phase evidence";
const sourceSha = "1".repeat(40);
const baseSha = "2".repeat(40);

void test("native phase triggers follow consumed actions and preserve product/build inputs", () => {
  const selected = (file) => workflow.on.pull_request.paths.some((glob) => matchesGlob(file, glob));
  for (const file of [
    ".github/actions/check-nix/action.yml",
    ".github/actions/check-vize-apps/action.yml",
    ".github/actions/check-workflow-syntax/check.ts",
    "docs/content/guide.md",
  ])
    assert.equal(selected(file), false, file);
  for (const action of [
    "check-reference-path",
    "check-batch-generated-config",
    "check-computed-inlay",
    "check-editor-jsconfig",
    "check-package-private-imports",
    "check-component-tag-types",
    "check-project-navigation",
    "setup-rust-sticky-cache",
    "reuse-rust-target",
  ])
    assert.equal(selected(`.github/actions/${action}/run.sh`), true, action);
  for (const file of [
    "tools/support/compat/github/native-syntax-semantics.sh",
    "tools/support/compat/github/native-javascript-sfc-membership.sh",
    "tools/support/compat/github/native-empty-vue-star-exports.mjs",
    "tools/support/compat/github/native-lexical-dollar-receipt.mjs",
    "tools/support/compat/github/native-lexical-javascript-source.mjs",
    "tools/benchmarks/scripts/typechecker-template-emit-projection-receipt.mjs",
    "crates/vize_canon/src/lib.rs",
    "crates/vize_croquis/src/lib.rs",
    "crates/vize_maestro/src/ide/completion.rs",
    "crates/vize/tests/check_reference_path_cli.rs",
    "tests/_fixtures/differential/typechecker/outside-import-types/input.json",
    ".cargo/config.toml",
    "rust-toolchain.toml",
    "Cargo.toml",
    "Cargo.lock",
    "package.json",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "patches/example.patch",
    "tools/config/vite-plus/tasks/check.ts",
    ".gitmodules",
    "tests/_fixtures/_git/element-plus",
  ])
    assert.equal(selected(file), true, file);
});

// Official edited-event shape, pinned in the decision record: changes.base is
// separate from changes.body/title and carries both ref.from and sha.from.
const baseChange = { base: { ref: { from: "stack-parent" }, sha: { from: baseSha } } };
const bodyChange = { body: { from: "Previous description" } };
const titleChange = { title: { from: "Previous title" } };

function pullRequest(action, changes, runId = 100) {
  return {
    event_name: "pull_request",
    run_id: runId,
    event: {
      action,
      ...(changes && { changes }),
      pull_request: {
        number: 8561,
        head: { sha: sourceSha },
        base: { ref: "main", sha: baseSha },
      },
    },
  };
}

function evaluate(value, github) {
  // Execute the actual YAML expression, never a separately maintained policy.
  // These fixtures use nonempty strings, objects and booleans, for which this
  // expression subset has the same truthiness as Actions. Missing maps expose
  // absent properties without inventing a base change.
  const expression = /^\$\{\{\s*(.*?)\s*\}\}$/.exec(value)?.[1];
  assert.ok(expression, `expected an Actions expression: ${value}`);
  assert.doesNotMatch(expression, /\b(?:contains|startsWith|endsWith|fromJSON|toJSON)\b/);
  return runInNewContext(expression, {
    github: {
      ...github,
      event: { changes: {}, pull_request: {}, ...github.event },
    },
    format: (pattern, input) => pattern.replace("{0}", String(input)),
  });
}

function plan(github) {
  const runs = Boolean(evaluate(job.if, github));
  return {
    group: evaluate(workflow.concurrency.group, github),
    // Hosted skipped jobs retain their literal name expression.
    name: runs ? evaluate(job.name, github) : job.name,
    runs,
  };
}

function displacedBy(active, incoming) {
  return active.group === incoming.group && workflow.concurrency["cancel-in-progress"];
}

void test("body/title edits neither restart nor replace running or pending source evidence", () => {
  const source = plan(pullRequest("synchronize", undefined, 99));
  assert.equal(workflow.concurrency["cancel-in-progress"], true);
  for (const changes of [bodyChange, titleChange, { ...bodyChange, ...titleChange }, {}]) {
    const first = plan(pullRequest("edited", changes, 100));
    const second = plan(pullRequest("edited", changes, 101));
    assert.equal(first.runs, false);
    assert.equal(second.runs, false);
    assert.notEqual(first.name, evidenceName);
    assert.notEqual(second.name, evidenceName);
    assert.notEqual(first.group, second.group, "metadata events have unique groups");
    assert.equal(displacedBy(source, first), false, "active source evidence survives");
    assert.equal(displacedBy(source, second), false, "pending source evidence survives");
    assert.equal(displacedBy(first, source), false, "source updates ignore inert groups");
  }
  // This counterfactual is the original bug: merely adding job.if leaves both
  // genuine and skipped runs in the same group, which cancels genuine work.
  const oldGroup = "typechecker-native-phases-8561";
  assert.equal(displacedBy({ group: oldGroup }, { group: oldGroup }), true);
});

void test("real base retargets keep the original evidence name and supersede stale source runs", () => {
  const original = plan(pullRequest("synchronize", undefined, 99));
  for (const changes of [baseChange, { ...baseChange, ...bodyChange, ...titleChange }]) {
    const retarget = plan(pullRequest("edited", changes));
    assert.equal(retarget.runs, true);
    assert.equal(retarget.name, evidenceName);
    assert.equal(retarget.group, "typechecker-native-phases-8561");
    assert.equal(displacedBy(original, retarget), true);
  }
});

void test("all source/new-head triggers and immutable manual evidence remain enabled", () => {
  assert.deepEqual(workflow.on.pull_request.types, ["opened", "synchronize", "reopened", "edited"]);
  assert.equal(workflow.on.workflow_dispatch.inputs.source_sha.required, true);
  assert.ok(
    workflow.on.pull_request.paths.includes(".github/workflows/typechecker-native-phases.yml"),
  );
  // The steps load one tooling module; other tooling changes do not start this workflow.
  assert.ok(
    workflow.on.pull_request.paths.includes("tests/tooling/davinci-storage-rust-syntax.ts"),
  );
  assert.ok(!workflow.on.pull_request.paths.includes("tests/tooling/**"));
  for (const action of ["opened", "synchronize", "reopened"]) {
    const source = plan(pullRequest(action));
    assert.equal(source.runs, true);
    assert.equal(source.name, evidenceName);
    assert.equal(source.group, "typechecker-native-phases-8561");
  }
  const manual = plan({ event_name: "workflow_dispatch", run_id: 102, event: {} });
  assert.equal(manual.runs, true);
  assert.equal(manual.name, evidenceName);
  assert.equal(manual.group, "typechecker-native-phases-102");
});
