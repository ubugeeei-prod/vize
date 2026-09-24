import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import ComboboxInput from "../combobox/combobox-input.vue";
import SelectContent from "../select/select-content.vue";
import SelectItem from "../select/select-item.vue";
import AutocompleteRoot from "./autocomplete-root.vue";
import {
  createWebStorageHistory,
  pushAutocompleteHistory,
  removeAutocompleteHistory,
} from "./autocomplete-history.ts";
import type { AutocompleteSlotState } from "./autocomplete-types.ts";

interface Place {
  readonly id: string;
  readonly label: string;
}

const places: readonly Place[] = [
  { id: "tokyo", label: "Tokyo Station" },
  { id: "kyoto", label: "Kyoto Tower" },
  { id: "osaka", label: "Osaka Castle" },
];

function mountAutocomplete(props: Record<string, unknown> = {}) {
  return mountInteraction(AutocompleteRoot, {
    props: { by: "id", id: "place", itemText: (place: Place) => place.label, ...props },
    record: ["update:modelValue", "update:history", "submit"],
    slots: {
      default: (state: AutocompleteSlotState<Place>) => [
        h(ComboboxInput, { ariaLabel: "Place" }),
        h(SelectContent, { portalDisabled: true }, () => [
          h("div", { "data-heading": state.showingHistory ? "recent" : "results" }),
          ...state.suggestions.map((place) =>
            h(SelectItem<Place>, { key: place.id, value: place }, () => place.label),
          ),
          h(
            "button",
            {
              "data-clear": "",
              hidden: !state.showingHistory,
              onClick: state.clearHistory,
              tabindex: -1,
              type: "button",
            },
            "Clear",
          ),
        ]),
      ],
    },
  });
}

async function settle(): Promise<void> {
  for (let tick = 0; tick < 4; tick++) await nextTick();
}

function input(handle: ReturnType<typeof mountAutocomplete>): HTMLInputElement {
  const element = handle.getByRole("combobox", { name: "Place" });
  if (!(element instanceof HTMLInputElement)) throw new Error("input expected");
  return element;
}

async function type(element: HTMLInputElement, text: string): Promise<void> {
  element.value = text;
  element.dispatchEvent(
    new InputEvent("input", { bubbles: true, data: text, inputType: "insertText" }),
  );
  await settle();
}

function keydown(target: Element, key: string): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key });
  target.dispatchEvent(event);
  return event;
}

function options(handle: ReturnType<typeof mountAutocomplete>): string[] {
  return [...handle.root().querySelectorAll("[role='option']:not([hidden])")].map(
    (option) => option.textContent ?? "",
  );
}

test("is a free-text combobox preset that opens on focus and marks itself", async () => {
  const handle = mountAutocomplete({ items: places });
  const field = input(handle);
  assert.equal(handle.root().getAttribute("data-vize-ui"), "combobox");
  assert.equal(handle.root().getAttribute("data-vize-ui-preset"), "autocomplete");
  assert.equal(field.getAttribute("aria-autocomplete"), "list");
  field.dispatchEvent(new FocusEvent("focus"));
  await settle();
  assert.equal(field.getAttribute("aria-expanded"), "true");
  handle.unmount();
});

test("choosing a suggestion records it in history and emits submit", async () => {
  const handle = mountAutocomplete({ items: places });
  const field = input(handle);
  field.focus();
  await type(field, "kyo");
  assert.deepEqual(options(handle), ["Tokyo Station", "Kyoto Tower"], "contains-matching");
  await type(field, "kyot");
  assert.deepEqual(options(handle), ["Kyoto Tower"]);
  keydown(field, "Enter");
  await settle();
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[places[1]]]);
  assert.deepEqual(handle.wrapper.emitted("update:history"), [[[places[1]]]]);
  assert.deepEqual(handle.wrapper.emitted("submit"), [["Kyoto Tower", places[1]]]);
  assert.equal(handle.root().getAttribute("data-history-count"), "1");
  handle.unmount();
});

test("a chosen suggestion submits its label without a custom itemText", async () => {
  const handle = mountAutocomplete({ items: places, itemText: undefined });
  const field = input(handle);
  field.focus();
  await type(field, "kyot");
  keydown(field, "Enter");
  await settle();
  assert.deepEqual(handle.wrapper.emitted("submit"), [["Kyoto Tower", places[1]]]);
  handle.unmount();
});

test("an empty query shows history first, newest first, deduplicated and capped", async () => {
  const handle = mountAutocomplete({
    defaultHistory: [places[2], places[0]],
    items: places,
    maxHistory: 2,
  });
  const field = input(handle);
  field.dispatchEvent(new FocusEvent("focus"));
  await settle();
  assert.equal(
    handle.root().querySelector("[data-heading]")?.getAttribute("data-heading"),
    "recent",
  );
  assert.deepEqual(options(handle), ["Osaka Castle", "Tokyo Station"]);
  await type(field, "kyoto");
  assert.equal(
    handle.root().querySelector("[data-heading]")?.getAttribute("data-heading"),
    "results",
  );
  keydown(field, "Enter");
  await settle();
  assert.deepEqual(handle.wrapper.emitted("update:history")?.at(-1), [[places[1], places[2]]]);
  handle.unmount();
});

test("free-text Enter submits the text and fromText records search-style history", async () => {
  const handle = mountAutocomplete({
    fromText: (text: string) => ({ id: `q:${text}`, label: text }),
    items: places,
  });
  const field = input(handle);
  field.focus();
  await type(field, "ramen near me");
  assert.equal(field.getAttribute("aria-activedescendant"), null);
  keydown(field, "Enter");
  await settle();
  assert.deepEqual(handle.wrapper.emitted("submit"), [["ramen near me", null]]);
  assert.deepEqual(handle.wrapper.emitted("update:history"), [
    [[{ id: "q:ramen near me", label: "ramen near me" }]],
  ]);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), undefined);
  handle.unmount();
});

test("free-text submission does not reuse a previously chosen suggestion", async () => {
  const handle = mountAutocomplete({ items: places });
  const field = input(handle);
  field.focus();
  await type(field, "kyot");
  keydown(field, "Enter");
  await settle();

  await type(field, "ramen near me");
  assert.equal(field.getAttribute("aria-activedescendant"), null);
  keydown(field, "Enter");
  await settle();
  assert.deepEqual(handle.wrapper.emitted("submit"), [
    ["Kyoto Tower", places[1]],
    ["ramen near me", null],
  ]);
  handle.unmount();
});

test("slot clearHistory empties the history", async () => {
  const handle = mountAutocomplete({ defaultHistory: [places[0]], items: places });
  const field = input(handle);
  field.focus();
  keydown(field, "ArrowDown");
  await settle();
  const clear = handle.root().querySelector<HTMLButtonElement>("[data-clear]");
  if (clear === null) assert.fail("clear button expected");
  assert.equal(clear.hidden, false);
  await handle.click(clear);
  await settle();
  assert.deepEqual(handle.wrapper.emitted("update:history"), [[[]]]);
  assert.equal(clear.hidden, true);
  handle.unmount();
});

test("history storage is read after mount and written on change", async () => {
  const store = new Map<string, string>([["recent", JSON.stringify(["osaka"])]]);
  const storage = createWebStorageHistory<Place>({
    key: "recent",
    parse: (data) =>
      Array.isArray(data) ? places.filter((place) => data.includes(place.id)) : null,
    serialize: (entries) => entries.map((entry) => entry.id),
    storage: () => ({
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => {
        store.set(key, value);
      },
    }),
  });
  const handle = mountAutocomplete({ historyStorage: storage, items: places });
  await settle();
  assert.equal(handle.root().getAttribute("data-history-count"), "1");
  const field = input(handle);
  field.focus();
  await type(field, "tokyo");
  keydown(field, "Enter");
  await settle();
  assert.equal(store.get("recent"), JSON.stringify(["tokyo", "osaka"]));
  handle.unmount();
});

test("history helpers are pure and storage failures are swallowed", () => {
  const equals = (left: string, right: string) => left === right;
  assert.deepEqual(pushAutocompleteHistory(["a", "b", "c"], "b", 3, equals), ["b", "a", "c"]);
  assert.deepEqual(pushAutocompleteHistory(["a", "b"], "c", 2, equals), ["c", "a"]);
  assert.deepEqual(removeAutocompleteHistory(["a", "b"], "a", equals), ["b"]);
  const broken = createWebStorageHistory<string>({
    key: "x",
    parse: () => {
      throw new Error("corrupt");
    },
    storage: () => ({
      getItem: () => "{",
      setItem: () => {
        throw new Error("quota");
      },
    }),
  });
  assert.equal(broken.read(), null);
  assert.doesNotThrow(() => broken.write(["a"]));
});
