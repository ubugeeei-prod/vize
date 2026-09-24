import assert from "node:assert/strict";
import { test } from "node:test";
import { nextTick, ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useNumberFormat } from "./use-number-format.ts";
import type { UseNumberFormatOptions } from "./use-number-format.ts";

void test("formats the reactive value with the explicit locale", () => {
  const value = ref<number | null>(1234.5);
  const number = useNumberFormat(value, { locale: "en-US" });

  assert.equal(number.locale.value, "en-US");
  assert.equal(number.formatted.value, "1,234.5");
  value.value = null;
  assert.equal(number.formatted.value, "");
});

void test("follows reactive options including the locale", async () => {
  const options = ref<UseNumberFormatOptions>({
    locale: "en-US",
    style: "currency",
    currency: "USD",
  });
  const number = useNumberFormat(() => 42, options);
  assert.equal(number.formatted.value, "$42.00");

  options.value = { locale: "de-DE", style: "currency", currency: "EUR" };
  await nextTick();
  assert.equal(number.formatted.value, "42,00 €");
  assert.equal(number.locale.value, "de-DE");
});

void test("formats bigint values, parts, and ranges", () => {
  const number = useNumberFormat(undefined, { locale: "en-US" });

  assert.equal(number.formatted.value, "");
  assert.equal(number.format(12345678901234567890n), "12,345,678,901,234,567,890");
  assert.deepEqual(
    number.formatToParts(1000).map((part) => part.type),
    ["integer", "group", "integer"],
  );
  assert.equal(number.formatRange(3, 5), "3–5");
});

void test("reuses the cached formatter for equivalent options", () => {
  const first = useNumberFormat(1, { locale: "en-US", maximumFractionDigits: 1 });
  const read = first.formatter.value;
  assert.equal(first.formatter.value, read);
});

void test("surfaces invalid options as the platform RangeError", () => {
  const number = useNumberFormat(1, { locale: "en-US", style: "currency" });
  assert.throws(() => number.formatted.value, TypeError);
  const invalid = useNumberFormat(1, { locale: "en-US", maximumFractionDigits: 200 });
  assert.throws(() => invalid.formatted.value, RangeError);
});

void test("server rendering is deterministic for explicit locales", async () => {
  const state = await renderComposableOnServer(() => ({
    us: useNumberFormat(9876.5, { locale: "en-US" }).formatted,
    ja: useNumberFormat(9876.5, { locale: "ja-JP", style: "currency", currency: "JPY" }).formatted,
    fallback: useNumberFormat(1000).locale,
  }));
  assert.equal(state, '{"us":"9,876.5","ja":"￥9,877","fallback":"en"}');
});
