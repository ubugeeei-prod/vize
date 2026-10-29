import assert from "node:assert/strict";
import { test } from "node:test";
import { differentialSteps } from "../../tools/support/compat/github/rust-differential-lanes.ts";

for (const name of ["component_v_once", "element_v_once"]) {
  test(`the required compiler differential executes ${name}`, () => {
    const steps = differentialSteps("compiler", ["vize_atelier_dom"]);
    const witness = steps.find((step) => step.args.includes(name));
    assert.ok(witness, "default workspace tests skip the feature-gated v-once witness");
    assert.ok(witness.args.includes("legacy-differential"));
    assert.ok(witness.args.includes("--test"));
  });
}
