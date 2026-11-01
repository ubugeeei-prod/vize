import assert from "node:assert/strict";
import { uiConformancePhases } from "../../../.github/actions/test-js-packages-with-history/native-fault-evidence.ts";

type Job = { steps?: Array<{ name?: string; run?: string; if?: string }> };

/** Preserve the original two required commands through their closed observer
 * delegation. The returned shell spellings keep the trusted-main comparison. */
export function requiredUiConformance(job: Job, phases: unknown = uiConformancePhases): string[] {
  const selected = job.steps?.filter((step) => step.name === "Check UI package conformance");
  assert.ok(selected);
  assert.equal(selected.length, 1);
  assert.deepEqual(selected[0], {
    name: "Check UI package conformance",
    if: "${{ inputs.run-js && matrix.surface == 'ui' && fromJSON(inputs.js-plan).conformance.ui }}",
    run: "node .github/actions/test-js-packages-with-history/native-fault-evidence.ts --ui-conformance",
  });
  assert.deepEqual(phases, [
    { name: "native-build", command: ["vp", "run", "--filter", "./npm/native", "build:ci"] },
    { name: "ui-check", command: ["vp", "run", "--filter", "./npm/ui", "check"] },
  ]);
  return ["vp run --filter './npm/native' build:ci", "vp run --filter './npm/ui' check"];
}
