import assert from "node:assert/strict";
import { test } from "node:test";
import { ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useSegmenter } from "./use-segmenter.ts";

const family = "\u{1F468}‍\u{1F469}‍\u{1F467}";

void test("counts user-perceived characters", () => {
  const text = ref(`hi ${family}!`);
  const segmenter = useSegmenter(text, { locale: "en-US" });

  assert.equal(text.value.length, 12);
  assert.equal(segmenter.graphemeCount.value, 5);
  assert.equal(segmenter.count.value, 5);
  text.value = "é";
  assert.equal(segmenter.graphemeCount.value, 1, "combining marks join their base");
});

void test("counts words and exposes word segments", () => {
  const segmenter = useSegmenter("Hello, brave new world.", {
    locale: "en-US",
    granularity: "word",
  });
  assert.equal(segmenter.wordCount.value, 4);
  assert.deepEqual(
    segmenter.segments.value
      .filter((segment) => segment.isWordLike)
      .map((segment) => segment.segment),
    ["Hello", "brave", "new", "world"],
  );
  assert.equal(segmenter.segments.value[1]?.index, 5);
});

void test("segments languages without spaces through the locale", () => {
  const segmenter = useSegmenter("吾輩は猫である", { locale: "ja-JP", granularity: "word" });
  assert.ok(segmenter.wordCount.value > 1);
});

void test("segments sentences", () => {
  const granularity = ref<"sentence" | "grapheme">("sentence");
  const segmenter = useSegmenter("One. Two? Three!", () => ({
    locale: "en-US",
    granularity: granularity.value,
  }));
  assert.deepEqual(
    segmenter.segments.value.map((segment) => segment.segment),
    ["One. ", "Two? ", "Three!"],
  );
  granularity.value = "grapheme";
  assert.equal(segmenter.count.value, 16);
});

void test("truncates without splitting clusters", () => {
  const segmenter = useSegmenter(`ab${family}cd`, { locale: "en-US" });
  assert.equal(segmenter.truncate(10), `ab${family}cd`);
  assert.equal(segmenter.truncate(4), `ab${family}…`);
  assert.equal(segmenter.truncate(3, ""), `ab${family}`);
  assert.equal(segmenter.truncate(0), "");
  assert.throws(() => segmenter.truncate(-1), /VIZE_COMPOSE_SEGMENTER_INVALID_LENGTH/);
});

void test("server rendering is deterministic", async () => {
  const state = await renderComposableOnServer(() => {
    const segmenter = useSegmenter(`Vize ${family}`, { locale: "en-US" });
    return { graphemes: segmenter.graphemeCount, words: segmenter.wordCount };
  });
  assert.equal(state, '{"graphemes":6,"words":1}');
});
