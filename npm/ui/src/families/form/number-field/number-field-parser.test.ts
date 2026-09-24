import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createNumberFieldParser } from "./number-field-parser.ts";
import {
  canStepNumberFieldValue,
  getNumberFieldState,
  normalizeNumberFieldBounds,
  snapNumberFieldValue,
  stepNumberFieldValue,
} from "./number-field-state.ts";

test("round-trips formatted text across locales, numbering systems, and styles", () => {
  const cases: readonly [string, Intl.NumberFormatOptions, number][] = [
    ["en-US", {}, -12_345.678],
    ["de-DE", { style: "currency", currency: "EUR" }, -12_345.68],
    ["fr-FR", {}, 12_345.5],
    ["ar-EG", {}, -1234.5],
    ["hi-IN-u-nu-deva", {}, 98_765.4],
    ["en-US", { style: "percent", maximumFractionDigits: 2 }, 0.1234],
    ["en-US", { style: "unit", unit: "kilometer-per-hour" }, 88.5],
    ["ja-JP", { style: "currency", currency: "JPY" }, -1200],
    ["en-US", { style: "currency", currency: "USD", currencySign: "accounting" }, -42.5],
    ["de-CH", {}, 1_234_567.25],
  ];
  for (const [locale, options, value] of cases) {
    const parser = createNumberFieldParser(locale, options);
    const text = parser.format(value);
    assert.equal(parser.parse(text), value, `${locale} ${JSON.stringify(options)} ${text}`);
    assert.equal(parser.isPartial(text), true, `${locale} partial ${text}`);
  }
});

test("parses plain ASCII input and rejects non-numbers", () => {
  const german = createNumberFieldParser("de-DE");
  assert.equal(german.parse("1234,5"), 1234.5);
  assert.equal(german.parse("  -7 "), -7);
  assert.equal(german.parse("−7"), -7, "Unicode minus sign");
  assert.equal(german.parse(""), null);
  assert.ok(Number.isNaN(german.parse("12a") ?? 0));
  assert.ok(Number.isNaN(german.parse("1,2,3") ?? 0));
  assert.ok(Number.isNaN(german.parse("-") ?? 0));

  const arabic = createNumberFieldParser("ar-EG");
  assert.equal(arabic.parse("١٢٣"), 123, "native digits");
  assert.equal(arabic.parse("123"), 123, "ASCII digits are always accepted");

  const percent = createNumberFieldParser("en-US", { style: "percent" });
  assert.equal(percent.parse("45"), 0.45);
  assert.equal(percent.parse("12.5%"), 0.125);
});

test("accepts partial prefixes while typing and honors sign and fraction limits", () => {
  const parser = createNumberFieldParser("en-US");
  for (const text of ["", "-", "+", ".", "1.", "-0.", "1,2"]) {
    assert.equal(parser.isPartial(text), true, text);
  }
  assert.equal(parser.isPartial("1e5"), false);
  assert.equal(parser.isPartial("--1"), false);
  assert.equal(parser.isPartial("-1", { allowNegative: false }), false);

  const yen = createNumberFieldParser("ja-JP", { style: "currency", currency: "JPY" });
  assert.equal(yen.isPartial("12.5"), false);
  assert.ok(Number.isNaN(yen.parse("12.5") ?? 0));
});

test("caches parsers per locale and option set", () => {
  const first = createNumberFieldParser("en-GB", { style: "currency", currency: "GBP" });
  const second = createNumberFieldParser("en-GB", { currency: "GBP", style: "currency" });
  assert.ok(first === second);
  assert.equal(first.locale, "en-GB");
  assert.equal(first.format(Number.NaN), "");
});

test("normalizes bounds, snaps, and steps along the grid", () => {
  const bounds = normalizeNumberFieldBounds({ min: 0, max: 50, step: 5 });
  assert.deepEqual(bounds, { min: 0, max: 50, step: 5, largeStep: 50 });
  assert.deepEqual(normalizeNumberFieldBounds({ min: 10, max: 5, step: -1, largeStep: 0 }), {
    min: 10,
    max: Number.POSITIVE_INFINITY,
    step: 1,
    largeStep: 10,
  });
  assert.equal(normalizeNumberFieldBounds({ step: 0.1 }).largeStep, 1);
  assert.equal(snapNumberFieldValue(12, bounds), 10);
  assert.equal(snapNumberFieldValue(13, bounds), 15);
  assert.equal(snapNumberFieldValue(80, bounds), 50);
  assert.equal(stepNumberFieldValue(7, 1, bounds), 10);
  assert.equal(stepNumberFieldValue(7, -1, bounds), 5);
  assert.equal(stepNumberFieldValue(10, 1, bounds, 20), 30);
  assert.equal(stepNumberFieldValue(48, 1, bounds, 50), 50);
  assert.equal(stepNumberFieldValue(null, 1, bounds), 0);
  assert.equal(stepNumberFieldValue(null, -1, bounds), 50);
  assert.equal(stepNumberFieldValue(null, 1, normalizeNumberFieldBounds({ min: -5 })), -5);
  assert.equal(stepNumberFieldValue(null, 1, normalizeNumberFieldBounds()), 0);
  const decimals = normalizeNumberFieldBounds({ step: 0.1 });
  assert.equal(stepNumberFieldValue(0.2, 1, decimals), 0.3);
  assert.equal(stepNumberFieldValue(-0.1, 1, decimals), 0);
  assert.equal(canStepNumberFieldValue(50, 1, bounds), false);
  assert.equal(canStepNumberFieldValue(0, -1, bounds), false);
  assert.equal(canStepNumberFieldValue(null, -1, bounds), true);
});

test("derives the published state token", () => {
  const bounds = normalizeNumberFieldBounds({ min: 0, max: 10 });
  const base = { bounds, disabled: false, readOnly: false, invalid: false };
  assert.equal(getNumberFieldState({ ...base, value: null }), "empty");
  assert.equal(getNumberFieldState({ ...base, value: 0 }), "min");
  assert.equal(getNumberFieldState({ ...base, value: 10 }), "max");
  assert.equal(getNumberFieldState({ ...base, value: 5 }), "in-range");
  assert.equal(getNumberFieldState({ ...base, value: 5, invalid: true }), "invalid");
  assert.equal(getNumberFieldState({ ...base, value: 5, readOnly: true }), "readonly");
  assert.equal(getNumberFieldState({ ...base, value: 5, disabled: true }), "disabled");
});
