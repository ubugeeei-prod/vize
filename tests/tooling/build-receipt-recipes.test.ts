import assert from "node:assert/strict";
import { test } from "node:test";

import {
  BUILD_RECIPE,
  LEGACY_BUILD_RECIPE,
  MATRIX_BUILD_RECIPE,
  PROFILE_BUILD_RECIPE,
  SHIPPING_BUILD_RECIPE,
  binaryRelativePath,
  recipeBinaryRelativePath,
} from "../differential/build-receipt.ts";

test("each source-build recipe names the profile directory it writes", () => {
  assert.equal(MATRIX_BUILD_RECIPE, "cargo build --profile ci-matrix -p vize");
  assert.equal(recipeBinaryRelativePath(MATRIX_BUILD_RECIPE), binaryRelativePath("ci-matrix"));
  assert.equal(recipeBinaryRelativePath(PROFILE_BUILD_RECIPE), binaryRelativePath("ci-opt"));
  // Existing recipes keep their staging paths.
  for (const recipe of [BUILD_RECIPE, LEGACY_BUILD_RECIPE, SHIPPING_BUILD_RECIPE]) {
    assert.equal(recipeBinaryRelativePath(recipe), binaryRelativePath("ci"));
  }
  assert.equal(recipeBinaryRelativePath(), binaryRelativePath("ci"));
  assert.throws(
    () => recipeBinaryRelativePath("cargo build --profile dev -p vize"),
    /unknown source-build recipe/,
  );
});
