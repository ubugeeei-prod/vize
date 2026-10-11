import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { getProgressBarState, getProgressBarStyle } from "./progress-bar-state.ts";

test("preserves ordinary progress ranges, repairs, and indeterminate state", () => {
  assert.deepEqual(getProgressBarState({ value: 60 }), {
    value: 60,
    min: 0,
    max: 100,
    percent: 60,
    ratio: 0.6,
    dir: "ltr",
    indeterminate: false,
    complete: false,
    invalid: false,
    state: "loading",
  });
  const repaired = getProgressBarState({ min: 2, max: 2, value: 2 });
  assert.equal(repaired.min, 2);
  assert.equal(repaired.max, 102);
  assert.equal(repaired.percent, 0);
  assert.equal(repaired.invalid, true);
  assert.equal(repaired.complete, false);
  const advancing = getProgressBarState({ min: -(2 ** 60), max: -(2 ** 60), value: -(2 ** 60) });
  assert.equal(advancing.max, -(2 ** 60) + 128);
  assert.equal(advancing.percent, 0);
  assert.equal(advancing.complete, false);
  const indeterminate = getProgressBarState({ value: Number.NaN });
  assert.equal(indeterminate.value, null);
  assert.equal(indeterminate.percent, null);
  assert.equal(indeterminate.ratio, null);
  assert.equal(indeterminate.state, "indeterminate");
  assert.equal(getProgressBarStyle(indeterminate)["--vize-ui-progress-bar-percent"], "0%");
});

test("keeps signed finite ranges at zero, fifty, and one hundred percent", () => {
  for (const [value, percent, ratio] of [
    [-Number.MAX_VALUE, 0, 0],
    [0, 50, 0.5],
    [Number.MAX_VALUE, 100, 1],
  ] as const) {
    const state = getProgressBarState({ min: -Number.MAX_VALUE, max: Number.MAX_VALUE, value });
    assert.equal(state.min, -Number.MAX_VALUE);
    assert.equal(state.max, Number.MAX_VALUE);
    assert.equal(state.value, value);
    assert.equal(state.percent, percent);
    assert.equal(state.ratio, ratio);
    assert.equal(state.complete, value === Number.MAX_VALUE);
    assert.equal(state.invalid, false);
    assert.equal(getProgressBarStyle(state)["--vize-ui-progress-bar-percent"], `${percent}%`);
    assert.equal(getProgressBarStyle(state)["--vize-ui-progress-bar-ratio"], String(ratio));
  }
  const ordinary = getProgressBarState({ max: Number.MAX_VALUE, value: Number.MAX_VALUE });
  assert.equal(ordinary.min, 0);
  assert.equal(ordinary.percent, 100);
  assert.equal(ordinary.ratio, 1);
});

test("reports finite invalid percentages when the default repair cannot advance the minimum", () => {
  for (const min of [
    Number.MAX_VALUE,
    -Number.MAX_VALUE,
    Number.MAX_VALUE / 2,
    -Number.MAX_VALUE / 2,
    2 ** 60,
    -(2 ** 61),
  ]) {
    const state = getProgressBarState({ min, max: min, value: min });
    assert.equal(state.min, min);
    assert.equal(state.max, min);
    assert.equal(state.value, min);
    assert.equal(state.percent, 0);
    assert.equal(state.ratio, 0);
    assert.equal(state.invalid, true);
    assert.equal(state.complete, true);
    assert.equal(state.state, "complete");
    assert.equal(getProgressBarStyle(state)["--vize-ui-progress-bar-percent"], "0%");
    assert.equal(getProgressBarStyle(state)["--vize-ui-progress-bar-ratio"], "0");
    const indeterminate = getProgressBarState({ min });
    assert.equal(indeterminate.value, null);
    assert.equal(indeterminate.percent, null);
    assert.equal(indeterminate.ratio, null);
    assert.equal(indeterminate.invalid, true);
    assert.equal(indeterminate.state, "indeterminate");
  }
});

test("publishes finite bounded progress and CSS for extreme and invalid inputs", () => {
  for (const min of [
    -Number.MAX_VALUE,
    -1e308,
    -1e20,
    -100,
    0,
    100,
    1e20,
    1e308,
    Number.MAX_VALUE,
  ]) {
    for (const max of [min, -Number.MAX_VALUE, 0, 100, Number.MAX_VALUE, Number.NaN]) {
      for (const value of [null, Number.NaN, -Number.MAX_VALUE, 0, Number.MAX_VALUE]) {
        const state = getProgressBarState({ min, max, value });
        assert.ok(Number.isFinite(state.min));
        assert.ok(Number.isFinite(state.max));
        assert.ok(state.min <= state.max);
        if (state.value === null) {
          assert.equal(state.percent, null);
          assert.equal(state.ratio, null);
        } else {
          assert.ok(state.value >= state.min && state.value <= state.max);
          assert.ok(Number.isFinite(state.percent));
          assert.ok(Number.isFinite(state.ratio));
          assert.ok(state.percent !== null && state.percent >= 0 && state.percent <= 100);
          assert.ok(state.ratio !== null && state.ratio >= 0 && state.ratio <= 1);
        }
        for (const token of Object.values(getProgressBarStyle(state))) {
          assert.doesNotMatch(token, /NaN|Infinity/);
        }
      }
    }
  }
});
