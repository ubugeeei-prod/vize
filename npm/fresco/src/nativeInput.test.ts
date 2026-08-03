import assert from "node:assert/strict";
import { test } from "node:test";
import { normalizeKeyEventType } from "./nativeInput.js";

void test("normalizes native key event phases at the public protocol boundary", () => {
  assert.equal(normalizeKeyEventType("press"), "press");
  assert.equal(normalizeKeyEventType("repeat"), "repeat");
  assert.equal(normalizeKeyEventType("release"), "release");
  assert.equal(normalizeKeyEventType("future-phase"), undefined);
  assert.equal(normalizeKeyEventType(undefined), undefined);
});
