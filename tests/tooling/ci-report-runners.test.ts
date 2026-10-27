import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
const { parse } = createRequire(import.meta.url)("yaml") as typeof import("yaml");
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
type Job = Record<string, unknown> & { steps: Record<string, unknown>[] };
const reports = [
  {
    file: ".github/workflows/pr-rust-products.yml",
    id: "rust-product-report",
    needs: ["pr-rust-shard"],
    permissions: {
      contents: "read",
      actions: "read",
      "pull-requests": "read",
    },
    if: "${{ always() && !contains(needs.*.result, 'cancelled') }}",
    timeout: 10,
    leaf: {
      name: "Require the complete Rust tier",
      env: {
        RUN_RUST: "${{ inputs.run-rust }}",
        NEEDS_JSON: "${{ toJSON(needs) }}",
        GH_TOKEN: "${{ github.token }}",
      },
      run: "node tools/support/compat/github/require-rust-tier.ts --products",
    },
  },
  {
    file: ".github/workflows/pr-rust-checks.yml",
    id: "rust-source-report",
    needs: ["merge-rust-differential", "pr-rust-build"],
    permissions: {
      contents: "read",
      actions: "read",
    },
    if: "${{ always() && !contains(needs.*.result, 'cancelled') }}",
    timeout: 10,
    leaf: {
      name: "Require the complete Rust tier",
      env: {
        RUN_RUST: "${{ inputs.run-rust }}",
        NEEDS_JSON: "${{ toJSON(needs) }}",
      },
      run: "node tools/support/compat/github/require-rust-tier.ts --source",
    },
  },
  {
    file: ".github/workflows/pr-source-checks.yml",
    id: "source-report",
    needs: [
      "pr-source-plan",
      "pr-rust-source",
      "pr-rust-products",
      "pr-js-packages",
      "pr-native-runtime",
      "pr-playground-test",
      "nuxt-module-qualification",
      "canonical-corpus",
      "ci-selection-tests",
      "wasm-release-check",
      "affected-tooling-report",
    ],
    permissions: {
      contents: "read",
      actions: "read",
      "pull-requests": "read",
    },
    if: "${{ always() && !contains(needs.*.result, 'cancelled') }}",
    timeout: 5,
    leaf: {
      name: "Require every source gate to succeed",
      env: {
        NEEDS_JSON: "${{ toJSON(needs) }}",
        GH_TOKEN: "${{ github.token }}",
      },
      run: "node tools/support/compat/github/canonical-corpus-selection.mjs --check",
    },
  },
  {
    file: ".github/workflows/check.yml",
    id: "test-report",
    needs: [
      "iteration-plan",
      "fmt-rust",
      "check-js",
      "security-audit",
      "node-engine-compat",
      "check-vize-apps",
      "pr-source-checks",
      "instruction-counts",
      "level-dependency-direction",
    ],
    permissions: {
      contents: "read",
      "pull-requests": "read",
      actions: "read",
    },
    if: "${{ always() && (github.event_name == 'pull_request' || github.event_name == 'merge_group') }}",
    timeout: 5,
    leaf: {
      name: "Require every needed job to succeed",
      env: {
        NEEDS_JSON: "${{ toJSON(needs) }}",
      },
      run: "node tools/support/compat/github/require-needs-success.ts --check-report",
    },
  },
];
for (const expected of reports) {
  void test(expected.id + " keeps its complete failure contract on the hosted report pool", () => {
    const workflow = parse(readFileSync(path.join(root, expected.file), "utf8")) as {
      jobs: Record<string, Job>;
    };
    const job = workflow.jobs[expected.id];
    assert.equal(job["runs-on"], "ubuntu-24.04");
    assert.deepEqual(job.needs, expected.needs);
    assert.deepEqual(job.permissions, expected.permissions);
    assert.equal(job.if, expected.if);
    assert.equal(job["timeout-minutes"], expected.timeout);
    assert.deepEqual(job.steps.at(-1), expected.leaf);
    assert.deepEqual(
      job.steps.filter((step) => Object.hasOwn(step, "run")).map((step) => step.run),
      [expected.leaf.run],
    );
  });
}
