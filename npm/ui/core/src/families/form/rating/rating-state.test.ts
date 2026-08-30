import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { getRatingState } from "./rating-state.ts";

test("normalizes finite integer bounds, value, direction, and state", () => {
  assert.deepEqual(
    getRatingState({
      value: 3.7,
      min: 0,
      max: 5,
      count: 10,
      direction: "rtl",
      required: true,
      clearable: true,
    }),
    {
      value: 4,
      min: 0,
      max: 5,
      count: 6,
      items: [0, 1, 2, 3, 4, 5],
      percent: (5 / 6) * 100,
      direction: "rtl",
      disabled: false,
      readOnly: false,
      required: true,
      invalid: false,
      clearable: true,
      state: "selected",
    },
  );
  assert.deepEqual(
    getRatingState({
      value: Number.NaN,
      min: Number.NEGATIVE_INFINITY,
      max: -1,
      count: 0,
      disabled: true,
    }),
    {
      value: null,
      min: 1,
      max: 1,
      count: 1,
      items: [1],
      percent: 0,
      direction: "ltr",
      disabled: true,
      readOnly: false,
      required: false,
      invalid: false,
      clearable: false,
      state: "disabled",
    },
  );
});
