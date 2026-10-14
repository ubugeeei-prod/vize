import assert from "node:assert/strict";

import { afterEach, test } from "vite-plus/test";
import { defineComponent, h } from "vue";
import type { PropType } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
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
} from "../../menus/menu/menu-test-utils.ts";

import {
  SplitButtonContent,
  SplitButtonItem,
  SplitButtonSub,
  SplitButtonSubContent,
  SplitButtonSubTrigger,
} from "./split-button.ts";
import SplitButtonAction from "./split-button-action.vue";
import SplitButtonRoot from "./split-button-root.vue";
import SplitButtonTrigger from "./split-button-trigger.vue";

const { cleanup, mountMenu } = createMenuMounter(mountInteraction);

afterEach(cleanup);

const Fixture = defineComponent({
  props: {
    log: { type: Array as PropType<string[]>, default: () => [] },
    actionProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    triggerProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
    rootProps: { type: Object as PropType<Record<string, unknown>>, default: () => ({}) },
  },
  setup: (props) => () =>
    h("div", [
      h(
        SplitButtonRoot,
        {
          id: "save",
          ariaLabel: "Save options",
          ...props.rootProps,
          "onUpdate:open": (value: boolean) => props.log.push(`open:${value}`),
        },
        () => [
          h(
            SplitButtonAction,
            { ...props.actionProps, onPress: () => props.log.push("press") },
            () => "Save",
          ),
          h(SplitButtonTrigger, { ariaLabel: "More save options", ...props.triggerProps }),
          h(SplitButtonContent, null, () => [
            h(SplitButtonItem, { onSelect: () => props.log.push("select:draft") }, () => "Draft"),
            h(SplitButtonItem, null, () => "Schedule"),
            h(SplitButtonSub, null, () => [
              h(SplitButtonSubTrigger, null, () => "Export"),
              h(SplitButtonSubContent, null, () => h(SplitButtonItem, null, () => "PDF")),
            ]),
          ]),
        ],
      ),
    ]),
});

function group(): HTMLElement {
  return one('[data-vize-ui="split-button"]');
}

function action(): HTMLButtonElement {
  return one('[data-vize-ui="split-button-action"]') as HTMLButtonElement;
}

function trigger(): HTMLButtonElement {
  return one('[data-vize-ui="split-button-trigger"]') as HTMLButtonElement;
}

test("renders a named group with a primary action and a menu button", async () => {
  const log: string[] = [];
  mountMenu(Fixture, { props: { log } });
  assert.equal(group().getAttribute("role"), "group");
  assert.equal(group().getAttribute("aria-label"), "Save options");
  assert.equal(group().getAttribute("data-state"), "closed");
  assert.deepEqual(
    [...group().querySelectorAll("button")].map((button) => button.getAttribute("data-vize-ui")),
    ["split-button-action", "split-button-trigger"],
  );
  assert.equal(action().getAttribute("type"), "button");
  assert.equal(action().hasAttribute("aria-haspopup"), false);
  assert.equal(trigger().id, "save-trigger");
  assert.equal(trigger().getAttribute("aria-label"), "More save options");
  assert.equal(trigger().getAttribute("aria-haspopup"), "menu");
  assert.equal(trigger().getAttribute("aria-expanded"), "false");
  assert.equal(trigger().hasAttribute("aria-controls"), false);

  await mouseClick(action());
  assert.deepEqual(log, ["press"], "the primary action never opens the menu on press");
  assert.equal(openMenus().length, 0);
});

test("mouse pointer-down on the trigger toggles the menu and focuses the content", async () => {
  const log: string[] = [];
  mountMenu(Fixture, { props: { log } });
  const down = pointer(trigger(), "pointerdown");
  await settle();
  assert.equal(down.defaultPrevented, true, "opening keeps focus off the trigger");
  assert.equal(openMenus().length, 1);
  const menu = one('[role="menu"]');
  assert.equal(menu.getAttribute("data-menu-kind"), "split-button");
  assert.equal(menu.getAttribute("aria-labelledby"), "save-trigger");
  assert.equal(document.activeElement, menu);
  assert.equal(trigger().getAttribute("aria-expanded"), "true");
  assert.equal(trigger().getAttribute("aria-controls"), menu.id);
  assert.equal(group().getAttribute("data-state"), "open");
  assert.equal(action().getAttribute("data-state"), "open");
  // The following click belongs to the same press and must not toggle again.
  await mouseClick(trigger());
  assert.equal(openMenus().length, 1);
  pointer(trigger(), "pointerdown");
  await settle();
  assert.equal(openMenus().length, 0);
  assert.deepEqual(log, ["open:true", "open:false"]);
});

test("secondary buttons and ctrl-clicks do not toggle; touch taps and openOn click toggle on click", async () => {
  const handle = mountMenu(Fixture);
  pointer(trigger(), "pointerdown", { button: 2 });
  pointer(trigger(), "pointerdown", { ctrlKey: true });
  await settle();
  assert.equal(openMenus().length, 0);
  pointer(trigger(), "pointerdown", { pointerType: "touch" });
  pointer(trigger(), "pointerup", { pointerType: "touch" });
  await mouseClick(trigger());
  assert.equal(openMenus().length, 1);
  handle.unmount();

  mountMenu(Fixture, { props: { triggerProps: { openOn: "click" } } });
  pointer(trigger(), "pointerdown");
  await settle();
  assert.equal(openMenus().length, 0);
  await mouseClick(trigger());
  assert.equal(openMenus().length, 1);
});

test("trigger keyboard opens on the first or last item and Escape returns focus to it", async () => {
  mountMenu(Fixture);
  trigger().focus();
  for (const [key, expected] of [
    ["Enter", "Draft"],
    [" ", "Draft"],
    ["ArrowDown", "Draft"],
    ["ArrowUp", "Export"],
  ] as const) {
    const event = keydown(trigger(), key);
    await settle();
    assert.equal(event.defaultPrevented, true, `${key} is consumed`);
    assert.equal(focusedText(), expected, `${key} focuses ${expected}`);
    keydown(document.activeElement ?? document.body, "Escape");
    await settle();
    assert.equal(openMenus().length, 0);
    assert.equal(document.activeElement, trigger());
  }
  trigger().click();
  await settle();
  assert.equal(focusedText(), "Draft", "a detail-0 click is a keyboard activation");
});

test("Alt+ArrowDown on the primary action opens the menu and focus returns to the action", async () => {
  const log: string[] = [];
  mountMenu(Fixture, { props: { log } });
  action().focus();
  const plain = keydown(action(), "ArrowDown");
  await settle();
  assert.equal(plain.defaultPrevented, false, "plain ArrowDown stays with the page");
  assert.equal(openMenus().length, 0);

  const event = keydown(action(), "ArrowDown", { altKey: true });
  await settle();
  assert.equal(event.defaultPrevented, true);
  assert.equal(focusedText(), "Draft");
  keydown(document.activeElement ?? document.body, "Escape");
  await settle();
  assert.equal(openMenus().length, 0);
  assert.equal(document.activeElement, action(), "focus returns to the element that opened");

  // A later trigger-owned open returns focus to the trigger again.
  trigger().focus();
  keydown(trigger(), "Enter");
  await settle();
  keydown(document.activeElement ?? document.body, "Escape");
  await settle();
  assert.equal(document.activeElement, trigger());
  assert.deepEqual(log, ["open:true", "open:false", "open:true", "open:false"]);
});

test("preventable keydown and pointerdown emits keep the menu closed", async () => {
  mountMenu(Fixture, {
    props: {
      actionProps: { onKeydown: (event: KeyboardEvent) => event.preventDefault() },
      triggerProps: {
        onPointerdown: (event: PointerEvent) => event.preventDefault(),
        onKeydown: (event: KeyboardEvent) => event.preventDefault(),
      },
    },
  });
  keydown(action(), "ArrowDown", { altKey: true });
  pointer(trigger(), "pointerdown");
  keydown(trigger(), "Enter");
  await settle();
  assert.equal(openMenus().length, 0);
});

test("selecting an item closes every level and returns focus to the trigger", async () => {
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

  keydown(trigger(), "Enter");
  await settle();
  await mouseClick(item("Draft"));
  assert.equal(openMenus().length, 0);
  assert.equal(log.includes("select:draft"), true);
  assert.equal(log.includes("press"), false, "selecting an item is not the primary action");
});

test("a disabled root disables both buttons and closes an open menu", async () => {
  const log: string[] = [];
  const handle = mountMenu(Fixture, { props: { log, rootProps: { defaultOpen: true } } });
  await settle();
  assert.equal(openMenus().length, 1);
  await handle.wrapper.setProps({ rootProps: { defaultOpen: true, disabled: true } });
  await settle();
  assert.equal(openMenus().length, 0);
  assert.equal(group().getAttribute("aria-disabled"), "true");
  assert.equal(group().getAttribute("data-disabled"), "true");
  assert.equal(action().disabled, true);
  assert.equal(trigger().disabled, true);
  pointer(trigger(), "pointerdown");
  keydown(trigger(), "ArrowDown");
  keydown(action(), "ArrowDown", { altKey: true });
  await mouseClick(trigger());
  await mouseClick(action());
  assert.equal(openMenus().length, 0);
  assert.equal(log.includes("press"), false);
});

test("the action and the trigger can be disabled independently", async () => {
  const log: string[] = [];
  const handle = mountMenu(Fixture, { props: { log, actionProps: { disabled: true } } });
  assert.equal(action().disabled, true);
  assert.equal(action().getAttribute("data-disabled"), "true");
  assert.equal(trigger().disabled, false);
  keydown(action(), "ArrowDown", { altKey: true });
  await settle();
  assert.equal(openMenus().length, 0);
  keydown(trigger(), "Enter");
  await settle();
  assert.equal(openMenus().length, 1);
  handle.unmount();

  mountMenu(Fixture, { props: { log, triggerProps: { disabled: true } } });
  assert.equal(trigger().disabled, true);
  assert.equal(action().disabled, false);
  pointer(trigger(), "pointerdown");
  keydown(trigger(), "ArrowDown");
  await settle();
  assert.equal(openMenus().length, 0);
  await mouseClick(action());
  assert.deepEqual(log, ["open:true", "press"]);
});

/** Give the layout-free test DOM a viewport so the positioner does not clamp to the origin. */
function withViewport(width: number, height: number): () => void {
  const root = document.documentElement;
  Object.defineProperty(root, "clientWidth", { configurable: true, get: () => width });
  Object.defineProperty(root, "clientHeight", { configurable: true, get: () => height });
  return () => {
    Reflect.deleteProperty(root, "clientWidth");
    Reflect.deleteProperty(root, "clientHeight");
  };
}

test("the menu is positioned against the group by default and the trigger on request", async () => {
  const restoreViewport = withViewport(1_000, 800);
  try {
    const horizontal: number[] = [];
    for (const anchor of ["group", "trigger"] as const) {
      const handle = mountMenu(Fixture, {
        props: { rootProps: { anchor }, triggerProps: { openOn: "click" } },
      });
      stubRect(group(), { x: 100, y: 40, width: 120, height: 32 });
      stubRect(trigger(), { x: 180, y: 40, width: 40, height: 32 });
      trigger().click();
      await settle();
      const positioned = one('[role="menu"]').closest<HTMLElement>("[style]");
      const translate = /translate\((-?[\d.]+)px, (-?[\d.]+)px\)/.exec(
        positioned?.getAttribute("style") ?? "",
      );
      assert.ok(translate, `anchor ${anchor} positions the menu`);
      horizontal.push(Number(translate[1]));
      handle.unmount();
    }
    assert.deepEqual(horizontal, [100, 180], "the menu starts at the anchor's inline start");
  } finally {
    restoreViewport();
  }
});
