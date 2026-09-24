import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import SelectContent from "../select/select-content.vue";
import SelectItem from "../select/select-item.vue";
import ComboboxChip from "./combobox-chip.vue";
import ComboboxInput from "./combobox-input.vue";
import ComboboxRoot from "./combobox-root.vue";
import {
  containsComboboxFilter,
  inlineComboboxCompletion,
  normalizeComboboxText,
  startsWithComboboxFilter,
} from "./combobox-filter.ts";
import {
  activeOption,
  cities,
  comboboxInput,
  keydown,
  comboboxOptions,
  settle,
  type,
  visibleOptions,
} from "./combobox-test-utils.ts";
import type { City } from "./combobox-test-utils.ts";

function mountCombobox(
  rootProps: Record<string, unknown> = {},
  options: Parameters<typeof comboboxOptions>[1] = {},
) {
  return mountInteraction(ComboboxRoot, comboboxOptions(rootProps, options));
}

test("renders APG combobox semantics on a native text input", async () => {
  const handle = mountCombobox();
  await settle();
  const input = comboboxInput(handle);
  assert.equal(handle.root().getAttribute("data-vize-ui"), "combobox");
  assert.equal(input.id, "city-input");
  assert.equal(input.type, "text");
  assert.equal(input.getAttribute("aria-autocomplete"), "list");
  assert.equal(input.getAttribute("aria-expanded"), "false");
  assert.equal(input.getAttribute("aria-controls"), null);
  assert.equal(input.getAttribute("autocomplete"), "off");
  assert.equal(input.placeholder, "Search cities");
  assert.equal(handle.root().querySelector("[data-vize-ui='combobox-content']"), null);
  handle.unmount();
});

test("unnamed combobox accepts a cyclic selected value", async () => {
  const cyclic: { label: string; self?: unknown } = { label: "Cycle" };
  cyclic.self = cyclic;
  const handle = mountCombobox({
    by: undefined,
    defaultValue: cyclic,
    itemText: (value: typeof cyclic) => value.label,
  });
  await settle();
  assert.equal(comboboxInput(handle).value, "Cycle");
  handle.unmount();
});

test("typing opens, filters accent-insensitively, highlights the first match, and Enter selects", async () => {
  const handle = mountCombobox();
  const input = comboboxInput(handle);
  input.focus();
  await type(input, "bo");
  assert.equal(input.getAttribute("aria-expanded"), "true");
  assert.equal(input.getAttribute("aria-controls"), "city-listbox");
  const listbox = handle.root().querySelector("[data-vize-ui='combobox-content']");
  assert.equal(listbox?.getAttribute("role"), "listbox");
  assert.deepEqual(visibleOptions(handle), ["Bogotá", "Boston"]);
  assert.equal(activeOption(handle), "Bogotá");

  await type(input, "bogota");
  assert.deepEqual(visibleOptions(handle), ["Bogotá"]);
  const enter = keydown(input, "Enter");
  await settle();
  assert.equal(enter.defaultPrevented, true);
  assert.equal(input.value, "Bogotá");
  assert.equal(input.getAttribute("aria-expanded"), "false");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[cities[1]]]);
  assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], enter);
  handle.unmount();
});

test("items mode exposes filteredItems and filter functions are injectable", async () => {
  const handle = mountCombobox({ filter: startsWithComboboxFilter, items: cities });
  const input = comboboxInput(handle);
  await type(input, "o");
  assert.deepEqual(visibleOptions(handle), [], "startsWith rejects inner matches");
  assert.equal(
    handle.root().querySelector("[data-vize-ui='combobox-empty']")?.hasAttribute("hidden"),
    false,
  );
  await type(input, "k");
  assert.deepEqual(visibleOptions(handle), ["Kyoto"]);
  handle.unmount();

  const unfiltered = mountCombobox({ filter: false, items: cities });
  await type(comboboxInput(unfiltered), "zzz");
  assert.equal(visibleOptions(unfiltered).length, cities.length);
  unfiltered.unmount();
});

test("inline autocomplete completes the first match and selects the completion", async () => {
  const handle = mountCombobox({ autocomplete: "inline" });
  const input = comboboxInput(handle);
  input.focus();
  assert.equal(input.getAttribute("aria-autocomplete"), "inline");
  await type(input, "ky");
  assert.equal(input.value, "kyoto", "the typed prefix is kept verbatim");
  assert.equal(input.selectionStart, 2);
  assert.equal(input.selectionEnd, 5);
  assert.equal(activeOption(handle), "Kyoto");
  assert.equal(visibleOptions(handle).length, cities.length, "inline mode does not filter");

  await type(input, "k", "deleteContentBackward");
  assert.equal(input.value, "k", "deletion never re-completes");
  handle.unmount();
});

test("both autocomplete filters and completes", async () => {
  const handle = mountCombobox({ autocomplete: "both" });
  const input = comboboxInput(handle);
  input.focus();
  await type(input, "zu");
  assert.equal(input.value, "zurich");
  assert.deepEqual(visibleOptions(handle), ["Zürich"]);
  handle.unmount();
});

test("autocomplete none lists everything without highlighting", async () => {
  const handle = mountCombobox({ autocomplete: "none" });
  const input = comboboxInput(handle);
  await type(input, "ky");
  assert.equal(visibleOptions(handle).length, cities.length);
  assert.equal(input.getAttribute("aria-activedescendant"), null);
  handle.unmount();
});

test("strict mode restores the selected label on blur; free text survives otherwise", async () => {
  const strict = mountCombobox({ defaultValue: cities[3] });
  const input = comboboxInput(strict);
  assert.equal(input.value, "Kyoto");
  input.focus();
  await type(input, "Kyo");
  input.dispatchEvent(new FocusEvent("blur", { relatedTarget: null }));
  await settle();
  assert.equal(input.value, "Kyoto");
  strict.unmount();

  const free = mountCombobox({ name: "city", strict: false });
  const freeInput = comboboxInput(free);
  await type(freeInput, "Atlantis");
  freeInput.dispatchEvent(new FocusEvent("blur", { relatedTarget: null }));
  await settle();
  assert.equal(freeInput.value, "Atlantis");
  const hidden = free.root().querySelector<HTMLInputElement>("[data-vize-ui='combobox-native']");
  assert.equal(hidden?.value, "Atlantis");
  assert.equal(free.wrapper.emitted("update:modelValue"), undefined);
  free.unmount();
});

test("clearing the text clears a strict single selection", async () => {
  const handle = mountCombobox({ defaultValue: cities[0] });
  await type(comboboxInput(handle), "", "deleteContentBackward");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[null]]);
  handle.unmount();
});

test("createOption offers a create option unless the text matches exactly", async () => {
  let nextId = 100;
  const handle = mountCombobox({
    createOption: (text: string) => ({ id: nextId++, name: text }),
  });
  const input = comboboxInput(handle);
  await type(input, "Kyoto");
  const create = () => handle.root().querySelector("[data-vize-ui='combobox-create-item']");
  assert.equal(create()?.hasAttribute("hidden"), true, "exact matches hide create");

  await type(input, "Lima");
  assert.equal(create()?.hasAttribute("hidden"), false);
  assert.equal(create()?.textContent, 'Create "Lima"');
  assert.equal(activeOption(handle), 'Create "Lima"', "create is the first visible option");
  keydown(input, "Enter");
  await settle();
  assert.deepEqual(handle.wrapper.emitted("create"), [[{ id: 100, name: "Lima" }, "Lima"]]);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[{ id: 100, name: "Lima" }]]);
  assert.equal(input.value, "Lima");
  handle.unmount();
});

test("clicking the create option creates the typed value", async () => {
  const handle = mountCombobox({ createOption: (text: string) => ({ id: 7, name: text }) });
  await type(comboboxInput(handle), "Oslo");
  const create = handle.root().querySelector("[data-vize-ui='combobox-create-item']");
  if (!(create instanceof HTMLElement)) assert.fail("create option must render");
  await handle.click(create);
  assert.deepEqual(handle.wrapper.emitted("create")?.[0]?.[1], "Oslo");
  handle.unmount();
});

test("inline completion creates the typed query rather than the suggestion", async () => {
  const handle = mountCombobox({
    autocomplete: "inline",
    createOption: (text: string) => ({ id: 7, name: text }),
  });
  const input = comboboxInput(handle);
  await type(input, "Ky");
  assert.equal(input.value, "Kyoto");
  const create = handle.root().querySelector("[data-vize-ui='combobox-create-item']");
  if (!(create instanceof HTMLElement)) assert.fail("create option must render");
  await handle.click(create);
  assert.equal(handle.wrapper.emitted("create")?.[0]?.[1], "Ky");
  handle.unmount();
});

test("readonly combobox never offers or creates a custom option", async () => {
  const handle = mountCombobox({
    createOption: (text: string) => ({ id: 7, name: text }),
    defaultOpen: true,
    readonly: true,
  });
  (handle.wrapper.vm as unknown as { setInputValue(value: string): void }).setInputValue("Oslo");
  await settle();
  assert.equal(
    handle.root().querySelector("[data-vize-ui='combobox-create-item']")?.hasAttribute("hidden"),
    true,
  );
  keydown(comboboxInput(handle), "Enter");
  await settle();
  assert.equal(handle.wrapper.emitted("create"), undefined);
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
  handle.unmount();
});

test("multiple mode renders chips, keeps the popup open, and deletes with Backspace", async () => {
  const handle = mountCombobox({ defaultValue: [cities[0]], multiple: true });
  const input = comboboxInput(handle);
  input.focus();
  assert.equal(input.value, "");
  assert.equal(handle.root().querySelectorAll("[data-vize-ui='combobox-chip']").length, 1);

  await type(input, "ky");
  keydown(input, "Enter");
  await settle();
  assert.equal(input.value, "", "multiple mode clears the text after choosing");
  assert.equal(input.getAttribute("aria-expanded"), "true");
  assert.deepEqual(
    [...handle.root().querySelectorAll("[data-vize-ui='combobox-chip']")].map((chip) =>
      chip.textContent?.replace("×", ""),
    ),
    ["Berlin", "Kyoto"],
  );

  const backspace = keydown(input, "Backspace");
  await settle();
  assert.equal(backspace.defaultPrevented, true);
  assert.equal(handle.root().querySelectorAll("[data-vize-ui='combobox-chip']").length, 1);

  const remove = handle.getByRole("button", { name: "Remove Berlin" });
  assert.equal(remove.tabIndex, -1);
  const changesBeforeClick = handle.wrapper.emitted("change")?.length ?? 0;
  await handle.click(remove);
  assert.equal(handle.root().querySelectorAll("[data-vize-ui='combobox-chip']").length, 0);
  const clickChange = handle.wrapper.emitted("change")?.[changesBeforeClick];
  assert.deepEqual(clickChange?.[0], []);
  assert.ok(clickChange?.[2] instanceof MouseEvent);
  assert.deepEqual(
    handle.wrapper.emitted("update:modelValue")?.map(([value]) => value),
    [[cities[0], cities[3]], [cities[0]], []],
  );
  handle.unmount();
});

test("Escape leaves a closed multiple selection untouched", async () => {
  const handle = mountCombobox({ defaultValue: [cities[0]], multiple: true });
  const input = comboboxInput(handle);
  const escape = keydown(input, "Escape");
  await settle();
  assert.equal(escape.defaultPrevented, false);
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
  handle.unmount();
});

test("keyboard matrix: arrows, Alt+arrows, PageDown, Escape, and Tab", async () => {
  const handle = mountCombobox({ defaultValue: cities[2] });
  const input = comboboxInput(handle);
  input.focus();

  keydown(input, "ArrowDown");
  await settle();
  assert.equal(activeOption(handle), "Boston", "opening highlights the selection");
  keydown(input, "ArrowDown");
  await settle();
  assert.equal(activeOption(handle), "Kyoto");
  keydown(input, "PageDown");
  await settle();
  assert.equal(activeOption(handle), "Zürich");
  keydown(input, "ArrowUp", { altKey: true });
  await settle();
  assert.equal(input.getAttribute("aria-expanded"), "false");

  keydown(input, "ArrowUp");
  await settle();
  assert.equal(activeOption(handle), "Zürich", "ArrowUp opens on the last option");
  keydown(input, "Escape");
  await settle();
  assert.equal(input.getAttribute("aria-expanded"), "false");

  keydown(input, "ArrowDown", { altKey: true });
  await settle();
  assert.equal(input.getAttribute("aria-expanded"), "true");
  assert.equal(input.getAttribute("aria-activedescendant"), null, "Alt+ArrowDown only opens");
  const tab = keydown(input, "Tab");
  await settle();
  assert.equal(tab.defaultPrevented, false);
  assert.equal(input.getAttribute("aria-expanded"), "false");

  const home = keydown(input, "Home");
  assert.equal(home.defaultPrevented, false, "Home and End keep native caret movement");
  const escape = keydown(input, "Escape");
  await settle();
  assert.equal(escape.defaultPrevented, true);
  assert.equal(input.value, "", "Escape while closed clears the text");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue")?.at(-1), [null]);
  handle.unmount();
});

test("Enter without a highlighted option keeps native form submission", async () => {
  const handle = mountCombobox();
  const enter = keydown(comboboxInput(handle), "Enter");
  assert.equal(enter.defaultPrevented, false);
  handle.unmount();
});

test("the toggle button and anchor keep focus in the input", async () => {
  const handle = mountCombobox();
  const input = comboboxInput(handle);
  input.focus();
  const toggle = handle.getByRole("button", { name: "Show suggestions" });
  assert.equal(toggle.tabIndex, -1);
  await handle.click(toggle);
  await settle();
  assert.equal(input.getAttribute("aria-expanded"), "true");
  assert.equal(toggle.getAttribute("aria-expanded"), "true");
  assert.equal(document.activeElement, input);

  const pointerdown = new PointerEvent("pointerdown", { bubbles: true, cancelable: true });
  handle.root().querySelector("[role='option']")?.dispatchEvent(pointerdown);
  assert.equal(pointerdown.defaultPrevented, true);
  await handle.click(handle.getByRole("option", { name: "Kyoto" }));
  assert.equal(input.value, "Kyoto");

  input.blur();
  const anchor = handle.root().querySelector("[data-vize-ui='combobox-anchor']");
  const anchorDown = new PointerEvent("pointerdown", { bubbles: true, cancelable: true });
  anchor?.dispatchEvent(anchorDown);
  assert.equal(anchorDown.defaultPrevented, true);
  assert.equal(document.activeElement, input, "pressing the anchor focuses the input");
  handle.unmount();
});

test("openOnFocus opens when the input gains focus", async () => {
  const handle = mountCombobox({ openOnFocus: true });
  const input = comboboxInput(handle);
  input.dispatchEvent(new FocusEvent("focus"));
  await settle();
  assert.equal(input.getAttribute("aria-expanded"), "true");
  handle.unmount();
});

test("disabled and readonly inputs refuse edits", async () => {
  const disabled = mountCombobox({ disabled: true });
  assert.equal(comboboxInput(disabled).disabled, true);
  keydown(comboboxInput(disabled), "ArrowDown");
  await settle();
  assert.equal(comboboxInput(disabled).getAttribute("aria-expanded"), "false");
  disabled.unmount();

  const readonly = mountCombobox({ defaultValue: [cities[0]], multiple: true, readonly: true });
  const input = comboboxInput(readonly);
  assert.equal(input.readOnly, true);
  keydown(input, "ArrowDown");
  keydown(input, "Backspace");
  await settle();
  assert.equal(input.getAttribute("aria-expanded"), "false");
  assert.equal(readonly.wrapper.emitted("update:modelValue"), undefined);
  readonly.unmount();
});

test("controlled inputValue waits for the parent", async () => {
  const handle = mountCombobox({ inputValue: "Ber" });
  const input = comboboxInput(handle);
  assert.equal(input.value, "Ber");
  await type(input, "Bert");
  assert.deepEqual(handle.wrapper.emitted("update:inputValue"), [["Bert"]]);
  assert.equal(input.value, "Ber");
  handle.unmount();
});

test("parts require a Combobox provider", () => {
  assert.throws(() => mountInteraction(ComboboxInput), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(
    () => mountInteraction(ComboboxChip, { props: { value: "x" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});

test("Select popup parts publish combobox part names inside a ComboboxRoot", async () => {
  const handle = mountInteraction(ComboboxRoot, {
    props: { defaultOpen: true },
    slots: {
      default: () => [
        h(ComboboxInput, { ariaLabel: "Fruit", ariaLabelledby: "fruit-label" }),
        h(SelectContent, { portalDisabled: true }, () => [
          h(SelectItem<string>, { value: "apple" }, () => "Apple"),
        ]),
      ],
    },
  });
  await settle();
  const listbox = handle.root().querySelector("[role='listbox']");
  assert.equal(listbox?.getAttribute("data-vize-ui"), "combobox-content");
  assert.equal(listbox?.getAttribute("aria-labelledby"), "fruit-label");
  assert.equal(
    handle.root().querySelector("[role='option']")?.getAttribute("data-vize-ui"),
    "combobox-item",
  );
  handle.unmount();
});

test("filter helpers normalize accents, case, and inline completion", () => {
  const city: City = { id: 1, name: "Zürich" };
  assert.equal(normalizeComboboxText("  ÉCOLE "), "ecole");
  assert.equal(containsComboboxFilter(city, "RIC", "Zürich"), true);
  assert.equal(containsComboboxFilter(city, "", "Zürich"), true);
  assert.equal(startsWithComboboxFilter(city, "zu", "Zürich"), true);
  assert.equal(startsWithComboboxFilter(city, "ri", "Zürich"), false);
  assert.equal(inlineComboboxCompletion("zu", "Zürich"), "zurich", "keeps the typed prefix");
  assert.equal(inlineComboboxCompletion("", "Zürich"), null);
  assert.equal(inlineComboboxCompletion("xy", "Zürich"), null);
});
