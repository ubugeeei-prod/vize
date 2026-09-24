import assert from "node:assert/strict";
import { test } from "node:test";
import { ref } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { usePluralRules } from "./use-plural-rules.ts";
import type { PluralMessages } from "./use-plural-rules.ts";

const files: PluralMessages = { "=0": "No files", one: "# file", other: "{count} files" };

void test("selects categories and interpolates the formatted count", () => {
  const count = ref(1);
  const plural = usePluralRules(count, files, { locale: "en-US" });

  assert.equal(plural.category.value, "one");
  assert.equal(plural.message.value, "1 file");
  count.value = 1200;
  assert.equal(plural.category.value, "other");
  assert.equal(plural.message.value, "1,200 files");
});

void test("prefers exact overrides over categories", () => {
  const plural = usePluralRules(0, files, { locale: "en-US" });
  assert.equal(plural.category.value, "other");
  assert.equal(plural.message.value, "No files");
});

void test("supports ordinal rules and function messages", () => {
  const suffixes = { one: "st", two: "nd", few: "rd", other: "th" } as const;
  const ordinal = usePluralRules(
    22,
    {
      one: (count) => `${count}${suffixes.one}`,
      two: (count) => `${count}${suffixes.two}`,
      few: (count) => `${count}${suffixes.few}`,
      other: (count) => `${count}${suffixes.other}`,
    },
    { locale: "en-US", type: "ordinal" },
  );
  assert.equal(ordinal.message.value, "22nd");
  assert.equal(ordinal.format(13), "13th");
  assert.equal(ordinal.select(3), "few");
});

void test("uses locale-specific categories and falls back to other", () => {
  const polish = usePluralRules(
    5,
    { one: "# plik", few: "# pliki", other: "# plików" },
    {
      locale: "pl-PL",
    },
  );
  assert.equal(polish.category.value, "many");
  assert.equal(polish.message.value, "5 plików", "missing categories fall back to other");
  assert.equal(polish.format(3), "3 pliki");

  const japanese = usePluralRules(1, { other: "#件" }, { locale: "ja-JP" });
  assert.equal(japanese.message.value, "1件");
});

void test("formats fractional counts with the configured digits", () => {
  const plural = usePluralRules(
    1.5,
    { one: "# star", other: "# stars" },
    {
      locale: "en-US",
      minimumFractionDigits: 1,
    },
  );
  assert.equal(plural.message.value, "1.5 stars");
});

void test("server rendering is deterministic", async () => {
  const state = await renderComposableOnServer(() => ({
    en: usePluralRules(2, files, { locale: "en-US" }).message,
    ar: usePluralRules(2, { two: "اثنان", other: "#" }, { locale: "ar" }).category,
  }));
  assert.equal(state, '{"en":"2 files","ar":"two"}');
});
