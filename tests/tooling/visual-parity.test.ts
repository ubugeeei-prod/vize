import assert from "node:assert/strict";
import { test } from "node:test";

import { visualComparisonDimensions } from "../_helpers/visual-parity.ts";

test("visual parity compares the shared viewport width and full page height", () => {
  assert.deepEqual(
    visualComparisonDimensions({ height: 15_617, width: 3_349 }, { height: 15_617, width: 1_280 }),
    { height: 15_617, width: 1_280 },
  );

  assert.deepEqual(
    visualComparisonDimensions({ height: 720, width: 1_280 }, { height: 900, width: 1_280 }),
    { height: 900, width: 1_280 },
  );
});
