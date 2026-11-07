import assert from "node:assert/strict";
import { test } from "vitest";
import { normalizeKeyEventType, normalizeMouseEvent } from "./nativeInput.js";

test("normalizes native key event phases at the public protocol boundary", () => {
  assert.equal(normalizeKeyEventType("press"), "press");
  assert.equal(normalizeKeyEventType("repeat"), "repeat");
  assert.equal(normalizeKeyEventType("release"), "release");
  assert.equal(normalizeKeyEventType("future-phase"), undefined);
  assert.equal(normalizeKeyEventType(undefined), undefined);
});

test("preserves native mouse actions and modifiers at the public protocol boundary", () => {
  assert.deepEqual(
    normalizeMouseEvent({
      eventType: "mouse",
      mouseKind: "scroll-down",
      x: 7,
      y: 11,
      modifiers: {
        ctrl: false,
        alt: false,
        shift: true,
        meta: false,
        super: false,
        hyper: false,
        capsLock: false,
        numLock: false,
      },
    }),
    {
      type: "mouse",
      kind: "scroll-down",
      button: undefined,
      x: 7,
      y: 11,
      modifiers: {
        ctrl: false,
        alt: false,
        shift: true,
        meta: false,
        super: false,
        hyper: false,
        capsLock: false,
        numLock: false,
      },
    },
  );
});
