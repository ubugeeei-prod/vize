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

// #13021 appended its library-generic fallthrough suite after #12458's.
const libraryFallthroughFilter = " content_mapper_tsgo_cli::library_generic_fallthrough_13021::";

function restoreLibraryFallthroughSuite(workflow: string): string {
  assert.equal(workflow.split(libraryFallthroughFilter).length, 2, "library fallthrough suite");
  const appended = fallthroughFilter + libraryFallthroughFilter + " --nocapture\n";
  assert.equal(workflow.split(appended).length, 2, "library fallthrough suite command");
  return workflow.replace(libraryFallthroughFilter, "");
}

// The packaged library-generic data-attr suite followed #13021's.
const packagedDataAttrFilter = " content_mapper_tsgo_cli::packaged_generic_data_attrs::";

function restorePackagedDataAttrSuite(workflow: string): string {
  assert.equal(workflow.split(packagedDataAttrFilter).length, 2, "packaged data-attr suite");
  const appended = libraryFallthroughFilter + packagedDataAttrFilter + " --nocapture\n";
  assert.equal(workflow.split(appended).length, 2, "packaged data-attr suite command");
  return workflow.replace(packagedDataAttrFilter, "");
}

// Reverse only the reviewed Rust 1.100.0 pin edit (#12954) on its two installers.
function restoreRustToolchainPin(workflow: string): string {
  assert.equal(workflow.split('toolchain: "1.100.0"').length, 3);
  return workflow.replaceAll('toolchain: "1.100.0"', 'toolchain: "1.99.1"');
}

/** Restores content-mapper-conformance.yml before its sealed command-local edits. */

export function restoreConformanceWorkflow(workflow: string): string {
  return restoreConformanceNativeStaging(
    restoreFallthroughValueSuite(
      restoreExactTsgoTimeout(
        restoreLibraryFallthroughSuite(
          restorePackagedDataAttrSuite(restoreRustToolchainPin(workflow)),
        ),
      ),
    ),
  );
}
