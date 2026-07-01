import assert from "node:assert/strict";
import test from "node:test";

import { cssLengthToPx } from "./css-length.js";

void test("cssLengthToPx converts common CSS length units for previews", () => {
  assert.equal(cssLengthToPx(12), 12);
  assert.equal(cssLengthToPx("12px"), 12);
  assert.equal(cssLengthToPx("1.5rem"), 24);
  assert.equal(cssLengthToPx("4em"), 64);
  assert.equal(cssLengthToPx("var(--space)"), null);
});
