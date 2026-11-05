import assert from "node:assert/strict";

type Job = { steps?: Record<string, unknown>[] };
const checkout = "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd";
const contracts = {
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

/** Invert only #11756/#11778's independently exercised checkout input changes. */
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
      "fetch-depth": contract.depth,
      "sparse-checkout-cone-mode": false,
      "sparse-checkout": contract.files.map((file) => `/${file}\n`).join(""),
    },
  });
  steps[0] = structuredClone(before);
}
