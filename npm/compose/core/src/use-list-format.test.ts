import assert from "node:assert/strict";
import { test } from "node:test";
import { ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useListFormat } from "./use-list-format.ts";

void test("joins the reactive list", () => {
  const items = ref(["apples", "pears"]);
  const list = useListFormat(items, { locale: "en-US" });

  assert.equal(list.formatted.value, "apples and pears");
  items.value = [...items.value, "plums"];
  assert.equal(list.formatted.value, "apples, pears, and plums");
  items.value = [];
  assert.equal(list.formatted.value, "");
});

void test("follows reactive options", () => {
  const type = ref<Intl.ListFormatType>("conjunction");
  const list = useListFormat(["a", "b", "c"], () => ({ locale: "en-US", type: type.value }));
  type.value = "disjunction";
  assert.equal(list.formatted.value, "a, b, or c");
});

void test("formats arbitrary iterables and parts", () => {
  const list = useListFormat([], { locale: "en-US", style: "short", type: "unit" });
  assert.equal(list.format(new Set(["1h", "5m"])), "1h, 5m");
  assert.deepEqual(
    list.formatToParts(["x", "y"]).map((part) => part.type),
    ["element", "literal", "element"],
  );
});

void test("surfaces invalid options as the platform RangeError", () => {
  const list = useListFormat(["a"], { locale: "en-US", style: "tiny" as "short" });
  assert.throws(() => list.formatted.value, RangeError);
});

void test("server rendering is deterministic", async () => {
  const state = await renderComposableOnServer(() => ({
    en: useListFormat(["Tokyo", "Osaka", "Kyoto"], { locale: "en-US" }).formatted,
    ja: useListFormat(["東京", "大阪", "京都"], { locale: "ja-JP" }).formatted,
  }));
  assert.equal(state, '{"en":"Tokyo, Osaka, and Kyoto","ja":"東京、大阪、京都"}');
});
