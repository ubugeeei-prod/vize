import assert from "node:assert/strict";

export const stagedWorkflowAction = "./.vize-workflow-tools/.github/actions/stage-workflow-scripts";
export function verifyWorkflowStage(owner: string, checkout: unknown, stage: unknown): void {
  assert.match(owner, /^[a-z0-9-]+$/u);
  assert.deepEqual(checkout, {
    name: "Check out the workflow's scripts",
    uses: "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd",
    with: {
      ref: "${{ github.workflow_sha }}",
      path: ".vize-workflow-tools",
      "persist-credentials": false,
      "sparse-checkout-cone-mode": false,
      "sparse-checkout":
        "/.github/actions/stage-workflow-scripts/\n/.github/scripts/workflows/shared/\n/.github/scripts/workflows/" +
        owner +
        "/",
    },
  });
  assert.deepEqual(stage, {
    name: "Stage scripts outside the source checkout",
    uses: stagedWorkflowAction,
  });
}
