import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

const read = (file: string) => fs.readFileSync(new URL(file, import.meta.url), "utf8");
const originalBytes = read("./fixtures/real-project-matrix-original-67e3.yml");
assert.equal(
  createHash("sha256").update(originalBytes).digest("hex"),
  "3f5650d943dfbb73d9cea41782499c6225938d7ce4330549d4c27f7f90ea267c",
);
const prefix =
  "node tests/differential/build-receipt.ts\n" +
  'node tests/tooling/support/lsp/verify-session-captures.ts snapshot "$FIXTURE_REPORT_DIR/lsp-wire-before.json"\n' +
  "printf '%s\\n' 'Lifecycle timings include passive raw RPC recording; they are not paired performance acceptance.' > \"$FIXTURE_REPORT_DIR/lsp-wire-timing-policy.txt\"\n";

test("passive native LSP capture retains the complete original Matrix workflow", () => {
  const original = parse(originalBytes);
  const current = parse(read("../../.github/workflows/real-project-matrix.yml"));
  assert.deepEqual(original.on.schedule, [{ cron: "37 5 * * 0" }]);
  assert.deepEqual(current.on.schedule, [{ cron: "37 5 * * *" }]);
  current.on.schedule = original.on.schedule;
  // Only the explicit nonempty selector changes scope. Invert its exact additive
  // wiring before comparing every original tool, gate, upload and timeout below.
  assert.deepEqual(current.on.workflow_dispatch.inputs.project, {
    description: "Optional exact fixture ID for a scoped diagnostic; empty runs the full matrix",
    required: false,
    default: "",
    type: "string",
  });
  delete current.on.workflow_dispatch.inputs.project;
  assert.equal(
    current.concurrency.group,
    "real-project-matrix-${{ github.ref }}${{ inputs.project && format('-scoped-{0}', inputs.project) || '' }}",
  );
  current.concurrency.group = original.concurrency.group;
  assert.equal(
    current.jobs["real-project-matrix"].env.FIXTURE_SHARD_COUNT,
    "${{ matrix.shardCount || '22' }}",
  );
  current.jobs["real-project-matrix"].env.FIXTURE_SHARD_COUNT =
    original.jobs["real-project-matrix"].env.FIXTURE_SHARD_COUNT;
  assert.match(current["run-name"], /inputs\.project && 'scoped-diagnostic' \|\| /);
  // Descriptive labels preserve the numeric fixture, cache and artifact routing.
  const matrixJob = current.jobs["real-project-matrix"];
  assert.equal(matrixJob.name, "real projects (${{ matrix.name }})");
  assert.equal(matrixJob.needs, "plan-projects");
  assert.equal(matrixJob.strategy.matrix, "${{ fromJSON(needs.plan-projects.outputs.matrix) }}");
  assert.equal(
    current.jobs["plan-projects"].outputs.matrix,
    "${{ steps.projects.outputs.matrix }}",
  );
  delete current.jobs["plan-projects"];
  delete matrixJob.needs;
  matrixJob.name = original.jobs["real-project-matrix"].name;
  matrixJob.strategy.matrix = original.jobs["real-project-matrix"].strategy.matrix;
  const steps = current.jobs["real-project-matrix"].steps;
  const oldSteps = original.jobs["real-project-matrix"].steps;
  // The report successor passed both immutable Rust references in run 39022693865.
  // Accept only the complete historical or qualified Node step, then invert
  // that one executable before the unchanged whole-workflow comparison.
  const lint = steps.find((step) => step.id === "lint_divergence");
  const oldLint = oldSteps.find((step) => step.id === "lint_divergence");
  assert.ok(lint && oldLint);
  const rustCommand = "rust-script tools/commands/fixtures/lint-divergence-report.rs";
  const nodeCommand = "node tools/commands/fixtures/lint-divergence-report.ts";
  assert.equal(oldLint.run.split(rustCommand).length, 2);
  const nodeRun = oldLint.run.replace(rustCommand, nodeCommand);
  assert.ok([oldLint.run, nodeRun].includes(lint.run));
  assert.deepEqual(lint, { ...oldLint, run: lint.run });
  lint.run = oldLint.run;

  const lsp = steps.find((step) => step.id === "lsp");
  const oldLsp = oldSteps.find((step) => step.id === "lsp");
  assert.ok(lsp && oldLsp);
  assert.deepEqual(lsp.env, {
    ...oldLsp.env,
    VIZE_LSP_REQUIRE_SOURCE_BUILD: "1",
    VIZE_LSP_BUILD_RECIPE: "cargo build --profile ci -p vize",
  });
  assert.equal(lsp.run, prefix + oldLsp.run);
  delete lsp.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
  delete lsp.env.VIZE_LSP_BUILD_RECIPE;
  lsp.run = lsp.run.slice(prefix.length);

  // A retried network-only fetch and step timeouts sized from green shards.
  const fetch = steps.findIndex((step) => step.name === "Fetch Rust dependencies");
  assert.equal(steps[fetch + 1]?.name, "Build Vize CLI");
  assert.match(steps.splice(fetch, 1)[0].run, /cargo fetch --locked/);
  for (const [id, minutes] of [
    ["core_tools", 60],
    ["lsp", 30],
  ] as const) {
    const step = steps.find((candidate) => candidate.id === id);
    assert.equal(step["timeout-minutes"], minutes, id);
    delete step["timeout-minutes"];
  }

  const integrity = steps.findIndex((step) => step.name === "Validate source-bound LSP wire");
  assert.equal(integrity, steps.findIndex((step) => step.id === "lsp") + 1);
  assert.deepEqual(steps.splice(integrity, 1)[0], {
    name: "Validate source-bound LSP wire",
    if: "${{ always() }}",
    run: 'node tests/tooling/support/lsp/verify-session-captures.ts verify "$FIXTURE_REPORT_DIR/lsp-wire-before.json" "$FIXTURE_REPORT_DIR/lsp-wire-integrity.json"',
  });

  const index = steps.findIndex((step) => step.name === "Upload source-bound LSP wire evidence");
  assert.ok(index >= 0);
  const [wire] = steps.splice(index, 1);
  assert.deepEqual(wire, {
    name: "Upload source-bound LSP wire evidence",
    if: "${{ always() }}",
    uses: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
    with: {
      name: "real-project-lsp-wire-${{ matrix.shard }}-${{ github.sha }}",
      path: "target/differential/lsp-sessions\n" + "target/ci/vize.differential-build.json\n",
      "if-no-files-found": "error",
      "retention-days": 30,
    },
  });
  const corpus = current.jobs["davinci-dom-corpus"];
  assert.deepEqual(corpus.permissions, {
    contents: "read",
    actions: "read",
    checks: "read",
  });
  corpus.permissions = original.jobs["davinci-dom-corpus"].permissions;
  // The release-gated title also names the exact
  // release dispatch beside the original enforced and observational titles.
  assert.match(
    original["run-name"],
    /'enforced' \|\| 'observational' \}\} @ \$\{\{ github\.sha \}\}$/,
  );
  assert.match(
    current["run-name"],
    /'enforced' \|\|[\s\S]*'release-gated'\) \|\| 'observational' \}\} @ \$\{\{ github\.sha \}\}$/,
  );
  current["run-name"] = original["run-name"];
  // Retire only the unused compiler bootstrap; keep the archived workflow whole.
  const moon = "./.github/actions/setup-moonbit";
  const moonIndex = oldSteps.findIndex((step) => step.uses === moon);
  assert.equal(oldSteps.filter((step) => step.uses === moon).length, 1);
  assert.deepEqual(oldSteps[moonIndex], { uses: moon });
  assert.equal(
    steps.some((step) => step.uses === moon),
    false,
  );
  assert.deepEqual(steps[moonIndex], oldSteps[moonIndex + 1]);
  steps.splice(moonIndex, 0, oldSteps[moonIndex]);
  assert.deepEqual(current, original);
});
