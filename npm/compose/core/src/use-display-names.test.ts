import assert from "node:assert/strict";
import { test } from "node:test";
import { ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useDisplayNames } from "./use-display-names.ts";

void test("names the reactive code", () => {
  const code = ref<string | null>("JP");
  const region = useDisplayNames(code, { type: "region", locale: "en-US" });

  assert.equal(region.name.value, "Japan");
  code.value = "DE";
  assert.equal(region.name.value, "Germany");
  code.value = null;
  assert.equal(region.name.value, undefined);
});

void test("names languages, currencies, scripts, and fields in the locale", () => {
  const language = useDisplayNames("ja", { type: "language", locale: "ja-JP" });
  assert.equal(language.name.value, "日本語");
  const currency = useDisplayNames("EUR", { type: "currency", locale: "en-US" });
  assert.equal(currency.name.value, "Euro");
  const script = useDisplayNames("Latn", { type: "script", locale: "en-US" });
  assert.equal(script.name.value, "Latin");
  const field = useDisplayNames("month", { type: "dateTimeField", locale: "en-US" });
  assert.equal(field.name.value, "month");
  const standard = useDisplayNames("en-GB", {
    type: "language",
    locale: "en-US",
    languageDisplay: "standard",
  });
  assert.equal(standard.name.value, "English (United Kingdom)");
});

void test("returns undefined for malformed codes and honors the fallback", () => {
  const region = useDisplayNames("not a region!", { type: "region", locale: "en-US" });
  assert.equal(region.name.value, undefined);
  assert.equal(region.of("ZZ"), "Unknown Region");

  const none = useDisplayNames("XA", { type: "region", locale: "en-US", fallback: "none" });
  assert.equal(none.of("QM"), undefined);
});

void test("follows the reactive locale", () => {
  const locale = ref("en-US");
  const names = useDisplayNames("FR", () => ({ type: "region", locale: locale.value }));
  assert.equal(names.name.value, "France");
  locale.value = "de-DE";
  assert.equal(names.name.value, "Frankreich");
});

void test("server rendering is deterministic", async () => {
  const state = await renderComposableOnServer(() => ({
    en: useDisplayNames("JP", { type: "region", locale: "en-US" }).name,
    ja: useDisplayNames("US", { type: "region", locale: "ja-JP" }).name,
  }));
  assert.equal(state, '{"en":"Japan","ja":"アメリカ合衆国"}');
});
