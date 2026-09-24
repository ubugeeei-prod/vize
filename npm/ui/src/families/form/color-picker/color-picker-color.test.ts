import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  DEFAULT_COLOR,
  colorEquals,
  createColor,
  formatColor,
  formatColorChannelValue,
  fromHsla,
  fromRgba,
  getColorChannelGradient,
  getColorChannelLabel,
  getColorChannelRange,
  getColorChannelValue,
  isColorFormat,
  isColorValue,
  parseColor,
  resolveColorChannelSpace,
  setColorChannelValue,
  snapColorChannelValue,
  toHsla,
  toRgba,
} from "./color-picker-color.ts";
import type { ColorValue } from "./color-picker-color.ts";

function hex(input: string): string | null {
  const color = parseColor(input);
  return color === null ? null : formatColor(color, "hex");
}

test("parses every hex notation and normalizes to lowercase", () => {
  assert.equal(hex("#F00"), "#ff0000");
  assert.equal(hex("#f008"), "#ff000088");
  assert.equal(hex("#00FF00"), "#00ff00");
  assert.equal(hex("#0000ff80"), "#0000ff80");
  assert.equal(hex("  #abc  "), "#aabbcc");
  for (const invalid of ["#", "#ff", "#fffff", "#ggg", "fff", "#1234567", "#123456789"]) {
    assert.equal(parseColor(invalid), null, invalid);
  }
});

test("parses legacy and modern rgb syntax with percentages and alpha", () => {
  assert.equal(hex("rgb(255, 0, 0)"), "#ff0000");
  assert.equal(hex("rgba(255, 0, 0, 0.5)"), "#ff000080");
  assert.equal(hex("rgb(255 128 0)"), "#ff8000");
  assert.equal(hex("rgb(100% 50% 0% / 50%)"), "#ff800080");
  assert.equal(hex("RGB(0 0 255 / .25)"), "#0000ff40");
  assert.equal(hex("rgb(300 -20 0)"), "#ff0000", "channels clamp like CSS");
  assert.equal(hex("rgb(0 0 0 / 2)"), "#000000", "alpha clamps to 1");
  for (const invalid of [
    "rgb()",
    "rgb(1, 2)",
    "rgb(1 2 3 4)",
    "rgb(1, 2, 3, 4, 5)",
    "rgb(1 2 / 3 / 4)",
    "rgb(a b c)",
    "rgb(1 2 3 /)",
    "rgb(1,2 3)",
    "cmyk(1 2 3)",
  ]) {
    assert.equal(parseColor(invalid), null, invalid);
  }
});

test("parses hsl with angle units and hsb notation", () => {
  assert.equal(hex("hsl(0, 100%, 50%)"), "#ff0000");
  assert.equal(hex("hsla(120, 100%, 50%, 0.5)"), "#00ff0080");
  assert.equal(hex("hsl(240deg 100% 50%)"), "#0000ff");
  assert.equal(hex("hsl(0.5turn 100% 50%)"), "#00ffff");
  assert.equal(hex(`hsl(${Math.PI}rad 100% 50%)`), "#00ffff");
  assert.equal(hex("hsl(200grad 100% 50%)"), "#00ffff");
  assert.equal(hex("hsl(-120 100% 50%)"), "#0000ff", "negative hue wraps");
  assert.equal(hex("hsl(0 0% 100%)"), "#ffffff");
  assert.equal(hex("hsb(0 100% 100%)"), "#ff0000");
  assert.equal(hex("hsba(120, 100%, 50%, 1)"), "#008000");
  assert.equal(parseColor("hsl(0 100% 50% / x)"), null);
  assert.equal(parseColor("hsl(10px 100% 50%)"), null);
});

test("parses transparent and rejects empty or unknown input", () => {
  assert.equal(formatColor(parseColor("transparent") ?? DEFAULT_COLOR, "hex8"), "#00000000");
  assert.equal(parseColor(""), null);
  assert.equal(parseColor("   "), null);
  assert.equal(parseColor("red"), null);
});

test("formats every color format with and without alpha", () => {
  const opaque = parseColor("#3366cc") ?? DEFAULT_COLOR;
  const translucent = { ...opaque, alpha: 0.5 };
  assert.equal(formatColor(opaque, "hex"), "#3366cc");
  assert.equal(formatColor(opaque, "hex8"), "#3366ccff");
  assert.equal(formatColor(opaque, "rgb"), "rgb(51 102 204)");
  assert.equal(formatColor(opaque, "hsl"), "hsl(220 60% 50%)");
  assert.equal(formatColor(opaque, "hsb"), "hsb(220 75% 80%)");
  assert.equal(formatColor(translucent, "hex"), "#3366cc80");
  assert.equal(formatColor(translucent, "rgb"), "rgb(51 102 204 / 0.5)");
  assert.equal(formatColor(translucent, "hsl"), "hsl(220 60% 50% / 0.5)");
  assert.equal(formatColor(translucent, "hsb"), "hsb(220 75% 80% / 0.5)");
  assert.equal(formatColor(opaque), "#3366cc", "hex is the default format");
});

test("round-trips every 8-bit primary and secondary through each format", () => {
  const samples = ["#000000", "#ffffff", "#ff0000", "#00ff00", "#0000ff", "#ffff00", "#808080"];
  for (const sample of samples) {
    const color = parseColor(sample);
    assert.ok(color, sample);
    for (const format of ["hex", "hex8", "rgb", "hsb"] as const) {
      const reparsed = parseColor(formatColor(color, format));
      assert.ok(reparsed, `${sample} ${format}`);
      assert.equal(formatColor(reparsed, "hex"), sample, `${sample} via ${format}`);
    }
  }
  for (let value = 0; value < 256; value += 17) {
    const sample = `#${value.toString(16).padStart(2, "0").repeat(3)}`;
    assert.equal(hex(formatColor(parseColor(sample) ?? DEFAULT_COLOR, "rgb")), sample);
  }
});

test("converts between hsb, rgb, and hsl exactly", () => {
  const color = createColor({ hue: 210, saturation: 50, brightness: 80, alpha: 0.4 });
  const rgb = toRgba(color);
  assert.deepEqual(
    [Math.round(rgb.red), Math.round(rgb.green), Math.round(rgb.blue), rgb.alpha],
    [102, 153, 204, 0.4],
  );
  const hsl = toHsla(color);
  assert.equal(Math.round(hsl.hue), 210);
  assert.equal(Math.round(hsl.saturation), 50);
  assert.equal(Math.round(hsl.lightness), 60);
  const back = fromHsla(hsl);
  assert.ok(Math.abs(back.saturation - 50) < 1e-9);
  assert.ok(Math.abs(back.brightness - 80) < 1e-9);
  const fromRgb = fromRgba(rgb);
  assert.ok(Math.abs(fromRgb.hue - 210) < 1e-9);
});

test("achromatic conversions keep the fallback hue and saturation", () => {
  const blue = createColor({ hue: 240, saturation: 70, brightness: 90 });
  const grey = fromRgba({ red: 128, green: 128, blue: 128, alpha: 1 }, blue);
  assert.equal(grey.hue, 240);
  assert.equal(grey.saturation, 0);
  const black = fromRgba({ red: 0, green: 0, blue: 0, alpha: 1 }, blue);
  assert.equal(black.hue, 240);
  assert.equal(black.saturation, 70);
  const hslBlack = fromHsla({ hue: 240, saturation: 100, lightness: 0, alpha: 1 }, blue);
  assert.equal(hslBlack.saturation, 70);
  assert.equal(toHsla(createColor({ hue: 0, saturation: 0, brightness: 100 })).saturation, 0);
});

test("createColor wraps hue and clamps every channel", () => {
  assert.deepEqual(createColor({ hue: 370, saturation: 120, brightness: -5, alpha: 3 }), {
    hue: 10,
    saturation: 100,
    brightness: 0,
    alpha: 1,
  });
  assert.deepEqual(createColor({ hue: -30, saturation: Number.NaN, brightness: 50 }), {
    hue: 330,
    saturation: 0,
    brightness: 50,
    alpha: 1,
  });
  assert.equal(createColor({ hue: Number.POSITIVE_INFINITY, saturation: 0, brightness: 0 }).hue, 0);
  assert.ok(Object.isFrozen(createColor({ hue: 0, saturation: 0, brightness: 0 })));
});

test("reads and writes every channel in its color space", () => {
  const color = parseColor("#3366cc") ?? DEFAULT_COLOR;
  assert.equal(Math.round(getColorChannelValue(color, "hue")), 220);
  assert.equal(Math.round(getColorChannelValue(color, "saturation")), 75);
  assert.equal(Math.round(getColorChannelValue(color, "saturation", "hsl")), 60);
  assert.equal(Math.round(getColorChannelValue(color, "lightness", "hsl")), 50);
  assert.equal(Math.round(getColorChannelValue(color, "red", "rgb")), 51);
  assert.equal(Math.round(getColorChannelValue(color, "green", "rgb")), 102);
  assert.equal(Math.round(getColorChannelValue(color, "blue", "rgb")), 204);
  assert.equal(getColorChannelValue(color, "alpha"), 1);

  assert.equal(formatColor(setColorChannelValue(color, "red", 255, "rgb")), "#ff66cc");
  assert.equal(formatColor(setColorChannelValue(color, "green", 0, "rgb")), "#3300cc");
  assert.equal(formatColor(setColorChannelValue(color, "blue", 0, "rgb")), "#336600");
  assert.equal(formatColor(setColorChannelValue(color, "hue", 0)), "#cc3333");
  assert.equal(formatColor(setColorChannelValue(color, "hue", 360)), "#cc3333");
  assert.equal(snapColorChannelValue("hue", setColorChannelValue(color, "hue", 360).hue), 360);
  assert.equal(formatColor(setColorChannelValue(color, "brightness", 100)), "#4080ff");
  assert.equal(formatColor(setColorChannelValue(color, "saturation", 0)), "#cccccc");
  assert.equal(formatColor(setColorChannelValue(color, "lightness", 100, "hsl")), "#ffffff");
  assert.equal(formatColor(setColorChannelValue(color, "saturation", 100, "hsl")), "#0055ff");
  assert.equal(formatColor(setColorChannelValue(color, "alpha", 0.5)), "#3366cc80");
  assert.equal(setColorChannelValue(color, "alpha", 5).alpha, 1, "values clamp to range");
  assert.equal(setColorChannelValue(color, "alpha", Number.NaN).alpha, 0);
});

test("resolves channel spaces, ranges, labels, snapping, and value text", () => {
  assert.equal(resolveColorChannelSpace("lightness", "hsb"), "hsl");
  assert.equal(resolveColorChannelSpace("brightness", "hsl"), "hsb");
  assert.equal(resolveColorChannelSpace("red"), "rgb");
  assert.equal(resolveColorChannelSpace("saturation", "rgb"), "hsb");
  assert.equal(resolveColorChannelSpace("hue", "hsl"), "hsl");
  assert.equal(resolveColorChannelSpace("alpha"), "hsb");
  assert.deepEqual(getColorChannelRange("hue"), { min: 0, max: 360, step: 1, pageStep: 15 });
  assert.deepEqual(getColorChannelRange("red"), { min: 0, max: 255, step: 1, pageStep: 16 });
  assert.deepEqual(getColorChannelRange("alpha"), { min: 0, max: 1, step: 0.01, pageStep: 0.1 });
  assert.equal(getColorChannelLabel("lightness"), "Lightness");
  assert.equal(snapColorChannelValue("alpha", 0.456), 0.46);
  assert.equal(snapColorChannelValue("hue", 12.6), 13);
  assert.equal(snapColorChannelValue("hue", 12.6, 5), 15);
  assert.equal(snapColorChannelValue("saturation", 140), 100);
  assert.equal(snapColorChannelValue("red", 10, Number.NaN), 10);
  assert.equal(formatColorChannelValue("hue", 209.6), "Hue 210°");
  assert.equal(formatColorChannelValue("alpha", 0.25), "Alpha 25%");
  assert.equal(formatColorChannelValue("green", 12.2), "Green 12");
  assert.equal(formatColorChannelValue("brightness", 40), "Brightness 40%");
});

test("builds exact channel preview gradients", () => {
  const color = parseColor("#3366cc") ?? DEFAULT_COLOR;
  assert.equal(
    getColorChannelGradient(color, "hue"),
    "linear-gradient(to right, rgb(255 0 0), rgb(255 255 0), rgb(0 255 0), rgb(0 255 255), rgb(0 0 255), rgb(255 0 255), rgb(255 0 0))",
  );
  assert.equal(
    getColorChannelGradient(color, "alpha", "hsb", "to top"),
    "linear-gradient(to top, rgb(51 102 204 / 0), rgb(51 102 204))",
  );
  assert.equal(
    getColorChannelGradient(color, "red", "rgb"),
    "linear-gradient(to right, rgb(0 102 204), rgb(255 102 204))",
  );
  assert.equal(
    getColorChannelGradient(color, "lightness", "hsl"),
    "linear-gradient(to right, rgb(0 0 0), rgb(51 102 204), rgb(255 255 255))",
  );
});

test("compares colors at 8-bit precision and guards unknown values", () => {
  const left: ColorValue = createColor({ hue: 0, saturation: 100, brightness: 100 });
  const right: ColorValue = createColor({ hue: 0.01, saturation: 100, brightness: 100 });
  assert.equal(colorEquals(left, right), true);
  assert.equal(colorEquals(left, { ...left, alpha: 0.5 }), false);
  assert.equal(isColorValue(left), true);
  assert.equal(isColorValue({ hue: 0, saturation: 0, brightness: 0 }), false);
  assert.equal(isColorValue({ hue: 0, saturation: 0, brightness: Number.NaN, alpha: 1 }), false);
  assert.equal(isColorValue(null), false);
  assert.equal(isColorFormat("hsl"), true);
  assert.equal(isColorFormat("cmyk"), false);
  assert.equal(isColorFormat(1), false);
});
