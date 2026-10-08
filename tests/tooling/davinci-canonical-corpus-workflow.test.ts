import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import {
  canonicalCorpusRequired,
  canonicalSourceReport,
} from "../../tools/support/compat/github/canonical-corpus-selection.mjs";
import { changedPaths } from "../../tools/support/compat/github/plan-source-checks.ts";
import { readRepoFile } from "./support/github-workflows.ts";

await test("actual input edits, removals and renames select the full canonical corpus", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-canonical-selection-"));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: directory, encoding: "utf8" }).trim();
  try {
    git("init", "--quiet");
    git("config", "user.email", "fixture@example.invalid");
    git("config", "user.name", "Fixture");
    writeFileSync(join(directory, ".gitmodules"), "original\n");
    git("add", ".gitmodules");
    git("commit", "--quiet", "-m", "original");
    const original = git("rev-parse", "HEAD");
    git("mv", ".gitmodules", "authored metadata.txt");
    git("commit", "--quiet", "-m", "rename");
    const renamed = git("rev-parse", "HEAD");
    assert.deepEqual(changedPaths(original, renamed, directory), [
      ".gitmodules",
      "authored metadata.txt",
    ]);
    for (const event of ["pull_request", "merge_group"])
      assert.equal(
        canonicalCorpusRequired(changedPaths(original, renamed, directory), event),
        true,
      );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

await test("selection covers pinned inputs, drivers and real producers in both contexts", () => {
  const inputs = [
    ".gitmodules",
    "tests/_fixtures/_git/vue-benchmarks",
    "tests/_fixtures/fixture-compatibility-ledger.json",
    "tests/_fixtures/vue-benchmarks-upstream.json",
    "tools/commands/fixtures/davinci-dom-corpus-workflow.rs",
    "tools/support/compat/fixtures/davinci-dom-corpus-workflow.mjs",
    "tools/support/compat/davinci/lib/corpus-baseline-contract.mjs",
    "tools/support/compat/github/canonical-corpus-selection.mjs",
    "tools/support/compat/github/canonical-corpus-identity.mjs",
    "tools/support/compat/github/canonical-corpus-inventory.mjs",
    "tools/support/compat/github/canonical-corpus-hydration.mjs",
    "tools/support/compat/github/canonical-corpus-observer.mjs",
    "tools/support/compat/github/canonical-corpus-workers.mjs",
    "tests/tooling/support/canonical-corpus-worker-inventory.mjs",
    "tests/tooling/fixtures/canonical-observer-logs/dom.log",
    "tools/support/compat/github/comparison-base.mjs",
    "tools/support/compat/github/comparison-base.ts",
    "tools/support/compat/github/plan-source-checks.ts",
    "tools/benchmarks/scripts/vue-benchmarks-current-typecheck.mjs",
    ".github/workflows/davinci-canonical-corpus.yml",
    ".github/workflows/pr-source-checks.yml",
    ".github/actions/setup-rust-script/action.yml",
    "davinci/vize_l1/src/lib.rs",
    "crates/vize_atelier_ssr/src/lib.rs",
    "crates/vize_armature/src/lib.rs",
    "Cargo.lock",
  ];
  for (const event of ["pull_request", "merge_group"]) {
    for (const input of inputs) assert.equal(canonicalCorpusRequired([input], event), true, input);
    assert.equal(canonicalCorpusRequired([], event), true);
    assert.equal(canonicalCorpusRequired(["docs/status.md"], event), false);
    assert.equal(canonicalCorpusRequired(["crates/vize_maestro/src/ide/rename.rs"], event), false);
  }
  assert.throws(() => canonicalCorpusRequired([".gitmodules"], "workflow_dispatch"));
  for (const path of ["/outside", "../escape", "a/./b", "", "a\0b"])
    assert.throws(() => canonicalCorpusRequired([path], "pull_request"));
  assert.throws(() => canonicalCorpusRequired([".gitmodules", "../escape"], "merge_group"));
});

const needs = (required: string, result: string, event = "pull_request") => ({
  "pr-source-plan": { result: "success", outputs: { "canonical-corpus": required } },
  "pr-rust-source": { result: "success" },
  "pr-js-packages": { result: "success" },
  "pr-tooling-scripts": { result: "success" },
  "pr-playground-test": { result: "success" },
  "wit-contracts": { result: event === "merge_group" ? "success" : "skipped" },
  "canonical-corpus": { result },
});

await test("a required canonical execution cannot be skipped, failed or hidden by aggregation", () => {
  for (const event of ["pull_request", "merge_group"]) {
    assert.equal(canonicalSourceReport(needs("true", "success", event), event).exitCode, 0);
    assert.equal(canonicalSourceReport(needs("false", "skipped", event), event).exitCode, 0);
    for (const result of ["skipped", "failure", "cancelled", "", "queued"])
      assert.throws(() => canonicalSourceReport(needs("true", result, event), event));
    for (const required of ["", "TRUE", "null"])
      assert.throws(() => canonicalSourceReport(needs(required, "success", event), event));
    for (const gate of [
      "pr-rust-source",
      "pr-js-packages",
      "pr-tooling-scripts",
      "pr-playground-test",
    ])
      assert.equal(
        canonicalSourceReport(
          { ...needs("false", "skipped", event), [gate]: { result: "failure" } },
          event,
        ).exitCode,
        1,
      );
    assert.throws(() =>
      canonicalSourceReport(
        { ...needs("true", "success", event), "pr-source-plan": { result: "failure" } },
        event,
      ),
    );
  }
  assert.equal(canonicalSourceReport(needs("false", "skipped"), "merge_group").exitCode, 1);
});

await test("ordinary source and queue reports require the strict reusable execution", () => {
  const source = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml"));
  const corpus = parse(readRepoFile(".github", "workflows", "davinci-canonical-corpus.yml"));
  const matrix = parse(readRepoFile(".github", "workflows", "real-project-matrix.yml"));
  const worker = corpus.jobs["canonical-observers"];
  assert.equal(corpus.jobs["davinci-dom-corpus"].needs, "canonical-observers");
  assert.equal(corpus.jobs["davinci-dom-corpus"].if, "${{ always() }}");
  assert.equal(worker.strategy["fail-fast"], false);
  assert.equal(worker.strategy["max-parallel"], 3);
  assert.deepEqual(worker.strategy.matrix.observer, ["dom", "ssr-pug", "reach"]);
  assert.equal(
    worker.steps.find((step) => step.name === "Upload required observer evidence").with.overwrite,
    undefined,
  );
  assert.deepEqual(matrix.jobs["davinci-dom-corpus"], {
    permissions: { contents: "read", actions: "read" },
    uses: "./.github/workflows/davinci-canonical-corpus.yml",
    with: { davinci_dom_corpus_mode: "${{ inputs.davinci_dom_corpus_mode || 'enforce' }}" },
  });
  assert.deepEqual(source.jobs["canonical-corpus"], {
    permissions: { contents: "read", actions: "read" },
    needs: "pr-source-plan",
    if: "${{ !cancelled() && needs.pr-source-plan.result == 'success' && needs.pr-source-plan.outputs.canonical-corpus == 'true' }}",
    uses: "./.github/workflows/davinci-canonical-corpus.yml",
  });
  assert.ok(source.jobs["source-report"].needs.includes("canonical-corpus"));
  assert.equal(
    source.jobs["source-report"].steps.at(-1).run,
    "node tools/support/compat/github/canonical-corpus-selection.mjs --check",
  );
  assert.equal(corpus.on.workflow_call.inputs.davinci_dom_corpus_mode.default, "enforce");
  assert.deepEqual(corpus.permissions, { contents: "read" });
  assert.deepEqual(corpus.env, {
    CARGO_TERM_COLOR: "always",
    FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true,
  });
  assert.equal(corpus.jobs["davinci-dom-corpus"]["timeout-minutes"], 120);
});
