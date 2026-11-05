import assert from "node:assert/strict";
import { test } from "node:test";

import { createSurface } from "../../tools/benchmarks/scripts/compare-tools-report.mjs";
import { CHECK_ENGINE_CLASSES, CHECK_VARIANTS } from "./support/benchmark/check-surface.ts";

function checkSurface(variants = CHECK_VARIANTS) {
  return {
    id: "check",
    label: "Type check",
    files: 500,
    bytes: 1_000_000,
    baselineId: "vue-tsc",
    vizeSingleId: "vize-check-1t",
    vizeMaxId: "vize-check-max",
    engineClasses: CHECK_ENGINE_CLASSES,
    variants,
  };
}

test("generated type-check benchmark surfaces require every engine-class row", () => {
  for (const id of Object.keys(CHECK_ENGINE_CLASSES)) {
    assert.throws(
      () => createSurface(checkSurface(CHECK_VARIANTS.filter((variant) => variant.id !== id))),
      new RegExp(`compare-tools: check is missing required engine-class variants: ${id}`),
      id,
    );
  }
});

test("complete type-check benchmark surfaces publish the vue-tsc ratio", () => {
  const surface = createSurface(checkSurface());

  assert.equal(surface.speedupStatus, "cross-engine");
  assert.equal(surface.speedupBaselineId, "vue-tsc");
  assert.equal(surface.primarySpeedup, 16);
});
