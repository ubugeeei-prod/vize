import assert from "node:assert/strict";

import SelectRoot from "./select-root.vue";
import { mountInteraction } from "../../../testing/mount.ts";
import { test } from "vite-plus/test";

import { isPrintableKey, readBooleanProp } from "./select-keyboard.ts";
import {
  fruits,
  keydown,
  fruitSelectOptions,
  optionId,
  settle,
  trigger,
} from "./select-test-utils.ts";

function mountFruitSelect(
  rootProps: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(SelectRoot, fruitSelectOptions(rootProps, contentProps));
}

async function openWith(key: string, props: Record<string, unknown> = {}) {
  const handle = mountFruitSelect(props);
  const button = trigger(handle);
  button.focus();
  const event = keydown(button, key);
  await settle();
  return { button, event, handle };
}

function active(handle: ReturnType<typeof mountFruitSelect>, button: HTMLElement): string {
  const id = button.getAttribute("aria-activedescendant");
  if (id === null) return "";
  return handle.root().querySelector(`[id="${id}"]`)?.textContent?.replace("✓", "") ?? "";
}

for (const key of ["ArrowDown", "ArrowUp", "Enter", " "]) {
  test(`closed ${JSON.stringify(key)} opens and highlights the selected option`, async () => {
    const { button, event, handle } = await openWith(key, { defaultValue: fruits[2] });
    assert.equal(event.defaultPrevented, true);
    assert.equal(button.getAttribute("aria-expanded"), "true");
    assert.equal(active(handle, button), "Blueberry");
    assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
    handle.unmount();
  });
}

test("closed ArrowDown without a selection highlights the first option", async () => {
  const { button, handle } = await openWith("ArrowDown");
  assert.equal(active(handle, button), "Apple");
  handle.unmount();
});

test("closed Home and End open on the first and last enabled option", async () => {
  const home = await openWith("Home", { defaultValue: fruits[2] });
  assert.equal(active(home.handle, home.button), "Apple");
  home.handle.unmount();
  const end = await openWith("End", { defaultValue: fruits[0] });
  assert.equal(active(end.handle, end.button), "Date");
  end.handle.unmount();
});

test("closed printable characters open and run typeahead", async () => {
  const { button, event, handle } = await openWith("b");
  assert.equal(event.defaultPrevented, true);
  assert.equal(active(handle, button), "Banana");
  keydown(button, "l");
  await settle();
  assert.equal(active(handle, button), "Blueberry");
  const space = keydown(button, " ");
  await settle();
  assert.equal(space.defaultPrevented, true, "space continues a pending typeahead query");
  assert.equal(button.getAttribute("aria-expanded"), "true");
  handle.unmount();
});

test("open arrows skip disabled options and stop at the edges without loop", async () => {
  const { button, handle } = await openWith("ArrowDown", { defaultValue: fruits[2] });
  keydown(button, "ArrowDown");
  await settle();
  assert.equal(active(handle, button), "Date", "Cherry is disabled");
  keydown(button, "ArrowDown");
  await settle();
  assert.equal(active(handle, button), "Date");
  keydown(button, "ArrowUp");
  await settle();
  assert.equal(active(handle, button), "Blueberry");
  keydown(button, "Home");
  await settle();
  assert.equal(active(handle, button), "Apple");
  keydown(button, "ArrowUp");
  await settle();
  assert.equal(active(handle, button), "Apple");
  keydown(button, "End");
  await settle();
  assert.equal(active(handle, button), "Date");
  keydown(button, "PageUp");
  await settle();
  assert.equal(active(handle, button), "Apple");
  keydown(button, "PageDown");
  await settle();
  assert.equal(active(handle, button), "Date");
  handle.unmount();
});

test("loop wraps arrow navigation", async () => {
  const { button, handle } = await openWith("End", { loop: true });
  keydown(button, "ArrowDown");
  await settle();
  assert.equal(active(handle, button), "Apple");
  keydown(button, "ArrowUp");
  await settle();
  assert.equal(active(handle, button), "Date");
  handle.unmount();
});

for (const key of ["Enter", " "]) {
  test(`open ${JSON.stringify(key)} selects the highlighted option and closes`, async () => {
    const { button, handle } = await openWith("ArrowDown");
    keydown(button, "ArrowDown");
    await settle();
    const event = keydown(button, key);
    await settle();
    assert.equal(event.defaultPrevented, true);
    assert.equal(button.getAttribute("aria-expanded"), "false");
    assert.equal(button.getAttribute("aria-activedescendant"), null);
    assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[fruits[1]]]);
    assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], event);
    handle.unmount();
  });
}

test("Enter in multiple mode toggles and keeps the popup open", async () => {
  const { button, handle } = await openWith("ArrowDown", { multiple: true });
  keydown(button, "Enter");
  await settle();
  keydown(button, "ArrowDown");
  keydown(button, "Enter");
  await settle();
  keydown(button, "ArrowUp");
  keydown(button, "Enter");
  await settle();
  assert.equal(button.getAttribute("aria-expanded"), "true");
  assert.deepEqual(
    handle.wrapper.emitted("update:modelValue")?.map(([value]) => value),
    [[fruits[0]], [fruits[0], fruits[1]], [fruits[1]]],
  );
  handle.unmount();
});

test("Escape closes without selecting and is left alone while closed", async () => {
  const { button, handle } = await openWith("ArrowDown");
  keydown(button, "ArrowDown");
  const escape = keydown(button, "Escape");
  await settle();
  assert.equal(escape.defaultPrevented, true);
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
  const closedEscape = keydown(button, "Escape");
  assert.equal(closedEscape.defaultPrevented, false);
  handle.unmount();
});

test("Tab closes without committing unless selectOnTab is set", async () => {
  const plain = await openWith("ArrowDown");
  keydown(plain.button, "ArrowDown");
  const tab = keydown(plain.button, "Tab");
  await settle();
  assert.equal(tab.defaultPrevented, false, "Tab keeps its native focus movement");
  assert.equal(plain.button.getAttribute("aria-expanded"), "false");
  assert.equal(plain.handle.wrapper.emitted("update:modelValue"), undefined);
  plain.handle.unmount();

  const committing = await openWith("ArrowDown", { selectOnTab: true });
  keydown(committing.button, "ArrowDown");
  keydown(committing.button, "Tab");
  await settle();
  assert.deepEqual(committing.handle.wrapper.emitted("update:modelValue"), [[fruits[1]]]);
  committing.handle.unmount();
});

test("Alt+ArrowUp commits the highlighted option and closes", async () => {
  const { button, handle } = await openWith("ArrowDown");
  keydown(button, "ArrowDown");
  keydown(button, "ArrowUp", { altKey: true });
  await settle();
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[fruits[1]]]);
  handle.unmount();
});

test("IME composition and modified keys are ignored", async () => {
  const handle = mountFruitSelect();
  const button = trigger(handle);
  assert.equal(keydown(button, "ArrowDown", { isComposing: true }).defaultPrevented, false);
  assert.equal(keydown(button, "a", { ctrlKey: true }).defaultPrevented, false);
  await settle();
  assert.equal(button.getAttribute("aria-expanded"), "false");
  handle.unmount();
});

test("the highlighted option id tracks aria-activedescendant", async () => {
  const { button, handle } = await openWith("ArrowDown");
  assert.equal(button.getAttribute("aria-activedescendant"), optionId(handle, "Apple"));
  handle.unmount();
});

test("keyboard helpers classify printable keys and boolean attributes", () => {
  const make = (key: string, init: KeyboardEventInit = {}) =>
    new KeyboardEvent("keydown", { key, ...init });
  assert.equal(isPrintableKey(make("a")), true);
  assert.equal(isPrintableKey(make("é")), true);
  assert.equal(isPrintableKey(make(" ")), false);
  assert.equal(isPrintableKey(make("Enter")), false);
  assert.equal(isPrintableKey(make("a", { metaKey: true })), false);
  assert.equal(readBooleanProp(""), true);
  assert.equal(readBooleanProp(true), true);
  assert.equal(readBooleanProp(false), false);
  assert.equal(readBooleanProp(undefined), false);
});
