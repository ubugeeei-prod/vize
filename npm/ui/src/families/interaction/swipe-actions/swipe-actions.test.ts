import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import SwipeActions from "./swipe-actions.vue";
import SwipeActionsAction from "./swipe-actions-action.vue";
import SwipeActionsContent from "./swipe-actions-content.vue";
import SwipeActionsTray from "./swipe-actions-tray.vue";
import type { SwipeActionsExpose, SwipeActionsSlotState } from "./swipe-actions-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function size(element: Element | null, width: number): void {
  if (element instanceof HTMLElement)
    Object.defineProperty(element, "offsetWidth", { value: width, configurable: true });
}

function mountRow(
  props: Record<string, unknown> = {},
  onSelect: (value: string) => void = () => undefined,
) {
  const handle = mountInteraction(SwipeActions, {
    props,
    record: ["update:open", "fullSwipe"],
    slots: {
      default: (state: SwipeActionsSlotState) => [
        h(SwipeActionsTray, { side: "leading" }, () =>
          h(SwipeActionsAction, { value: "pin", onSelect }, () => "Pin"),
        ),
        h(SwipeActionsTray, { side: "trailing" }, () => [
          h(SwipeActionsAction, { value: "archive", onSelect }, () => "Archive"),
          h(SwipeActionsAction, { value: "delete", onSelect }, () => "Delete"),
        ]),
        h(SwipeActionsContent, null, () => `Message ${state.state}`),
      ],
    },
  });
  const root = handle.root();
  size(root.querySelector('[data-side="leading"]'), 80);
  size(root.querySelector('[data-side="trailing"]'), 160);
  size(root.querySelector('[data-vize-ui="swipe-actions-content"]'), 400);
  return handle;
}

function parts(root: HTMLElement) {
  const content = root.querySelector('[data-vize-ui="swipe-actions-content"]');
  const leading = root.querySelector('[data-side="leading"]');
  const trailing = root.querySelector('[data-side="trailing"]');
  assert.ok(
    content instanceof HTMLElement &&
      leading instanceof HTMLElement &&
      trailing instanceof HTMLElement,
  );
  return { content, leading, trailing };
}

function pointer(target: HTMLElement, type: string, x: number, y = 0): void {
  target.dispatchEvent(
    new PointerEvent(type, {
      bubbles: true,
      cancelable: true,
      button: 0,
      clientX: x,
      clientY: y,
      pointerId: 4,
    }),
  );
}

async function drag(
  target: HTMLElement,
  from: number,
  to: number,
  end = "pointerup",
): Promise<void> {
  pointer(target, "pointerdown", from);
  pointer(target, "pointermove", from + Math.sign(to - from) * 10);
  pointer(target, "pointermove", to);
  await nextTick();
  pointer(target, end, to);
  await nextTick();
}

async function key(target: HTMLElement, name: string): Promise<void> {
  target.dispatchEvent(
    new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true }),
  );
  await nextTick();
  await nextTick();
}

test("renders a focusable row with inert trays and a keyboard hint", () => {
  const handle = mountRow();
  const { content, leading, trailing } = parts(handle.root());

  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.equal(content.tabIndex, 0);
  assert.equal(content.getAttribute("aria-keyshortcuts"), "ArrowLeft ArrowRight Escape");
  assert.equal(
    document.getElementById(content.getAttribute("aria-describedby") ?? "")?.textContent,
    "Use arrow keys to reveal actions",
  );
  assert.equal(leading.hasAttribute("inert"), true);
  assert.equal(trailing.getAttribute("role"), "group");
  assert.equal(handle.root().style.getPropertyValue("--vize-swipe-offset"), "0px");
  handle.unmount();
});

test("dragging reveals trays, snaps open past half the tray, and snaps back otherwise", async () => {
  const handle = mountRow();
  const root = handle.root();
  const { content, trailing } = parts(root);

  pointer(content, "pointerdown", 300);
  pointer(content, "pointermove", 290);
  pointer(content, "pointermove", 200);
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "dragging");
  assert.equal(root.style.getPropertyValue("--vize-swipe-offset"), "-100px");
  pointer(content, "pointerup", 200);
  await nextTick();
  assert.equal(root.getAttribute("data-open"), "trailing");
  assert.equal(
    root.style.getPropertyValue("--vize-swipe-offset"),
    "-160px",
    "rests at the tray width",
  );
  assert.equal(trailing.hasAttribute("inert"), false);

  await drag(content, 100, 130);
  assert.equal(root.getAttribute("data-open"), "trailing", "a short reverse drag stays open");
  await drag(content, 100, 200);
  assert.equal(
    root.getAttribute("data-state"),
    "closed",
    "a reverse drag past half the tray closes",
  );
  await drag(content, 100, 130);
  assert.equal(root.getAttribute("data-state"), "closed", "under half the leading tray snaps back");
  await drag(content, 100, 160);
  assert.equal(root.getAttribute("data-open"), "leading");
  assert.deepEqual(
    handle.recorded().map((entry) => entry.payload[0]),
    ["trailing", null, "leading"],
  );

  handle.unmount();
});

test("vertical drags and cancels never open, and sides without trays do not move", async () => {
  const handle = mountInteraction(SwipeActions, {
    slots: {
      default: () => [
        h(SwipeActionsTray, { side: "trailing" }, () =>
          h(SwipeActionsAction, { value: "a" }, () => "A"),
        ),
        h(SwipeActionsContent, null, () => "Row"),
      ],
    },
  });
  const root = handle.root();
  const content = root.querySelector<HTMLElement>('[data-vize-ui="swipe-actions-content"]');
  assert.ok(content);
  size(root.querySelector('[data-side="trailing"]'), 100);

  pointer(content, "pointerdown", 0, 0);
  pointer(content, "pointermove", 3, 40);
  pointer(content, "pointermove", 60, 80);
  pointer(content, "pointerup", 60, 80);
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "closed", "vertical scrolls are left alone");
  await drag(content, 0, 120);
  assert.equal(root.getAttribute("data-state"), "closed", "no leading tray, no movement");
  await drag(content, 200, 100, "pointercancel");
  assert.equal(root.getAttribute("data-state"), "closed");
  handle.unmount();
});

test("a long swipe past the threshold emits fullSwipe and closes", async () => {
  const handle = mountRow();
  const root = handle.root();
  const { content } = parts(root);

  pointer(content, "pointerdown", 380);
  pointer(content, "pointermove", 370);
  pointer(content, "pointermove", 100);
  await nextTick();
  assert.equal(root.getAttribute("data-full-swipe"), "trailing");
  pointer(content, "pointerup", 100);
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.deepEqual(handle.recorded().at(-1), { event: "fullSwipe", payload: ["trailing"] });
  handle.unmount();

  const off = mountRow({ fullSwipe: false });
  const offContent = parts(off.root()).content;
  await drag(offContent, 380, 50);
  assert.equal(off.root().getAttribute("data-open"), "trailing");
  assert.ok(off.recorded().every((entry) => entry.event !== "fullSwipe"));
  off.unmount();
});

test("arrow keys reveal trays and focus the first action; Escape and actions close", async () => {
  const selected: string[] = [];
  const handle = mountRow({}, (value) => selected.push(value));
  const root = handle.root();
  const { content, trailing } = parts(root);
  content.focus();

  await key(content, "ArrowLeft");
  assert.equal(root.getAttribute("data-open"), "trailing");
  assert.ok(document.activeElement === trailing.querySelector("button"));
  await key(content, "Escape");
  assert.equal(root.getAttribute("data-state"), "closed");
  await key(content, "ArrowRight");
  assert.equal(root.getAttribute("data-open"), "leading");
  const pin = root.querySelector<HTMLButtonElement>('[data-value="pin"]');
  pin?.click();
  await nextTick();
  await nextTick();
  assert.deepEqual(selected, ["pin"]);
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.ok(document.activeElement === content, "focus returns to the row");
  handle.unmount();
});

test("RTL mirrors keys and drag direction; controlled and disabled rows", async () => {
  const rtl = mountRow({ dir: "rtl" });
  const rtlContent = parts(rtl.root()).content;
  await key(rtlContent, "ArrowLeft");
  assert.equal(rtl.root().getAttribute("data-open"), "leading");
  assert.equal(rtl.root().style.getPropertyValue("--vize-swipe-offset"), "-80px");
  rtl.unmount();

  const controlled = mountRow({ open: null });
  const api = controlled.exposes<SwipeActionsExpose>();
  assert.equal(api.openSide("trailing"), true);
  await nextTick();
  assert.equal(
    controlled.root().getAttribute("data-state"),
    "closed",
    "controlled rows wait for the parent",
  );
  await controlled.wrapper.setProps({ open: "trailing" });
  assert.equal(controlled.root().getAttribute("data-open"), "trailing");
  controlled.unmount();

  const disabled = mountRow({ disabled: true });
  const disabledContent = parts(disabled.root()).content;
  assert.equal(disabledContent.tabIndex, -1);
  await key(disabledContent, "ArrowLeft");
  await drag(disabledContent, 300, 100);
  assert.equal(disabled.exposes<SwipeActionsExpose>().openSide("leading"), false);
  assert.deepEqual(disabled.recorded(), []);
  disabled.unmount();
});

test("parts require a SwipeActions provider", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(
      () => mountInteraction(SwipeActionsContent),
      /VIZE_UI_CONTEXT_MISSING: SwipeActions/,
    );
    assert.throws(
      () => mountInteraction(SwipeActionsTray, { props: { side: "leading" } }),
      /VIZE_UI_CONTEXT_MISSING/,
    );
    assert.throws(
      () => mountInteraction(SwipeActionsAction, { props: { value: "a" } }),
      /VIZE_UI_CONTEXT_MISSING/,
    );
  } finally {
    console.warn = warn;
  }
});
