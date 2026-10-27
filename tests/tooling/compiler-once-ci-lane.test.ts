import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { differentialSteps } from "../../tools/support/compat/github/rust-differential-lanes.ts";

test("the required compiler differential executes the cached component vnode witness", () => {
  const steps = differentialSteps("compiler", ["vize_atelier_dom"]);
  const witness = steps.find((step) => step.args.includes("component_v_once"));
  assert.ok(witness, "default workspace tests skip the feature-gated v-once witness");
  assert.ok(witness.args.includes("legacy-differential"));
  assert.ok(witness.args.includes("--test"));
  const manifest = readFileSync(
    new URL("../../crates/vize_atelier_dom/Cargo.toml", import.meta.url),
    "utf8",
  );
  assert.match(
    manifest,
    /\[\[test\]\]\nname = "component_v_once"\nrequired-features = \["legacy-differential"\]/,
  );
});
