import assert from "node:assert/strict";

type Job = { steps?: Record<string, unknown>[] };
const checkout = "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd";
const contracts = {
  "pr-source-plan": {
    files: [".github/source-plan-sparse-checkout"],
    inline: true,
    expansion: "git sparse-checkout set --no-cone --stdin < .github/source-plan-sparse-checkout",
  },
  "iteration-plan": {
    depth: 2,
    files: [
      "package.json",
      "tools/support/compat/github/plan-check-inputs.ts",
      "tools/support/compat/github/comparison-base.ts",
      "tools/support/github/opinionated-source-inputs.ts",
    ],
  },
  "test-report": {
    depth: 1,
    files: [
      "package.json",
      "tools/support/compat/github/require-needs-success.ts",
      "tools/support/compat/github/comment-test-report.ts",
      "tools/support/compat/github/test-cases.ts",
      "tools/support/compat/github/test-report-markdown.ts",
      "tools/support/compat/github/test-areas.ts",
      "tools/support/json-value.ts",
    ],
  },
} as const;

/** Invert only independently exercised checkout input changes. */
export function restoreReviewedCheckout(current: Job, original: Job, site: keyof typeof contracts) {
  assert.ok(Object.hasOwn(contracts, site), "unreviewed checkout site");
  const before = original.steps?.[0];
  const steps = current.steps;
  assert.ok(before && steps);
  assert.equal(before.uses, checkout);
  assert.equal(
    steps.filter(
      (step) => typeof step.uses === "string" && step.uses.startsWith("actions/checkout@"),
    ).length,
    1,
    "exactly one reviewed checkout is required",
  );
  const contract = contracts[site];
  assert.deepEqual(steps[0], {
    ...before,
    with: {
      ...("depth" in contract ? { "fetch-depth": contract.depth } : {}),
      "sparse-checkout-cone-mode": false,
      "sparse-checkout": contract.files
        .map((file) => `/${file}`)
        .join("\n")
        .concat("inline" in contract ? "" : "\n"),
    },
  });
  steps[0] = structuredClone(before);
  if ("expansion" in contract) {
    assert.deepEqual(steps[1], {
      name: "Expand the source planner checkout",
      run: contract.expansion,
    });
    steps.splice(1, 1);
  }
}
