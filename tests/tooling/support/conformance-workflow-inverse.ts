import assert from "node:assert/strict";
import { restoreConformanceNativeStaging } from "./native-staging-inverse.ts";

// The exact-tsgo-project job outgrew its 20-minute budget: a cold build cache
// spends ~10 min before the 20 stress cycles (~13-15 min). Sealed workflow
// chains invert each change.
const raisedTimeout = "    timeout-minutes: 35\n";
const formerTimeout = "    timeout-minutes: 20\n";

function restoreExactTsgoTimeout(workflow: string): string {
  assert.equal(workflow.split(raisedTimeout).length, 2, "exact-tsgo-project timeout");
  const job = workflow.indexOf("\n  exact-tsgo-project:\n");
  const at = workflow.indexOf(raisedTimeout);
  assert(job >= 0 && at > job && at < workflow.indexOf("    steps:\n", job));
  return workflow.replace(raisedTimeout, formerTimeout);
}

// #12458 appended its fallthrough value suite to the prop-problem command.
const fallthroughFilter = " content_mapper_tsgo_cli::fallthrough_value_12458::";
const propProblemFilter =
  " content_mapper_tsgo_cli::standard_tsgo_reports_each_component_prop_problem_once";

function restoreFallthroughValueSuite(workflow: string): string {
  assert.equal(workflow.split(fallthroughFilter).length, 2, "fallthrough value suite");
  const appended = propProblemFilter + fallthroughFilter + " --nocapture\n";
  assert.equal(workflow.split(appended).length, 2, "fallthrough value suite command");
  return workflow.replace(fallthroughFilter, "");
}

/** Restores content-mapper-conformance.yml before its sealed command-local edits. */
export function restoreConformanceWorkflow(workflow: string): string {
  return restoreConformanceNativeStaging(
    restoreFallthroughValueSuite(restoreExactTsgoTimeout(workflow)),
  );
}
