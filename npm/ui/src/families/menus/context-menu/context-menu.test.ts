import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h, ref } from "vue";
import type { PropType } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";

import { ContextMenuContent, ContextMenuItem } from "./context-menu.ts";
import type { ContextMenuRootExpose } from "./context-menu.ts";
import ContextMenuRoot from "./context-menu-root.vue";
import ContextMenuTrigger from "./context-menu-trigger.vue";
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
  stubRect,
  wait,
} from "../menu/menu-test-utils.ts";

const { cleanup, mountMenu } = createMenuMounter(mountInteraction);

afterEach(cleanup);

const rootRef = ref<ContextMenuRootExpose | null>(null);

const Fixture = defineComponent({
  props: {
    log: { type: Array as PropType<string[]>, default: () => [] },
    rootProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    triggerProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
  },
  setup: (props) => () =>
    h("div", [
      h("button", { type: "button", "data-testid": "outside" }, "Outside"),
      h(
        ContextMenuRoot,
        {
          ref: rootRef,
          id: "canvas",
          ...props.rootProps,
          "onUpdate:open": (value: boolean) => props.log.push(`open:${value}`),
        },
        () => [
          h(ContextMenuTrigger, { longPressDelay: 20, ...props.triggerProps }, () =>
            h("button", { type: "button", "data-testid": "target" }, "Canvas"),
          ),
          h(ContextMenuContent, { ariaLabel: "Canvas actions", offset: 0 }, () => [
            h(ContextMenuItem, { onSelect: () => props.log.push("select:cut") }, () => "Cut"),
            h(ContextMenuItem, null, () => "Copy"),
          ]),
        ],
      ),
    ]),
});

function region(): HTMLElement {
  return one('[data-vize-ui="context-menu-trigger"]');
}

function contextmenu(target: Element, x: number, y: number): MouseEvent {
  const event = new MouseEvent("contextmenu", {
    bubbles: true,
    cancelable: true,
    button: 2,
    clientX: x,
    clientY: y,
  });
  target.dispatchEvent(event);
  return event;
}

test("contextmenu opens at the pointer, prevents the native menu, and focuses the menu", async () => {
  const log: string[] = [];
  mountMenu(Fixture, { props: { log } });
  const event = contextmenu(one('[data-testid="target"]'), 40, 60);
  await settle();
  assert.equal(event.defaultPrevented, true);
  assert.equal(openMenus().length, 1);
  const menu = one('[role="menu"]');
  assert.equal(menu.getAttribute("data-menu-kind"), "context-menu");
  assert.equal(menu.getAttribute("aria-label"), "Canvas actions");
  assert.equal(menu.getAttribute("aria-labelledby"), null);
  assert.equal(region().getAttribute("data-state"), "open");
  assert.equal(document.activeElement, menu);
  assert.deepEqual(log, ["open:true"]);
});

test("a new request while open re-anchors the same open menu", async () => {
  const log: string[] = [];
  mountMenu(Fixture, { props: { log } });
  contextmenu(region(), 40, 60);
  await settle();
  const first = one('[role="menu"]');
  pointer(document.body, "pointerdown", { button: 2 });
  contextmenu(region(), 120, 10);
  await settle();
  assert.equal(openMenus().length, 1);
  assert.equal(one('[role="menu"]').id, first.id);
  assert.deepEqual(log, ["open:true", "open:false", "open:true"]);
});

test("Shift+F10 and the ContextMenu key open at the focused element with the first item focused", async () => {
  for (const init of [{ key: "F10", shiftKey: true }, { key: "ContextMenu" }]) {
    const handle = mountMenu(Fixture);
    const target = one('[data-testid="target"]');
    stubRect(target, { x: 10, y: 20, width: 80, height: 30 });
    target.focus();
    const event = keydown(target, init.key, init);
    await settle();
    assert.equal(event.defaultPrevented, true);
    assert.equal(focusedText(), "Cut");
    keydown(document.activeElement ?? document.body, "Escape");
    await settle();
    assert.equal(openMenus().length, 0);
    assert.equal(document.activeElement, target, "focus returns to the requesting element");
    handle.unmount();
  }
});

test("plain F10 and other keys do not open the menu", async () => {
  mountMenu(Fixture);
  keydown(one('[data-testid="target"]'), "F10");
  keydown(one('[data-testid="target"]'), "Enter");
  await settle();
  assert.equal(openMenus().length, 0);
});

test("a touch long press opens at the touch point; short taps do not", async () => {
  mountMenu(Fixture);
  pointer(region(), "pointerdown", { pointerType: "touch", clientX: 30, clientY: 30 });
  pointer(region(), "pointerup", { pointerType: "touch", clientX: 30, clientY: 30 });
  await wait(40);
  assert.equal(openMenus().length, 0);
  pointer(region(), "pointerdown", { pointerType: "touch", clientX: 30, clientY: 30 });
  await wait(40);
  assert.equal(openMenus().length, 1);
  pointer(region(), "pointerup", { pointerType: "touch", clientX: 30, clientY: 30 });
});

test("mouse presses never trigger the long-press path", async () => {
  mountMenu(Fixture);
  pointer(region(), "pointerdown", { clientX: 30, clientY: 30 });
  await wait(40);
  assert.equal(openMenus().length, 0);
  pointer(region(), "pointerup");
});

test("disabled triggers and roots let the native menu show", async () => {
  mountMenu(Fixture, { props: { triggerProps: { disabled: true } } });
  const event = contextmenu(region(), 5, 5);
  await settle();
  assert.equal(event.defaultPrevented, false);
  assert.equal(openMenus().length, 0);
  assert.equal(region().getAttribute("data-disabled"), "true");
  cleanup();

  mountMenu(Fixture, { props: { rootProps: { disabled: true } } });
  assert.equal(contextmenu(region(), 5, 5).defaultPrevented, false);
});

test("a prevented contextmenu emit keeps the native menu", async () => {
  mountMenu(Fixture, {
    props: { triggerProps: { onContextmenu: (event: MouseEvent) => event.preventDefault() } },
  });
  contextmenu(region(), 5, 5);
  await settle();
  assert.equal(openMenus().length, 0);
});

test("selecting closes; outside pointer-down closes without restoring focus", async () => {
  const log: string[] = [];
  mountMenu(Fixture, { props: { log } });
  contextmenu(region(), 5, 5);
  await settle();
  await mouseClick(item("Cut"));
  assert.deepEqual(log, ["open:true", "select:cut", "open:false"]);
  contextmenu(region(), 5, 5);
  await settle();
  pointer(document.body, "pointerdown");
  await settle();
  assert.equal(openMenus().length, 0);
});

test("exposes openAt for programmatic requests", async () => {
  mountMenu(Fixture);
  rootRef.value?.openAt({ x: 1, y: 2 });
  await settle();
  assert.equal(focusedText(), "Cut");
  rootRef.value?.close();
  await settle();
  assert.equal(openMenus().length, 0);
});
