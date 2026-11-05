import assert from "node:assert/strict";

type Step = { name?: string; if?: string; [key: string]: unknown };
type Job = { steps?: Step[] };

export const helperAbiStep = {
  name: "Check content mapper helper ABI",
  if: "${{ needs.iteration-plan.outputs.engines == 'true' }}",
  run: "node npm/cli/scripts/content-mapper-helper-abi.ts --check",
} satisfies Step;

export function restoreContentMapperHelperAbi(job: Job) {
  const steps = job.steps;
  assert(steps);
  const matches = steps.flatMap((step, index) => (step.name === helperAbiStep.name ? [index] : []));
  assert.equal(matches.length, 1, "exactly one reviewed helper ABI step is required");
  const index = matches[0];
  assert.deepEqual(steps[index], helperAbiStep);
  assert.equal(steps[index - 1]?.name, "Check package engine matrix");
  assert.equal(steps[index + 1]?.name, "Report unchanged inputs");
  steps.splice(index, 1);
}
