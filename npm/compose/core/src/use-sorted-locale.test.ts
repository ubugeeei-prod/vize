import assert from "node:assert/strict";
import { test } from "node:test";
import { ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useSortedLocale } from "./use-sorted-locale.ts";

void test("sorts strings by locale collation without mutating the source", () => {
  const words = ref(["zebra", "Äpfel", "apple", "Zoo"]);
  const source = words.value;
  const german = useSortedLocale(words, { locale: "de-DE" });
  assert.deepEqual(german.sorted.value, ["Äpfel", "apple", "zebra", "Zoo"]);
  assert.deepEqual(source, ["zebra", "Äpfel", "apple", "Zoo"]);

  const swedish = useSortedLocale(words, { locale: "sv-SE" });
  assert.deepEqual(swedish.sorted.value, ["apple", "zebra", "Zoo", "Äpfel"]);
});

void test("supports numeric collation, descending order, and keys", () => {
  const items = [{ name: "item 10" }, { name: "item 9" }, { name: "item 1" }];
  const sorted = useSortedLocale(items, {
    key: (item) => item.name,
    locale: "en-US",
    numeric: true,
  });
  assert.deepEqual(
    sorted.sorted.value.map((item) => item.name),
    ["item 1", "item 9", "item 10"],
  );

  const descending = useSortedLocale(["b", "a", "c"], { locale: "en-US", descending: true });
  assert.deepEqual(descending.sorted.value, ["c", "b", "a"]);
});

void test("is stable and follows reactive items and options", () => {
  const sensitivity = ref<Intl.CollatorOptions["sensitivity"]>("variant");
  const items = ref(["a", "A", "b"]);
  const sorted = useSortedLocale(items, () => ({
    locale: "en-US",
    sensitivity: sensitivity.value,
  }));
  assert.deepEqual(sorted.sorted.value, ["a", "A", "b"]);
  sensitivity.value = "base";
  items.value = ["A", "a", "b"];
  assert.deepEqual(sorted.sorted.value, ["A", "a", "b"], "equal keys keep their order");
  assert.equal(sorted.compare("a", "A"), 0);
});

void test("server rendering is deterministic", async () => {
  const state = await renderComposableOnServer(() => ({
    ja: useSortedLocale(["さくら", "あさひ", "かえで"], { locale: "ja-JP" }).sorted,
  }));
  assert.equal(state, '{"ja":["あさひ","かえで","さくら"]}');
});
