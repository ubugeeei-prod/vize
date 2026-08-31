import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createMeasureCache, type MeasureCacheConfig } from "./virtualizer-measure-cache.ts";

function cacheWith(overrides: Partial<MeasureCacheConfig> = {}) {
  return createMeasureCache({
    getCount: () => 10,
    getLanes: () => 1,
    getGap: () => 0,
    getPaddingStart: () => 0,
    resolveBaseSize: () => 20,
    usesExactSizes: () => false,
    ...overrides,
  });
}

test("places fixed-size items with gap and leading padding", () => {
  const cache = cacheWith({
    getGap: () => 4,
    getPaddingStart: () => 10,
    usesExactSizes: () => true,
  });

  assert.deepEqual(cache.placement(0), {
    start: 10,
    size: 20,
    end: 30,
    lane: 0,
    isMeasured: false,
  });
  assert.deepEqual(cache.placement(2), {
    start: 58,
    size: 20,
    end: 78,
    lane: 0,
    isMeasured: false,
  });
  assert.equal(cache.contentEnd(), 10 + 10 * 20 + 9 * 4);
});

test("resolves variable sizes per index", () => {
  const cache = cacheWith({
    resolveBaseSize: (index) => 10 + index * 10,
    usesExactSizes: () => true,
  });

  assert.equal(cache.placement(0).size, 10);
  assert.equal(cache.placement(3).start, 10 + 20 + 30);
  assert.equal(cache.setMeasured(3, 500), 0, "exact sizes must ignore measurements");
  assert.equal(cache.placement(3).size, 40);
  assert.equal(cache.placement(3).isMeasured, false);
});

test("overrides estimates with measurements and reports deltas", () => {
  const cache = cacheWith();

  assert.equal(cache.placement(5).start, 100);
  assert.equal(cache.setMeasured(2, 50), 30);
  assert.equal(cache.placement(2).size, 50);
  assert.equal(cache.placement(2).isMeasured, true);
  assert.equal(cache.placement(5).start, 130);
  assert.equal(cache.contentEnd(), 10 * 20 + 30);
  assert.equal(cache.setMeasured(2, 50), 0, "an unchanged measurement reports no delta");
  assert.equal(cache.measuredSize(2), 50);
});

test("assigns lanes round-robin with independent offsets", () => {
  const cache = cacheWith({ getLanes: () => 2, getCount: () => 5 });

  assert.equal(cache.placement(0).lane, 0);
  assert.equal(cache.placement(1).lane, 1);
  assert.equal(cache.placement(2).start, 20);
  assert.equal(cache.placement(3).start, 20);
  assert.equal(cache.laneLength(0), 3);
  assert.equal(cache.laneLength(1), 2);
  assert.equal(cache.laneIndexAt(1, 1), 3);

  cache.setMeasured(1, 100);
  assert.equal(cache.placement(3).start, 100, "lane one shifts");
  assert.equal(cache.placement(2).start, 20, "lane zero is untouched");
  assert.equal(cache.contentEnd(), 100 + 20);
});

test("finds the first visible position by offset", () => {
  const cache = cacheWith({ getCount: () => 100 });

  assert.equal(cache.firstVisiblePosition(0, 0), 0);
  assert.equal(cache.firstVisiblePosition(0, 20), 1, "an exact boundary belongs to the next item");
  assert.equal(cache.firstVisiblePosition(0, 199), 9);
  assert.equal(cache.firstVisiblePosition(0, 10_000), 99, "past the end clamps to the last item");
});

test("truncates cached offsets from an invalidated index", () => {
  let size = 20;
  const cache = cacheWith({ resolveBaseSize: () => size });

  assert.equal(cache.placement(9).start, 180);
  size = 30;
  assert.equal(cache.placement(9).start, 180, "cached offsets survive until invalidation");
  cache.invalidateFrom(5);
  assert.equal(cache.placement(4).start, 80, "offsets before the index are kept");
  assert.equal(cache.placement(9).start, 5 * 20 + 4 * 30);
  cache.invalidateFrom(0);
  assert.equal(cache.placement(9).start, 270);
});

test("clears measurements from an index onward", () => {
  const cache = cacheWith();
  cache.setMeasured(1, 40);
  cache.setMeasured(6, 40);

  cache.clearMeasuredFrom(5);
  assert.equal(cache.measuredSize(1), 40);
  assert.equal(cache.measuredSize(6), undefined);
  assert.equal(cache.contentEnd(), 10 * 20 + 20);
});

test("shifts measurements after a prepend", () => {
  const cache = cacheWith();
  cache.setMeasured(2, 50);

  cache.shiftMeasurements(3);
  assert.equal(cache.measuredSize(2), undefined);
  assert.equal(cache.measuredSize(5), 50);
  assert.equal(cache.placement(5).size, 50);
});

test("rejects invalid indexes and sizes", () => {
  const cache = cacheWith();

  assert.throws(() => cache.placement(-1), /VIZE_UI_VIRTUALIZER_OPTION/);
  assert.throws(() => cache.placement(10), /outside the collection/);
  assert.throws(() => cache.setMeasured(0, -1), /finite non-negative/);
  assert.throws(() => cache.setMeasured(0, Number.NaN), /finite non-negative/);

  const broken = cacheWith({ resolveBaseSize: () => Number.NaN });
  assert.throws(() => broken.placement(0), /the item size/);
});
