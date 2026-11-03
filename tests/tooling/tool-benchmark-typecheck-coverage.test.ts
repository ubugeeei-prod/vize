import assert from "node:assert/strict";
import { test } from "node:test";

import {
  ENGINE_CLASSES_BY_SURFACE as CLASSES_BY_SURFACE,
  HEADLINE_CHECK_LANES,
  createSurface,
} from "../../tools/benchmarks/scripts/compare-tools-report.mjs";
import { CHECK_ENGINE_CLASSES, CHECK_VARIANTS } from "./support/benchmark/check-surface.ts";

const ENGINE_CLASSES_BY_SURFACE: Record<string, Record<string, string>> = CLASSES_BY_SURFACE;

function checkSurface(variants = CHECK_VARIANTS, id = "check-small") {
  return {
    id,
    label: "Type check",
    files: 500,
    bytes: 1_000_000,
    baselineId: "vue-tsc",
    vizeSingleId: "vize-check-1t",
    vizeMaxId: "vize-check-max",
    engineClasses: ENGINE_CLASSES_BY_SURFACE[id],
    variants,
  };
}

const headline = () =>
  CHECK_VARIANTS.filter((variant) => HEADLINE_CHECK_LANES.includes(variant.id));

test("the small tier and the large SFC require every engine-class row", () => {
  assert.deepEqual(ENGINE_CLASSES_BY_SURFACE["check-small"], CHECK_ENGINE_CLASSES);
  assert.deepEqual(ENGINE_CLASSES_BY_SURFACE["large-check"], CHECK_ENGINE_CLASSES);
  for (const surfaceId of ["check-small", "large-check"]) {
    for (const id of Object.keys(CHECK_ENGINE_CLASSES)) {
      assert.throws(
        () =>
          createSurface(
            checkSurface(
              CHECK_VARIANTS.filter((variant) => variant.id !== id),
              surfaceId,
            ),
          ),
        new RegExp(`compare-tools: ${surfaceId} is missing required engine-class variants: ${id}`),
        id,
      );
    }
  }
});

test("the headline tier runs one lane per tool and requires each of them", () => {
  assert.deepEqual(
    Object.keys(ENGINE_CLASSES_BY_SURFACE.check).sort(),
    [...HEADLINE_CHECK_LANES].sort(),
  );
  for (const id of HEADLINE_CHECK_LANES) {
    assert.equal(ENGINE_CLASSES_BY_SURFACE.check[id], CHECK_ENGINE_CLASSES[id], id);
    assert.throws(
      () =>
        createSurface(
          checkSurface(
            headline().filter((variant) => variant.id !== id),
            "check",
          ),
        ),
      new RegExp(`compare-tools: check is missing required engine-class variants: ${id}`),
      id,
    );
  }
  // A lane outside the headline has no engine class there, so it cannot slip in.
  assert.throws(
    () => createSurface(checkSurface([...headline(), CHECK_VARIANTS[1]!], "check")),
    /variant verter-tsc of surface check has no engine class/u,
  );
});

test("complete type-check benchmark surfaces publish the vue-tsc ratio", () => {
  for (const [variants, id] of [
    [CHECK_VARIANTS, "check-small"],
    [headline(), "check"],
  ] as const) {
    const surface = createSurface(checkSurface(variants, id));
    assert.equal(surface.speedupStatus, "cross-engine");
    assert.equal(surface.speedupBaselineId, "vue-tsc");
    assert.equal(surface.primarySpeedup, 16);
  }
});
