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

/** Restores content-mapper-conformance.yml before its sealed command-local edits. */
export function restoreConformanceWorkflow(workflow: string): string {
  return restoreConformanceNativeStaging(restoreExactTsgoTimeout(workflow));
}
