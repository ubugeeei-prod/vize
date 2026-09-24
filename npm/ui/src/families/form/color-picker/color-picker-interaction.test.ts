import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { getColorChannelRange } from "./color-picker-color.ts";
import {
  capturePointer,
  fractionToValue,
  pointerFraction,
  releasePointer,
  resolveKeyIntent,
  valueToPercent,
} from "./color-picker-interaction.ts";
import {
  createEyeDropper,
  isEyeDropperAbort,
  isEyeDropperSupported,
  readEyeDropperResult,
} from "./color-picker-eye-dropper.ts";
import { getSwatchIdSegment } from "./color-picker-swatch-context.ts";

const rect = { left: 10, top: 20, width: 100, height: 50 };
const hue = getColorChannelRange("hue");
const saturation = getColorChannelRange("saturation");
const brightness = getColorChannelRange("brightness");

test("maps pointer coordinates to clamped fractions in both directions", () => {
  assert.deepEqual(pointerFraction(rect, 60, 45), { x: 0.5, y: 0.5 });
  assert.deepEqual(pointerFraction(rect, -100, 500), { x: 0, y: 0 });
  assert.deepEqual(pointerFraction(rect, 500, -100), { x: 1, y: 1 });
  assert.deepEqual(pointerFraction(rect, 35, 20, "rtl"), { x: 0.75, y: 1 });
  assert.deepEqual(pointerFraction({ left: 0, top: 0, width: 0, height: 0 }, 5, 5), {
    x: 0,
    y: 1,
  });
});

test("maps fractions and values across channel ranges", () => {
  assert.equal(fractionToValue(hue, 0.5), 180);
  assert.equal(fractionToValue(hue, 2), 360);
  assert.equal(fractionToValue(hue, Number.NaN), 0);
  assert.equal(valueToPercent(hue, 90), 25);
  assert.equal(valueToPercent(getColorChannelRange("alpha"), 0.333), 33.3);
  assert.equal(valueToPercent({ min: 5, max: 5, step: 1, pageStep: 1 }, 5), 0);
});

test("resolves one- and two-dimensional keyboard intents", () => {
  const twoD = { dimensions: 2, xRange: saturation, yRange: brightness } as const;
  assert.deepEqual(resolveKeyIntent({ key: "ArrowRight", shiftKey: false }, twoD), {
    kind: "delta",
    axis: "x",
    amount: 1,
  });
  assert.deepEqual(resolveKeyIntent({ key: "ArrowDown", shiftKey: true }, twoD), {
    kind: "delta",
    axis: "y",
    amount: -10,
  });
  assert.deepEqual(resolveKeyIntent({ key: "PageUp", shiftKey: false }, twoD), {
    kind: "delta",
    axis: "y",
    amount: 10,
  });
  assert.deepEqual(resolveKeyIntent({ key: "End", shiftKey: false }, twoD), {
    kind: "edge",
    axis: "x",
    edge: "max",
  });
  assert.deepEqual(
    resolveKeyIntent({ key: "ArrowLeft", shiftKey: false }, { ...twoD, dir: "rtl" }),
    { kind: "delta", axis: "x", amount: 1 },
  );
  const vertical = { dimensions: 1, orientation: "vertical", xRange: hue } as const;
  assert.deepEqual(resolveKeyIntent({ key: "ArrowRight", shiftKey: false }, vertical), {
    kind: "delta",
    axis: "y",
    amount: 1,
  });
  assert.deepEqual(resolveKeyIntent({ key: "PageDown", shiftKey: false }, vertical), {
    kind: "delta",
    axis: "y",
    amount: -15,
  });
  assert.deepEqual(resolveKeyIntent({ key: "Home", shiftKey: false }, vertical), {
    kind: "edge",
    axis: "y",
    edge: "min",
  });
  assert.equal(resolveKeyIntent({ key: "Tab", shiftKey: false }, vertical), null);
});

test("pointer capture helpers tolerate missing or throwing platform APIs", () => {
  const calls: string[] = [];
  const element = document.createElement("div");
  element.setPointerCapture = () => {
    calls.push("set");
    throw new Error("not active");
  };
  element.hasPointerCapture = () => true;
  element.releasePointerCapture = () => calls.push("release");
  capturePointer(element, 1);
  releasePointer(element, 1);
  assert.deepEqual(calls, ["set", "release"]);
  const bare = document.createElement("div");
  Reflect.set(bare, "setPointerCapture", undefined);
  Reflect.set(bare, "releasePointerCapture", undefined);
  capturePointer(bare, 1);
  releasePointer(bare, 1);
});

test("eye dropper helpers feature-detect and validate untyped platform values", () => {
  assert.equal(isEyeDropperSupported({}), false);
  assert.equal(createEyeDropper({}), null);
  class Dropper {
    open(): Promise<unknown> {
      return Promise.resolve({ sRGBHex: "#fff" });
    }
  }
  class Broken {}
  assert.equal(isEyeDropperSupported({ EyeDropper: Dropper }), true);
  assert.ok(createEyeDropper({ EyeDropper: Dropper }));
  assert.equal(createEyeDropper({ EyeDropper: Broken }), null);
  assert.equal(readEyeDropperResult({ sRGBHex: "#abcdef" }), "#abcdef");
  assert.equal(readEyeDropperResult({ sRGBHex: 1 }), null);
  assert.equal(readEyeDropperResult(null), null);
  assert.equal(isEyeDropperAbort(Object.assign(new Error("x"), { name: "AbortError" })), true);
  assert.equal(isEyeDropperAbort(new Error("x")), false);
  assert.equal(isEyeDropperAbort("AbortError"), false);
});

test("creates DOM-id-safe swatch segments for arbitrary color strings", () => {
  assert.equal(getSwatchIdSegment("red-1"), "swatch-red-1");
  assert.match(getSwatchIdSegment("#ff0000"), /^swatch-ff0000-[a-z0-9]+$/);
  assert.match(getSwatchIdSegment("  "), /^swatch-empty-[a-z0-9]+$/);
  assert.notEqual(getSwatchIdSegment("#f00"), getSwatchIdSegment("f00!"));
});
