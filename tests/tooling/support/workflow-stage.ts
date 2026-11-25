import assert from "node:assert/strict";
import { existsSync } from "node:fs";

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
        "/.github/actions/stage-workflow-scripts/\n/tools/ci/lib/scripts/\n" +
        (existsSync(new URL(`../../../tools/ci/workflows/${owner}/`, import.meta.url))
          ? `/tools/ci/workflows/${owner}/scripts/`
          : `/.github/scripts/workflows/${owner}/`),
    },
  });
  assert.deepEqual(stage, {
    name: "Stage scripts outside the source checkout",
    uses: stagedWorkflowAction,
  });
}
