import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h, ref } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";

import type { MenuContentExpose, MenuRootExpose, MenuSelectEvent } from "./menu.ts";
import MenuArrow from "./menu-arrow.vue";
import MenuCheckboxItem from "./menu-checkbox-item.vue";
import MenuContent from "./menu-content.vue";
import { MenuFixture } from "./menu-fixture.ts";
import MenuGroup from "./menu-group.vue";
import MenuItem from "./menu-item.vue";
import MenuItemIndicator from "./menu-item-indicator.vue";
import MenuLabel from "./menu-label.vue";
import MenuRadioGroup from "./menu-radio-group.vue";
import MenuRadioItem from "./menu-radio-item.vue";
import MenuRoot from "./menu-root.vue";
import MenuSeparator from "./menu-separator.vue";
import MenuTrigger from "./menu-trigger.vue";
import {
  createMenuMounter,
  focusedText,
  item,
  keydown,
  maybe,
  mouseClick,
  one,
  openMenus,
  pointer,
  press,
  settle,
} from "./menu-test-utils.ts";

const { cleanup, mountMenu } = createMenuMounter(mountInteraction);

afterEach(cleanup);

function trigger(): HTMLElement {
  return one('[data-vize-ui="menu-trigger"]');
}

test("renders menu-button ARIA, deterministic ids, and pointer-opened content", async () => {
  const handle = mountMenu(MenuFixture);
  const button = trigger();
  assert.equal(button.id, "actions-trigger");
  assert.equal(button.getAttribute("aria-haspopup"), "menu");
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(button.getAttribute("aria-controls"), null);
  assert.equal(maybe('[role="menu"]'), null);

  await mouseClick(button);
  const menu = one('[role="menu"]');
  assert.equal(button.getAttribute("aria-expanded"), "true");
  assert.equal(button.getAttribute("aria-controls"), "actions-content");
  assert.equal(button.getAttribute("data-state"), "open");
  assert.equal(menu.id, "actions-content");
  assert.equal(menu.getAttribute("aria-labelledby"), "actions-trigger");
  assert.equal(menu.getAttribute("aria-orientation"), "vertical");
  assert.equal(menu.getAttribute("data-vize-ui"), "menu-content");
  assert.equal(menu.getAttribute("data-menu-kind"), "menu");
  assert.equal(menu.getAttribute("data-side"), "bottom");
  assert.equal(menu.getAttribute("data-align"), "start");
  assert.equal(menu.getAttribute("dir"), "ltr");
  // Pointer opening focuses the menu itself with nothing highlighted.
  assert.equal(document.activeElement, menu);
  assert.equal(menu.querySelector("[data-highlighted]"), null);

  const group = one('[data-vize-ui="menu-group"]');
  const label = one('[data-vize-ui="menu-label"]');
  assert.equal(group.getAttribute("role"), "group");
  assert.equal(group.getAttribute("aria-labelledby"), label.id);
  assert.equal(item("New file").getAttribute("role"), "menuitem");
  assert.equal(item("New file").getAttribute("tabindex"), "-1");
  assert.equal(item("Show hidden").getAttribute("role"), "menuitemcheckbox");
  assert.equal(item("•Small").getAttribute("role"), "menuitemradio");
  assert.equal(one('[data-vize-ui="menu-separator"]').getAttribute("role"), "separator");
  assert.equal(one('[data-vize-ui="menu-arrow"]').getAttribute("aria-hidden"), "true");
  handle.unmount();
});

test("selecting an item emits a preventable select, closes the tree, and returns focus", async () => {
  const log: string[] = [];
  const handle = mountMenu(MenuFixture, { props: { log } });
  trigger().focus();
  await press("Enter");
  assert.equal(focusedText(), "New file");
  await mouseClick(item("Open"));
  assert.deepEqual(log, ["open:true", "select:open", "open:false"]);
  assert.equal(openMenus().length, 0);
  assert.equal(document.activeElement, trigger());
  handle.unmount();

  const kept: string[] = [];
  const prevented = mountMenu(MenuFixture, { props: { log: kept, preventSelect: true } });
  await mouseClick(trigger());
  await mouseClick(item("Open"));
  assert.deepEqual(kept, ["open:true", "select:open"]);
  assert.equal(openMenus().length, 1);
  prevented.unmount();
});

test("disabled items stay focusable, announce aria-disabled, and never select", async () => {
  const log: string[] = [];
  const handle = mountMenu(MenuFixture, { props: { log } });
  trigger().focus();
  await press("ArrowDown");
  await press("ArrowDown");
  await press("ArrowDown");
  const disabled = item("Delete");
  assert.equal(document.activeElement, disabled);
  assert.equal(disabled.getAttribute("aria-disabled"), "true");
  assert.equal(disabled.getAttribute("data-disabled"), "true");
  assert.equal(disabled.getAttribute("data-highlighted"), "true");
  await press("Enter");
  await mouseClick(disabled);
  assert.deepEqual(log, ["open:true"]);
  assert.equal(openMenus().length, 1);
  handle.unmount();
});

test("checkbox items toggle aria-checked, indicators, and honor closeOnSelect", async () => {
  const log: string[] = [];
  const handle = mountMenu(MenuFixture, { props: { log } });
  await mouseClick(trigger());
  const checkbox = item("Show hidden");
  const indicator = checkbox.querySelector('[data-vize-ui="menu-item-indicator"]');
  assert.ok(indicator instanceof HTMLElement);
  assert.equal(checkbox.getAttribute("aria-checked"), "false");
  assert.equal(indicator.hidden, true);
  assert.equal(indicator.textContent, "");
  await mouseClick(checkbox);
  assert.equal(checkbox.getAttribute("aria-checked"), "true");
  assert.equal(checkbox.getAttribute("data-state"), "checked");
  assert.equal(indicator.hidden, false);
  assert.equal(indicator.textContent, "✓");
  assert.deepEqual(log, ["open:true", "hidden:true"]);
  assert.equal(openMenus().length, 1, "closeOnSelect=false keeps the menu open");
  handle.unmount();

  const Indeterminate = defineComponent({
    setup: () => () =>
      h("div", [
        h(MenuRoot, { defaultOpen: true }, () => [
          h(MenuTrigger, null, () => "Open"),
          h(MenuContent, null, () =>
            h(MenuCheckboxItem, { defaultValue: "indeterminate" }, () => [
              h(MenuItemIndicator, null, ({ state }: { state: string }) => state),
              "Mixed",
            ]),
          ),
        ]),
      ]),
  });
  const mixed = mountMenu(Indeterminate);
  await settle();
  const box = item("indeterminateMixed");
  assert.equal(box.getAttribute("aria-checked"), "mixed");
  assert.equal(box.getAttribute("data-state"), "indeterminate");
  mixed.unmount();
});

test("radio groups select generic values with custom equality", async () => {
  interface Theme {
    readonly id: string;
    readonly label: string;
  }
  const themes: readonly Theme[] = [
    { id: "light", label: "Light" },
    { id: "dark", label: "Dark" },
  ];
  const selected = ref<Theme | null>({ id: "dark", label: "Dark (copy)" });
  const changes: string[] = [];
  const Themes = defineComponent({
    setup: () => () =>
      h("div", [
        h(MenuRoot, { defaultOpen: true }, () => [
          h(MenuTrigger, null, () => "Theme"),
          h(MenuContent, null, () =>
            h(
              MenuRadioGroup<Theme>,
              {
                modelValue: selected.value,
                equals: (left: Theme, right: Theme) => left.id === right.id,
                "onUpdate:modelValue": (value: Theme | null) => {
                  selected.value = value;
                },
                "onValue-change": (value: Theme | null, previous: Theme | null) =>
                  changes.push(`${previous?.id}->${value?.id}`),
              },
              () =>
                themes.map((theme) =>
                  h(MenuRadioItem<Theme>, { key: theme.id, value: theme }, () => theme.label),
                ),
            ),
          ),
        ]),
      ]),
  });
  const handle = mountMenu(Themes);
  await settle();
  assert.equal(item("Dark").getAttribute("aria-checked"), "true");
  assert.equal(item("Light").getAttribute("aria-checked"), "false");
  await mouseClick(item("Light"));
  assert.equal(selected.value?.id, "light");
  assert.deepEqual(changes, ["dark->light"]);
  assert.equal(openMenus().length, 0);
  handle.unmount();
});

test("controlled open emits requests without opening until the parent accepts", async () => {
  const handle = mountMenu(MenuFixture, { props: { rootProps: { open: false } } });
  await mouseClick(trigger());
  assert.equal(openMenus().length, 0);
  assert.equal(trigger().getAttribute("aria-expanded"), "false");
  await handle.wrapper.setProps({ rootProps: { open: true } });
  await settle();
  assert.equal(openMenus().length, 1);
  handle.unmount();
});

test("outside pointer-down closes without stealing focus; Escape closes and returns focus", async () => {
  const log: string[] = [];
  const handle = mountMenu(MenuFixture, { props: { log } });
  await mouseClick(trigger());
  const outside = one('[data-testid="after"]');
  pointer(document.body, "pointerdown");
  await settle();
  assert.equal(openMenus().length, 0);
  assert.notEqual(document.activeElement, trigger());

  trigger().focus();
  await press("ArrowDown");
  keydown(document.activeElement ?? document.body, "Escape");
  await settle();
  assert.equal(openMenus().length, 0);
  assert.equal(document.activeElement, trigger());
  assert.ok(outside);
  handle.unmount();

  const sticky = mountMenu(MenuFixture, {
    props: { contentProps: { closeOnEscape: false, closeOnPointerDownOutside: false } },
  });
  await mouseClick(trigger());
  keydown(one('[role="menu"]'), "Escape");
  pointer(document.body, "pointerdown");
  await settle();
  assert.equal(openMenus().length, 1);
  sticky.unmount();
});

test("modal menus inert outside content; non-modal menus leave the page interactive", async () => {
  const modal = mountMenu(MenuFixture);
  await mouseClick(trigger());
  const before = one('[data-testid="before"]');
  assert.equal(before.hasAttribute("inert"), true);
  assert.equal(trigger().hasAttribute("inert"), false, "the trigger stays a live branch");
  assert.equal(one('[role="menu"]').getAttribute("data-modal"), "true");
  pointer(document.body, "pointerdown");
  await settle();
  assert.equal(before.hasAttribute("inert"), false);
  modal.unmount();

  const nonModal = mountMenu(MenuFixture, { props: { rootProps: { modal: false } } });
  await mouseClick(trigger());
  assert.equal(one('[data-testid="before"]').hasAttribute("inert"), false);
  assert.equal(one('[role="menu"]').getAttribute("data-modal"), "false");
  nonModal.unmount();
});

test("disabled roots refuse to open and close when disabled while open", async () => {
  const handle = mountMenu(MenuFixture, { props: { rootProps: { disabled: true } } });
  await mouseClick(trigger());
  assert.equal(openMenus().length, 0);
  await handle.wrapper.setProps({ rootProps: { disabled: false, defaultOpen: true } });
  await mouseClick(trigger());
  assert.equal(openMenus().length, 1);
  await handle.wrapper.setProps({ rootProps: { disabled: true } });
  await settle();
  assert.equal(openMenus().length, 0);
  handle.unmount();
});

test("force-mounted closed content stays hidden and inactive", async () => {
  const handle = mountMenu(MenuFixture, { props: { contentProps: { forceMount: true } } });
  await settle();
  const menu = one('[data-vize-ui="menu-content"]');
  assert.equal(menu.hidden, true);
  assert.equal(menu.getAttribute("data-state"), "closed");
  assert.equal(one('[data-testid="before"]').hasAttribute("inert"), false);
  handle.unmount();
});

test("exposes root and content imperative APIs", async () => {
  const rootRef = ref<MenuRootExpose | null>(null);
  const contentRef = ref<MenuContentExpose | null>(null);
  const selects: MenuSelectEvent[] = [];
  const Imperative = defineComponent({
    setup: () => () =>
      h("div", [
        h(MenuRoot, { ref: rootRef, id: "imperative" }, () => [
          h(MenuTrigger, null, () => "Menu"),
          h(MenuContent, { ref: contentRef }, () => [
            h(MenuItem, { onSelect: (event: MenuSelectEvent) => selects.push(event) }, () => "One"),
            h(MenuItem, null, () => "Two"),
            h(MenuSeparator),
            h(MenuGroup, null, () => h(MenuLabel, { id: "custom-label" }, () => "Label")),
            h(MenuArrow),
          ]),
        ]),
      ]),
  });
  const handle = mountMenu(Imperative);
  const root = rootRef.value;
  assert.ok(root);
  assert.equal(root.id, "imperative");
  assert.equal(root.triggerId, "imperative-trigger");
  assert.equal(root.open, false);
  root.openMenu("last");
  await settle();
  assert.equal(root.open, true);
  assert.equal(focusedText(), "Two");
  contentRef.value?.focusFirst();
  assert.equal(focusedText(), "One");
  contentRef.value?.focusContent();
  assert.equal(document.activeElement, one('[role="menu"]'));
  assert.equal(one("#custom-label").textContent, "Label");
  root.close();
  await settle();
  assert.equal(root.open, false);
  root.toggle();
  await settle();
  assert.equal(root.open, true);
  assert.equal(selects.length, 0);
  handle.unmount();
});
