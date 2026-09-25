import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import EmojiPickerCategory from "./emoji-picker-category.vue";
import EmojiPickerEmpty from "./emoji-picker-empty.vue";
import EmojiPickerGrid from "./emoji-picker-grid.vue";
import EmojiPickerItem from "./emoji-picker-item.vue";
import EmojiPickerPreview from "./emoji-picker-preview.vue";
import EmojiPickerRoot from "./emoji-picker-root.vue";
import EmojiPickerSearch from "./emoji-picker-search.vue";
import EmojiPickerSkinTone from "./emoji-picker-skin-tone.vue";
import { fixtureEmoji, renderEmojiPickerTree } from "./emoji-picker-fixture-tree.ts";
import {
  buildEmojiSections,
  containsEmojiFilter,
  createEmojiVirtualGrid,
  emojiGlyph,
  toSkinTone,
} from "./emoji-picker-model.ts";
import type { FixtureEmoji } from "./emoji-picker-fixture-tree.ts";

void [
  EmojiPickerCategory,
  EmojiPickerEmpty,
  EmojiPickerItem,
  EmojiPickerPreview,
  EmojiPickerRoot,
  EmojiPickerSearch,
  EmojiPickerSkinTone,
];

function mountPicker(props: Record<string, unknown> = {}) {
  const Probe = defineComponent({
    name: "EmojiPickerProbe",
    emits: ["select", "update:skinTone", "update:search"],
    setup:
      (_, { emit }) =>
      () =>
        renderEmojiPickerTree({
          onSelect: (item: FixtureEmoji, glyph: string) => emit("select", item, glyph),
          "onUpdate:search": (text: string) => emit("update:search", text),
          "onUpdate:skinTone": (tone: number) => emit("update:skinTone", tone),
          ...props,
        }),
  });
  return mountInteraction(Probe);
}

function keydown(target: Element, key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init });
  target.dispatchEvent(event);
  return event;
}

function grid(handle: ReturnType<typeof mountPicker>): HTMLElement {
  return handle.getByRole("grid", { name: "Emoji" });
}

function active(handle: ReturnType<typeof mountPicker>): string | null {
  const id = grid(handle).getAttribute("aria-activedescendant");
  return id === null ? null : (document.getElementById(id)?.getAttribute("aria-label") ?? null);
}

async function press(
  handle: ReturnType<typeof mountPicker>,
  key: string,
  init?: KeyboardEventInit,
) {
  const event = keydown(grid(handle), key, init);
  await nextTick();
  return event;
}

function sectionLabels(handle: ReturnType<typeof mountPicker>): string[] {
  return [...handle.root().querySelectorAll("[role='columnheader']")].map(
    (header) => header.textContent ?? "",
  );
}

test("renders an APG grid with labelled category rowgroups and gridcells", async () => {
  const handle = mountPicker();
  await nextTick();
  const element = grid(handle);
  assert.equal(element.tabIndex, 0);
  assert.equal(element.getAttribute("aria-colcount"), "3");
  assert.equal(element.getAttribute("aria-rowcount"), "5", "3 item rows plus 2 label rows");
  assert.deepEqual(sectionLabels(handle), ["Smileys", "Animals"]);
  const group = handle.getByRole("rowgroup", { name: "Smileys" });
  assert.equal(group.querySelectorAll("[role='row']").length, 3);
  assert.equal(handle.getByRole("gridcell", { name: "wave" }).textContent, "👋");
  assert.equal(
    handle.root().querySelector("[data-vize-ui='emoji-picker-empty']")?.hasAttribute("hidden"),
    true,
  );
  handle.unmount();
});

test("2-D keyboard navigation crosses section boundaries over padded rows", async () => {
  const handle = mountPicker();
  grid(handle).focus();
  await nextTick();
  assert.equal(active(handle), "grinning");
  assert.equal((await press(handle, "ArrowDown")).defaultPrevented, true);
  assert.equal(active(handle), "smile");
  await press(handle, "ArrowDown");
  assert.equal(active(handle), "cat", "down enters the next section");
  await press(handle, "ArrowRight");
  assert.equal(active(handle), "dog");
  await press(handle, "ArrowUp");
  assert.equal(active(handle), "joy", "up skips the padding cell above dog");
  await press(handle, "End");
  assert.equal(active(handle), "wave");
  await press(handle, "ArrowRight");
  assert.equal(active(handle), "smile", "right wraps to the next row");
  await press(handle, "ArrowRight");
  assert.equal(active(handle), "cat", "padding is skipped horizontally");
  await press(handle, "ArrowLeft");
  assert.equal(active(handle), "smile");
  await press(handle, "Home");
  assert.equal(active(handle), "smile");
  await press(handle, "End", { ctrlKey: true });
  assert.equal(active(handle), "dog");
  await press(handle, "Home", { ctrlKey: true });
  assert.equal(active(handle), "grinning");
  await press(handle, "PageDown");
  assert.equal(active(handle), "cat");
  await press(handle, "PageUp");
  assert.equal(active(handle), "grinning");
  handle.unmount();
});

test("Enter, Space, and click emit select with the skin tone applied", async () => {
  const handle = mountPicker({ defaultSkinTone: 3 });
  grid(handle).focus();
  await nextTick();
  await press(handle, "End");
  const enter = await press(handle, "Enter");
  assert.equal(enter.defaultPrevented, true);
  await press(handle, "ArrowLeft");
  await press(handle, " ");
  await handle.click(handle.getByRole("gridcell", { name: "cat" }));
  assert.deepEqual(
    handle.wrapper.emitted("select")?.map(([item, glyph]) => [(item as FixtureEmoji).name, glyph]),
    [
      ["wave", "👋🏽"],
      ["joy", "😂"],
      ["cat", "🐱"],
    ],
  );
  assert.equal(active(handle), "cat", "clicked cells become active");
  handle.unmount();
});

test("the skin-tone radiogroup uses roving focus and updates glyphs", async () => {
  const handle = mountPicker();
  const group = handle.getByRole("radiogroup", { name: "Skin tone" });
  const radios = [...group.querySelectorAll<HTMLElement>("[role='radio']")];
  assert.equal(radios.length, 6);
  assert.deepEqual(
    radios.map((radio) => radio.tabIndex),
    [0, -1, -1, -1, -1, -1],
  );
  radios[0]?.focus();
  keydown(radios[0] as HTMLElement, "ArrowRight");
  await nextTick();
  assert.equal(document.activeElement, radios[1]);
  assert.equal(radios[1]?.getAttribute("aria-checked"), "true");
  assert.equal(handle.getByRole("gridcell", { name: "wave" }).textContent, "👋🏻");
  keydown(radios[1] as HTMLElement, "End");
  await nextTick();
  keydown(radios[5] as HTMLElement, "ArrowRight");
  await nextTick();
  assert.equal(radios[0]?.getAttribute("aria-checked"), "true", "arrows wrap");
  await handle.click(radios[2] as HTMLElement);
  assert.deepEqual(
    handle.wrapper.emitted("update:skinTone")?.map(([tone]) => tone),
    [1, 5, 0, 2],
  );
  handle.unmount();
});

test("controlled skin tone waits for the parent", async () => {
  const handle = mountPicker({ skinTone: 0 });
  await handle.click(handle.getByRole("radio", { name: "Dark" }));
  assert.deepEqual(handle.wrapper.emitted("update:skinTone"), [[5]]);
  assert.equal(handle.getByRole("gridcell", { name: "wave" }).textContent, "👋");
  handle.unmount();
});

test("search filters by name and keyword, highlights the first match, and drives the empty state", async () => {
  const handle = mountPicker();
  const search = handle.getByRole("searchbox", { name: "Search emoji" });
  if (!(search instanceof HTMLInputElement)) assert.fail("search input expected");
  assert.equal(search.getAttribute("aria-controls"), grid(handle).id);
  search.value = "HELLO";
  search.dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();
  assert.deepEqual(sectionLabels(handle), ["Search results"]);
  assert.equal(active(handle), "wave");
  assert.deepEqual(handle.wrapper.emitted("update:search"), [["HELLO"]]);
  const enter = keydown(search, "Enter");
  assert.equal(enter.defaultPrevented, true);
  const selected = handle.wrapper.emitted("select")?.[0]?.[0] as FixtureEmoji | undefined;
  assert.equal(selected?.name, "wave");

  search.value = "zzz";
  search.dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();
  const empty = handle.root().querySelector("[data-vize-ui='emoji-picker-empty']");
  assert.equal(empty?.hasAttribute("hidden"), false);
  assert.equal(empty?.textContent, "No emoji found");
  assert.equal(handle.root().getAttribute("data-empty"), "true");

  keydown(search, "Escape");
  await nextTick();
  assert.deepEqual(sectionLabels(handle), ["Smileys", "Animals"]);
  keydown(search, "ArrowDown");
  await nextTick();
  assert.equal(document.activeElement, grid(handle), "ArrowDown moves into the grid");
  assert.equal(active(handle), "grinning");
  handle.unmount();
});

test("recent items form the first section and repeat safely", async () => {
  const handle = mountPicker({ recent: [fixtureEmoji[5]] });
  await nextTick();
  assert.deepEqual(sectionLabels(handle), ["Recently used", "Smileys", "Animals"]);
  grid(handle).focus();
  await nextTick();
  assert.equal(active(handle), "dog");
  await press(handle, "ArrowDown");
  assert.equal(active(handle), "grinning");
  const ids = [...handle.root().querySelectorAll("[role='gridcell']")].map((cell) => cell.id);
  assert.equal(new Set(ids).size, ids.length, "repeated items keep unique cell ids");
  handle.unmount();
});

test("the preview follows the highlighted emoji", async () => {
  const handle = mountPicker();
  const preview = () => handle.root().querySelector("[data-vize-ui='emoji-picker-preview']");
  assert.equal(preview()?.getAttribute("data-empty"), "true");
  handle
    .getByRole("gridcell", { name: "dog" })
    .dispatchEvent(new PointerEvent("pointermove", { bubbles: true }));
  await nextTick();
  assert.equal(preview()?.textContent, "🐶 dog");
  assert.equal(preview()?.getAttribute("aria-live"), "polite");
  handle.unmount();
});

test("parts require an EmojiPicker provider", () => {
  assert.throws(() => mountInteraction(EmojiPickerGrid), /VIZE_UI_CONTEXT_MISSING/);
});

test("pure helpers build sections, glyphs, and the padded virtual grid", () => {
  const accessors = {
    getCategory: (item: FixtureEmoji) => item.category,
    getEmoji: (item: FixtureEmoji) => item.emoji,
    getName: (item: FixtureEmoji) => item.name,
    getSkins: (item: FixtureEmoji) => item.skins,
  };
  const sections = buildEmojiSections({
    accessors,
    categories: [{ id: "animals", label: "Animals" }],
    columns: 3,
    defaultLabel: "Emoji",
    filter: containsEmojiFilter,
    items: fixtureEmoji,
    recentLabel: "Recent",
    search: "",
    searchLabel: "Results",
  });
  assert.deepEqual(
    sections.map((section) => [section.id, section.label, section.rows.length]),
    [
      ["animals", "Animals", 1],
      ["smileys", "smileys", 2],
    ],
    "unknown categories follow in first-seen order",
  );
  const layout = createEmojiVirtualGrid(sections, 3);
  assert.equal(layout.count, 9);
  assert.equal(layout.cellAt(2), undefined);
  assert.deepEqual(layout.cellAt(6), { index: 3, section: 1 });
  assert.equal(layout.indexOf(1, 3), 6);
  assert.equal(emojiGlyph(fixtureEmoji[2] as FixtureEmoji, 5, accessors), "👋🏿");
  assert.equal(emojiGlyph(fixtureEmoji[0] as FixtureEmoji, 5, accessors), "😀");
  assert.equal(toSkinTone(9), 5);
  assert.equal(toSkinTone(Number.NaN), 0);
  assert.equal(
    containsEmojiFilter(fixtureEmoji[5] as FixtureEmoji, "PUP", {
      ...accessors,
      getKeywords: (item: FixtureEmoji) => item.keywords ?? [],
    }),
    true,
  );
});
