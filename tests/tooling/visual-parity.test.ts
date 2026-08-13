import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { PNG } from "pngjs";

import {
  comparePngBuffers,
  visualComparisonDimensions,
  visualDiffWithinBudget,
} from "../_helpers/visual-parity.ts";

test("visual parity compares the shared viewport width and full page height", () => {
  assert.deepEqual(
    visualComparisonDimensions({ height: 15_617, width: 3_349 }, { height: 15_617, width: 1_280 }),
    { height: 15_617, width: 1_280 },
  );

  assert.deepEqual(
    visualComparisonDimensions({ height: 720, width: 1_280 }, { height: 900, width: 1_280 }),
    { height: 900, width: 1_280 },
  );

  assert.deepEqual(
    visualComparisonDimensions(
      { height: 14_674, width: 1_208 },
      { height: 14_674, width: 1_208 },
      390,
    ),
    { height: 14_674, width: 390 },
  );
});

test("visual parity ignores tiny raster noise but keeps visible pixel changes", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-visual-parity-"));
  const reference = solidPng([250, 250, 250, 255]);
  const nearMatch = solidPng([248, 248, 248, 255]);
  const visibleChange = solidPng([0, 0, 0, 255]);

  assert.equal(
    comparePngBuffers(reference, nearMatch, path.join(dir, "near.png"), { threshold: 0.1 })
      .diffPixels,
    0,
  );
  assert.equal(
    comparePngBuffers(reference, visibleChange, path.join(dir, "visible.png"), { threshold: 0.1 })
      .diffPixels,
    1,
  );
});

test("visual diff budget can cap absolute pixels for narrow long pages", () => {
  assert.equal(
    visualDiffWithinBudget(
      { diffPixels: 41_240, diffRatio: 0.007987279231330897 },
      { maxDiffPixels: 45_000, maxDiffRatio: 0.004 },
    ),
    true,
  );
  assert.equal(
    visualDiffWithinBudget(
      { diffPixels: 41_240, diffRatio: 0.007987279231330897 },
      { maxDiffPixels: 40_000, maxDiffRatio: 0.004 },
    ),
    false,
  );
});

function solidPng([red, green, blue, alpha]: [number, number, number, number]): Buffer {
  const png = new PNG({ height: 1, width: 1 });
  png.data[0] = red;
  png.data[1] = green;
  png.data[2] = blue;
  png.data[3] = alpha;
  return PNG.sync.write(png);
}
