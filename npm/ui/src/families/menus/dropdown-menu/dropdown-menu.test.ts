import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h } from "vue";
import type { PropType } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";

import {
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
} from "./dropdown-menu.ts";
import DropdownMenuRoot from "./dropdown-menu-root.vue";
import DropdownMenuTrigger from "./dropdown-menu-trigger.vue";
import {
  createMenuMounter,
  focusedText,
  item,
  keydown,
  mouseClick,
  one,
  openMenus,
  pointer,
  settle,
} from "../menu/menu-test-utils.ts";

const { cleanup, mountMenu } = createMenuMounter(mountInteraction);

afterEach(cleanup);

const Fixture = defineComponent({
  props: {
    log: { type: Array as PropType<string[]>, default: () => [] },
    triggerProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    rootProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
  },
  setup: (props) => () =>
    h("div", [
      h(
        DropdownMenuRoot,
        {
          id: "file",
          ...props.rootProps,
          "onUpdate:open": (value: boolean) => props.log.push(`open:${value}`),
        },
        () => [
          h(DropdownMenuTrigger, { ...props.triggerProps }, () => "File"),
          h(DropdownMenuContent, null, () => [
            h(DropdownMenuItem, { onSelect: () => props.log.push("select:new") }, () => "New"),
            h(DropdownMenuItem, null, () => "Save"),
            h(DropdownMenuSub, null, () => [
              h(DropdownMenuSubTrigger, null, () => "Export"),
              h(DropdownMenuSubContent, null, () => h(DropdownMenuItem, null, () => "PDF")),
            ]),
          ]),
        ],
      ),
    ]),
});

function trigger(): HTMLElement {
  return one('[data-vize-ui="dropdown-menu-trigger"]');
}

test("mouse pointer-down toggles the menu and focuses the content", async () => {
  const log: string[] = [];
  mountMenu(Fixture, { props: { log } });
  const down = pointer(trigger(), "pointerdown");
  await settle();
  assert.equal(down.defaultPrevented, true, "opening keeps focus off the trigger");
  assert.equal(openMenus().length, 1);
  const menu = one('[role="menu"]');
  assert.equal(menu.getAttribute("data-menu-kind"), "dropdown-menu");
  assert.equal(menu.getAttribute("aria-labelledby"), "file-trigger");
  assert.equal(document.activeElement, menu);
  // The following click belongs to the same press and must not toggle again.
  await mouseClick(trigger());
  assert.equal(openMenus().length, 1);
  pointer(trigger(), "pointerdown");
  await settle();
  assert.equal(openMenus().length, 0);
  assert.deepEqual(log, ["open:true", "open:false"]);
});

test("secondary buttons and ctrl-clicks do not toggle; touch taps toggle on click", async () => {
  mountMenu(Fixture);
  pointer(trigger(), "pointerdown", { button: 2 });
  pointer(trigger(), "pointerdown", { ctrlKey: true });
  await settle();
  assert.equal(openMenus().length, 0);
  pointer(trigger(), "pointerdown", { pointerType: "touch" });
  pointer(trigger(), "pointerup", { pointerType: "touch" });
  await mouseClick(trigger());
  assert.equal(openMenus().length, 1);
});

test("openOn click waits for the click", async () => {
  mountMenu(Fixture, { props: { triggerProps: { openOn: "click" } } });
  pointer(trigger(), "pointerdown");
  await settle();
  assert.equal(openMenus().length, 0);
  await mouseClick(trigger());
  assert.equal(openMenus().length, 1);
});

test("keyboard and assistive-technology activation focus the first or last item", async () => {
  mountMenu(Fixture);
  trigger().focus();
  keydown(trigger(), "ArrowUp");
  await settle();
  assert.equal(focusedText(), "Export");
  keydown(document.activeElement ?? document.body, "Escape");
  await settle();
  assert.equal(document.activeElement, trigger());
  trigger().click();
  await settle();
  assert.equal(focusedText(), "New", "a detail-0 click is a keyboard activation");
});

test("preventable pointerdown and keydown emits keep the menu closed", async () => {
  mountMenu(Fixture, {
    props: {
      triggerProps: {
        onPointerdown: (event: PointerEvent) => event.preventDefault(),
        onKeydown: (event: KeyboardEvent) => event.preventDefault(),
      },
    },
  });
  pointer(trigger(), "pointerdown");
  keydown(trigger(), "Enter");
  await settle();
  assert.equal(openMenus().length, 0);
});

test("disabled triggers ignore every activation", async () => {
  mountMenu(Fixture, { props: { triggerProps: { disabled: true } } });
  assert.equal(trigger().hasAttribute("disabled"), true);
  pointer(trigger(), "pointerdown");
  keydown(trigger(), "ArrowDown");
  await mouseClick(trigger());
  assert.equal(openMenus().length, 0);
});

test("items and submenus are the shared menu parts", async () => {
  const log: string[] = [];
  mountMenu(Fixture, { props: { log } });
  trigger().focus();
  keydown(trigger(), "Enter");
  await settle();
  keydown(document.activeElement ?? document.body, "End");
  keydown(document.activeElement ?? document.body, "ArrowRight");
  await settle();
  assert.equal(focusedText(), "PDF");
  await mouseClick(item("PDF"));
  assert.equal(openMenus().length, 0);
  assert.equal(document.activeElement, trigger());
});
