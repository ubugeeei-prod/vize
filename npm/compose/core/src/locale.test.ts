import assert from "node:assert/strict";
import { test } from "node:test";
import { ref } from "vue";

import { useLocale } from "./locale.ts";

void test("canonicalizes reactive locales and reads native text direction", () => {
  const source = ref<Intl.Locale | string>("EN-us");
  const locale = useLocale(source);

  assert.equal(locale.locale.value, "en-US");
  assert.equal(locale.direction.value, "ltr");
  source.value = new Intl.Locale("ar-EG");
  assert.equal(locale.direction.value, "rtl");
});

void test("uses an injected detector before the documented fallback", () => {
  assert.equal(useLocale(undefined, { detect: () => "fr-FR" }).locale.value, "fr-FR");
  assert.equal(
    useLocale(undefined, { detect: () => undefined, fallback: "ja-JP" }).locale.value,
    "ja-JP",
  );
});

void test("reuses equivalent formatter options until the locale changes", () => {
  const source = ref("en-US");
  const locale = useLocale(source);
  const first = locale.number({ style: "unit", unit: "meter" });
  const equivalent = locale.number({ unit: "meter", style: "unit" });

  assert.equal(first, equivalent);
  source.value = "de-DE";
  assert.notEqual(locale.number({ style: "unit", unit: "meter" }), first);
});

void test("creates native list, date, and relative-time formatters", () => {
  const locale = useLocale("en-US");

  assert.ok(locale.list() instanceof Intl.ListFormat);
  assert.ok(locale.dateTime() instanceof Intl.DateTimeFormat);
  assert.ok(locale.relativeTime() instanceof Intl.RelativeTimeFormat);
});
