import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";

import MenuContent from "./menu-content.vue";
import { MenuFixture } from "./menu-fixture.ts";
import MenuItem from "./menu-item.vue";
import MenuRoot from "./menu-root.vue";
import MenuSub from "./menu-sub.vue";
import MenuSubContent from "./menu-sub-content.vue";
import MenuSubTrigger from "./menu-sub-trigger.vue";
import MenuTrigger from "./menu-trigger.vue";
import {
  createMenuMounter,
  focusedText,
  keydown,
  one,
  openMenus,
  press,
  settle,
} from "./menu-test-utils.ts";

const { cleanup, mountMenu } = createMenuMounter(mountInteraction);

afterEach(cleanup);

function trigger(): HTMLElement {
  return one('[data-vize-ui="menu-trigger"]');
}

function isTrigger(): boolean {
  return document.activeElement?.getAttribute("data-vize-ui") === "menu-trigger";
}

const openingKeys = [
  { key: "Enter", expected: "New file" },
  { key: " ", expected: "New file" },
  { key: "ArrowDown", expected: "New file" },
  { key: "ArrowUp", expected: "Share" },
] as const;

for (const { key, expected } of openingKeys) {
  test(`trigger ${JSON.stringify(key)} opens the menu and focuses ${expected}`, async () => {
    const handle = mountMenu(MenuFixture);
    trigger().focus();
    const event = keydown(trigger(), key);
    await settle();
    assert.equal(event.defaultPrevented, true);
    assert.equal(openMenus().length, 1);
    assert.equal(focusedText(), expected);
    assert.equal(document.activeElement?.getAttribute("data-highlighted"), "true");
    handle.unmount();
  });
}

test("arrow, Home/End, and Page keys move focus without wrapping by default", async () => {
  const handle = mountMenu(MenuFixture);
  trigger().focus();
  await press("ArrowDown");
  const moves: Array<[string, string]> = [
    ["ArrowUp", "New file"],
    ["ArrowDown", "Open"],
    ["End", "Share"],
    ["ArrowDown", "Share"],
    ["Home", "New file"],
    ["PageDown", "Share"],
    ["PageUp", "New file"],
  ];
  for (const [key, expected] of moves) {
    await press(key);
    assert.equal(focusedText(), expected, `${key} should focus ${expected}`);
  }
  handle.unmount();
});

test("loop wraps arrow navigation at both ends", async () => {
  const handle = mountMenu(MenuFixture, { props: { rootProps: { loop: true } } });
  trigger().focus();
  await press("ArrowDown");
  await press("ArrowUp");
  assert.equal(focusedText(), "Share");
  await press("ArrowDown");
  assert.equal(focusedText(), "New file");
  handle.unmount();
});

test("typeahead matches prefixes, cycles repeated letters, and honors textValue", async () => {
  const Typeahead = defineComponent({
    setup: () => () =>
      h("div", [
        h(MenuRoot, null, () => [
          h(MenuTrigger, null, () => "Fruit"),
          h(MenuContent, null, () => [
            h(MenuItem, null, () => "Apple"),
            h(MenuItem, null, () => "Apricot"),
            h(MenuItem, null, () => "Banana"),
            h(MenuItem, { textValue: "Cherry" }, () => "🍒"),
            h(MenuItem, null, () => "Blue berry"),
          ]),
        ]),
      ]),
  });
  const handle = mountMenu(Typeahead);
  trigger().focus();
  await press("ArrowDown");
  await press("b");
  assert.equal(focusedText(), "Banana");
  await press("b");
  assert.equal(focusedText(), "Blue berry", "repeating a letter cycles matches");
  await new Promise((resolve) => setTimeout(resolve, 550));
  await press("c");
  assert.equal(focusedText(), "🍒", "textValue drives matching");
  await new Promise((resolve) => setTimeout(resolve, 550));
  await press("a");
  await press("p");
  await press("r");
  assert.equal(focusedText(), "Apricot", "multi-character prefixes");
  handle.unmount();
});

test("Space extends a pending typeahead query instead of selecting", async () => {
  const log: string[] = [];
  const handle = mountMenu(MenuFixture, { props: { log } });
  trigger().focus();
  await press("ArrowDown");
  await press("n");
  assert.equal(focusedText(), "New file");
  await press(" ");
  assert.deepEqual(log, ["open:true"]);
  await new Promise((resolve) => setTimeout(resolve, 550));
  await press(" ");
  assert.deepEqual(log, ["open:true", "select:new", "open:false"]);
  handle.unmount();
});

test("Tab closes the whole menu tree and returns focus to the trigger", async () => {
  const handle = mountMenu(MenuFixture);
  trigger().focus();
  await press("ArrowUp");
  await press("ArrowRight");
  assert.equal(openMenus().length, 2);
  const event = keydown(document.activeElement ?? document.body, "Tab");
  await settle();
  assert.equal(event.defaultPrevented, true);
  assert.equal(openMenus().length, 0);
  assert.equal(isTrigger(), true);
  handle.unmount();
});

test("submenu keys: ArrowRight/Enter open, ArrowLeft/Escape close to the sub trigger", async () => {
  const handle = mountMenu(MenuFixture);
  trigger().focus();
  await press("ArrowUp");
  assert.equal(focusedText(), "Share");
  const shareTrigger = document.activeElement;
  assert.equal(shareTrigger?.getAttribute("aria-haspopup"), "menu");
  assert.equal(shareTrigger?.getAttribute("aria-expanded"), "false");

  await press("ArrowRight");
  assert.equal(openMenus().length, 2);
  assert.equal(focusedText(), "Email");
  assert.equal(shareTrigger?.getAttribute("aria-expanded"), "true");
  const sub = one('[data-vize-ui="menu-sub-content"]');
  assert.equal(sub.getAttribute("aria-labelledby"), shareTrigger?.id);
  assert.equal(sub.getAttribute("data-side"), "right");

  await press("ArrowLeft");
  assert.equal(openMenus().length, 1);
  assert.equal(focusedText(), "Share");

  await press("Enter");
  assert.equal(focusedText(), "Email");
  await press("Escape");
  assert.equal(openMenus().length, 1, "Escape closes only the submenu");
  assert.equal(focusedText(), "Share");

  await press(" ");
  assert.equal(focusedText(), "Email");
  await press("ArrowDown");
  assert.equal(focusedText(), "Copy link");
  handle.unmount();
});

test("rtl flips submenu arrow keys and placement", async () => {
  const handle = mountMenu(MenuFixture, { props: { rootProps: { dir: "rtl" } } });
  trigger().focus();
  await press("ArrowUp");
  await press("ArrowRight");
  assert.equal(openMenus().length, 1, "ArrowRight does not open a submenu in rtl");
  await press("ArrowLeft");
  assert.equal(openMenus().length, 2);
  assert.equal(focusedText(), "Email");
  const sub = one('[data-vize-ui="menu-sub-content"]');
  assert.equal(sub.getAttribute("dir"), "rtl");
  assert.equal(sub.getAttribute("data-side"), "left");
  await press("ArrowRight");
  assert.equal(openMenus().length, 1);
  assert.equal(focusedText(), "Share");
  handle.unmount();
});

test("disabled submenu triggers stay focusable but do not open", async () => {
  const Disabled = defineComponent({
    setup: () => () =>
      h("div", [
        h(MenuRoot, null, () => [
          h(MenuTrigger, null, () => "Menu"),
          h(MenuContent, null, () => [
            h(MenuItem, null, () => "First"),
            h(MenuSub, { disabled: true }, () => [
              h(MenuSubTrigger, null, () => "More"),
              h(MenuSubContent, null, () => h(MenuItem, null, () => "Hidden")),
            ]),
          ]),
        ]),
      ]),
  });
  const handle = mountMenu(Disabled);
  trigger().focus();
  await press("ArrowUp");
  assert.equal(focusedText(), "More");
  assert.equal(document.activeElement?.getAttribute("aria-disabled"), "true");
  await press("ArrowRight");
  await press("Enter");
  assert.equal(openMenus().length, 1);
  handle.unmount();
});
