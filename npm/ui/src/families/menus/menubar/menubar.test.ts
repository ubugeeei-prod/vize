import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h, ref } from "vue";
import type { PropType } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";

import {
  MenubarContent,
  MenubarItem,
  MenubarSub,
  MenubarSubContent,
  MenubarSubTrigger,
} from "./menubar.ts";
import type { MenubarRootExpose } from "./menubar.ts";
import MenubarMenu from "./menubar-menu.vue";
import MenubarRoot from "./menubar-root.vue";
import MenubarTrigger from "./menubar-trigger.vue";
import {
  createMenuMounter,
  focusedText,
  item,
  keydown,
  mouseClick,
  one,
  openMenus,
  pointer,
  press,
  settle,
} from "../menu/menu-test-utils.ts";

const { cleanup, mountMenu } = createMenuMounter(mountInteraction);

afterEach(cleanup);

const rootRef = ref<MenubarRootExpose | null>(null);

const Fixture = defineComponent({
  props: {
    log: { type: Array as PropType<string[]>, default: () => [] },
    rootProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    editDisabled: { type: Boolean, default: false },
  },
  setup: (props) => () =>
    h("div", [
      h(
        MenubarRoot,
        {
          ref: rootRef,
          ariaLabel: "Application",
          ...props.rootProps,
          "onUpdate:modelValue": (value: string | null) => props.log.push(`value:${value}`),
        },
        () => [
          h(MenubarMenu, { value: "file" }, () => [
            h(MenubarTrigger, null, () => "File"),
            h(MenubarContent, null, () => [
              h(MenubarItem, null, () => "New"),
              h(MenubarSub, null, () => [
                h(MenubarSubTrigger, null, () => "Recent"),
                h(MenubarSubContent, null, () => h(MenubarItem, null, () => "notes.txt")),
              ]),
            ]),
          ]),
          h(MenubarMenu, { value: "edit", disabled: props.editDisabled }, () => [
            h(MenubarTrigger, null, () => "Edit"),
            h(MenubarContent, null, () => [
              h(MenubarItem, null, () => "Undo"),
              h(MenubarItem, null, () => "Redo"),
            ]),
          ]),
          h(MenubarMenu, { value: "view" }, () => [
            h(MenubarTrigger, null, () => "View"),
            h(MenubarContent, null, () => h(MenubarItem, null, () => "Zoom")),
          ]),
        ],
      ),
    ]),
});

function trigger(value: string): HTMLElement {
  return one(`[data-vize-ui="menubar-trigger"][data-value="${value}"]`);
}

function tabStops(): string[] {
  return [...document.querySelectorAll('[data-vize-ui="menubar-trigger"]')]
    .filter((element) => element.getAttribute("tabindex") === "0")
    .map((element) => element.textContent ?? "");
}

test("renders a labelled horizontal menubar with a single roving tab stop", async () => {
  mountMenu(Fixture);
  await settle();
  const bar = one('[role="menubar"]');
  assert.equal(bar.getAttribute("aria-orientation"), "horizontal");
  assert.equal(bar.getAttribute("aria-label"), "Application");
  assert.equal(trigger("file").getAttribute("role"), "menuitem");
  assert.equal(trigger("file").getAttribute("aria-haspopup"), "menu");
  assert.deepEqual(tabStops(), ["File"]);
});

test("Left/Right move the roving focus with wrapping, Home/End jump", async () => {
  mountMenu(Fixture);
  await settle();
  trigger("file").focus();
  await press("ArrowRight");
  assert.equal(focusedText(), "Edit");
  assert.deepEqual(tabStops(), ["Edit"]);
  await press("End");
  assert.equal(focusedText(), "View");
  await press("ArrowRight");
  assert.equal(focusedText(), "File", "menubars wrap by default");
  await press("ArrowLeft");
  assert.equal(focusedText(), "View");
  await press("Home");
  assert.equal(focusedText(), "File");
  await press("v");
  assert.equal(focusedText(), "View", "typeahead across triggers");
  assert.equal(openMenus().length, 0);
});

test("rtl flips Left/Right across triggers", async () => {
  mountMenu(Fixture, { props: { rootProps: { dir: "rtl" } } });
  await settle();
  trigger("file").focus();
  await press("ArrowLeft");
  assert.equal(focusedText(), "Edit");
});

test("ArrowDown/Enter/Space open with the first item; ArrowUp with the last", async () => {
  const log: string[] = [];
  mountMenu(Fixture, { props: { log } });
  await settle();
  trigger("edit").focus();
  await press("ArrowDown");
  assert.equal(focusedText(), "Undo");
  assert.equal(trigger("edit").getAttribute("aria-expanded"), "true");
  assert.equal(one('[role="menu"]').getAttribute("data-menu-kind"), "menubar");
  await press("Escape");
  assert.equal(openMenus().length, 0);
  assert.equal(focusedText(), "Edit");
  await press("ArrowUp");
  assert.equal(focusedText(), "Redo");
  await press("Escape");
  await press("Enter");
  assert.equal(focusedText(), "Undo");
  assert.deepEqual(log, ["value:edit", "value:null", "value:edit", "value:null", "value:edit"]);
});

test("Right/Left inside an open menu hand off to the adjacent menu", async () => {
  mountMenu(Fixture);
  await settle();
  trigger("file").focus();
  await press("ArrowDown");
  assert.equal(focusedText(), "New");
  await press("ArrowRight");
  assert.equal(openMenus().length, 1);
  assert.equal(focusedText(), "Undo");
  assert.equal(trigger("edit").getAttribute("data-state"), "open");
  assert.equal(trigger("file").getAttribute("data-state"), "closed");
  await press("ArrowLeft");
  assert.equal(focusedText(), "New");
  await press("ArrowLeft");
  assert.equal(focusedText(), "Zoom", "hand-off wraps");
});

test("a submenu trigger consumes Right; Left closes the submenu before handing off", async () => {
  mountMenu(Fixture);
  await settle();
  trigger("file").focus();
  await press("ArrowDown");
  await press("ArrowDown");
  assert.equal(focusedText(), "Recent");
  await press("ArrowRight");
  assert.equal(focusedText(), "notes.txt");
  await press("ArrowRight");
  assert.equal(focusedText(), "Undo", "Right on a plain submenu item moves to the next menu");
  assert.equal(openMenus().length, 1);
});

test("Right on a trigger while a menu is open opens the next menu", async () => {
  mountMenu(Fixture);
  await settle();
  await mouseClick(trigger("file"));
  pointer(trigger("file"), "pointerdown");
  await settle();
  assert.equal(openMenus().length, 1);
  trigger("file").focus();
  keydown(trigger("file"), "ArrowRight");
  await settle();
  assert.equal(trigger("edit").getAttribute("data-state"), "open");
  assert.equal(focusedText(), "Undo");
});

test("pointer-down toggles a menu and hovering siblings switches the open menu", async () => {
  mountMenu(Fixture);
  await settle();
  pointer(trigger("view"), "pointerenter");
  await settle();
  assert.equal(openMenus().length, 0, "hover alone does not open a closed menubar");
  pointer(trigger("file"), "pointerdown");
  await settle();
  assert.equal(openMenus().length, 1);
  assert.equal(document.activeElement?.getAttribute("role"), "menu");
  pointer(trigger("view"), "pointerenter");
  await settle();
  assert.equal(trigger("view").getAttribute("data-state"), "open");
  assert.equal(trigger("file").getAttribute("data-state"), "closed");
  assert.equal(openMenus().length, 1);
  pointer(trigger("view"), "pointerdown");
  await settle();
  assert.equal(openMenus().length, 0);
});

test("menubar menus are non-modal and close on outside pointer-down", async () => {
  mountMenu(Fixture);
  await settle();
  pointer(trigger("file"), "pointerdown");
  await settle();
  assert.equal(trigger("edit").hasAttribute("inert"), false);
  pointer(document.body, "pointerdown");
  await settle();
  assert.equal(openMenus().length, 0);
});

test("disabled menus keep a focusable trigger that never opens", async () => {
  mountMenu(Fixture, { props: { editDisabled: true } });
  await settle();
  trigger("file").focus();
  await press("ArrowRight");
  assert.equal(focusedText(), "Edit");
  assert.equal(trigger("edit").getAttribute("aria-disabled"), "true");
  await press("ArrowDown");
  pointer(trigger("edit"), "pointerdown");
  await settle();
  assert.equal(openMenus().length, 0);
});

test("controlled value and the exposed API drive the open menu", async () => {
  const handle = mountMenu(Fixture, { props: { rootProps: { modelValue: "view" } } });
  await settle();
  assert.equal(trigger("view").getAttribute("aria-expanded"), "true");
  await mouseClick(item("Zoom"));
  assert.equal(openMenus().length, 1, "controlled value wins until the parent accepts");
  await handle.wrapper.setProps({ rootProps: { modelValue: null } });
  await settle();
  assert.equal(openMenus().length, 0);
  await handle.wrapper.setProps({ rootProps: {} });
  rootRef.value?.setValue("file");
  await settle();
  assert.equal(trigger("file").getAttribute("data-state"), "open");
  rootRef.value?.setValue(null);
  await settle();
  rootRef.value?.focus();
  assert.equal(focusedText(), "File");
});
