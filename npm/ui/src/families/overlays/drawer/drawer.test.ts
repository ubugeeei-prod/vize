import assert from "node:assert/strict";

import { afterEach, beforeEach, test, vi } from "vite-plus/test";
import { h, nextTick } from "vue";

import {
  DrawerClose,
  DrawerDescription,
  DrawerTitle,
  DrawerTrigger,
  resolveDrawerRelease,
  stepDrawerSnapPoint,
} from "./drawer.ts";
import DrawerContent from "./drawer-content.vue";
import DrawerHandle from "./drawer-handle.vue";
import DrawerRoot from "./drawer-root.vue";
import type { DrawerDismissEvent, DrawerDragEndEvent, DrawerRootExpose } from "./drawer.ts";
import { drawerSnapOffset } from "./drawer-snap.ts";
import { mountInteraction } from "../../../testing/mount.ts";

const drawerRect = { bottom: 800, height: 400, left: 0, right: 300, top: 400, width: 300 };
let clock = 0;

beforeEach(() => {
  clock = 0;
  vi.spyOn(HTMLDialogElement.prototype, "getBoundingClientRect").mockImplementation(() =>
    DOMRect.fromRect({
      height: drawerRect.height,
      width: drawerRect.width,
      x: drawerRect.left,
      y: drawerRect.top,
    }),
  );
});

afterEach(() => {
  vi.restoreAllMocks();
});

async function settle(): Promise<void> {
  await nextTick();
  await nextTick();
}

function mountDrawer(
  props: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(DrawerRoot, {
    props: { id: "cart", ...props },
    slots: {
      default: () => [
        h(DrawerTrigger, null, () => "Open cart"),
        h(DrawerContent, contentProps, () => [
          h(DrawerHandle),
          h(DrawerTitle, null, () => "Cart"),
          h(DrawerDescription, null, () => "Items waiting for checkout."),
          h("button", { type: "button", class: "inner" }, "Inner action"),
          h("input", { "aria-label": "Coupon" }),
          h(DrawerClose, null, () => "Close"),
        ]),
      ],
    },
  });
}

function dialogOf(root: HTMLElement): HTMLDialogElement {
  const dialog = root.querySelector('[data-vize-ui="drawer-content"]');
  assert.ok(dialog instanceof HTMLDialogElement);
  return dialog;
}

function pointer(
  target: Element,
  type: "pointercancel" | "pointerdown" | "pointermove" | "pointerup",
  y: number,
  x = 150,
): void {
  const event = new PointerEvent(type, {
    bubbles: true,
    button: 0,
    cancelable: true,
    clientX: x,
    clientY: y,
    pointerId: 7,
    pointerType: "touch",
  });
  Object.defineProperty(event, "timeStamp", { value: clock });
  target.dispatchEvent(event);
}

async function drag(target: Element, from: number, to: number, step = 100): Promise<void> {
  pointer(target, "pointerdown", from);
  clock += step;
  pointer(target, "pointermove", from + (to - from) / 2);
  clock += step;
  pointer(target, "pointermove", to);
  clock += step;
  pointer(target, "pointerup", to);
  await settle();
}

test("opens a labelled native modal dialog from the trigger and restores focus on close", async () => {
  const handle = mountDrawer();
  const root = handle.root();
  const trigger = handle.getByRole("button", { name: "Open cart" });
  const dialog = dialogOf(root);

  assert.equal(dialog.open, false);
  assert.equal(dialog.querySelector(".inner"), null);
  trigger.focus();
  await handle.click(trigger);
  await settle();

  assert.equal(dialog.open, true);
  assert.equal(root.getAttribute("data-state"), "open");
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(trigger.getAttribute("aria-controls"), "cart-content");
  assert.equal(dialog.id, "cart-content");
  assert.equal(dialog.getAttribute("aria-labelledby"), "cart-title");
  assert.equal(dialog.getAttribute("aria-describedby"), "cart-description");
  assert.equal(dialog.getAttribute("data-side"), "bottom");
  assert.equal(dialog.getAttribute("data-modal"), "true");
  assert.equal(dialog.style.getPropertyValue("--vize-drawer-offset"), "0px");
  assert.ok(dialog.contains(document.activeElement));

  const close = dialog.querySelector('[data-vize-ui="dialog-close"]');
  assert.ok(close instanceof HTMLButtonElement);
  await handle.click(close);
  await settle();

  assert.equal(dialog.open, false);
  assert.equal(root.getAttribute("data-state"), "closed");
  assert.ok(
    document.activeElement === trigger,
    `focus restored to ${document.activeElement?.tagName}`,
  );
  handle.unmount();
});

test("Escape and the native cancel event dismiss through Drawer state", async () => {
  const dismissals: DrawerDismissEvent[] = [];
  const handle = mountDrawer(
    { defaultOpen: true },
    { onDismiss: (e: DrawerDismissEvent) => dismissals.push(e) },
  );
  await settle();
  const dialog = dialogOf(handle.root());
  assert.equal(dialog.open, true);

  const cancel = new Event("cancel", { cancelable: true });
  dialog.dispatchEvent(cancel);
  await settle();
  assert.equal(cancel.defaultPrevented, true);
  assert.equal(dialog.open, false);
  assert.equal(dismissals[0]?.reason, "escape-key");

  await handle.wrapper.vm.$nextTick();
  handle.exposes<DrawerRootExpose>().openDrawer();
  await settle();
  assert.equal(dialog.open, true);
  const inner = dialog.querySelector(".inner");
  assert.ok(inner instanceof HTMLElement);
  inner.dispatchEvent(
    new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "Escape" }),
  );
  await settle();
  assert.equal(dialog.open, false);
  assert.equal(dismissals[1]?.reason, "escape-key");
  handle.unmount();
});

test("escape-key-down is preventable and non-dismissible drawers ignore cancel", async () => {
  const prevented = mountDrawer(
    { defaultOpen: true },
    { onEscapeKeyDown: (event: { preventDefault: () => void }) => event.preventDefault() },
  );
  await settle();
  const dialog = dialogOf(prevented.root());
  dialog.dispatchEvent(
    new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "Escape" }),
  );
  await settle();
  assert.equal(dialog.open, true);
  prevented.unmount();

  const locked = mountDrawer({ defaultOpen: true, dismissible: false });
  await settle();
  const lockedDialog = dialogOf(locked.root());
  const cancel = new Event("cancel", { cancelable: true });
  lockedDialog.dispatchEvent(cancel);
  await settle();
  assert.equal(cancel.defaultPrevented, true);
  assert.equal(lockedDialog.open, true);
  locked.unmount();
});

test("controlled open state waits for the parent", async () => {
  const handle = mountDrawer({ open: false });
  const trigger = handle.getByRole("button", { name: "Open cart" });
  const dialog = dialogOf(handle.root());

  await handle.click(trigger);
  await settle();
  assert.equal(dialog.open, false);
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);

  await handle.wrapper.setProps({ open: true });
  await settle();
  assert.equal(dialog.open, true);
  handle.unmount();
});

test("backdrop presses outside the dialog box dismiss unless prevented", async () => {
  const dismissals: DrawerDismissEvent[] = [];
  const handle = mountDrawer(
    { defaultOpen: true },
    { onDismiss: (event: DrawerDismissEvent) => dismissals.push(event) },
  );
  await settle();
  const dialog = dialogOf(handle.root());

  pointer(dialog, "pointerdown", 500);
  pointer(dialog, "pointerup", 500);
  await settle();
  assert.equal(dialog.open, true, "a press inside the box is not a backdrop press");

  pointer(dialog, "pointerdown", 100);
  await settle();
  assert.equal(dialog.open, false);
  assert.equal(dismissals[0]?.reason, "backdrop");
  handle.unmount();

  const kept = mountDrawer(
    { defaultOpen: true },
    { onBackdropPointerDown: (event: { preventDefault: () => void }) => event.preventDefault() },
  );
  await settle();
  const keptDialog = dialogOf(kept.root());
  pointer(keptDialog, "pointerdown", 100);
  await settle();
  assert.equal(keptDialog.open, true);
  kept.unmount();
});

test("non-modal drawers use show() and dismiss on outside presses", async () => {
  const handle = mountDrawer({ defaultOpen: true, modal: false });
  await settle();
  const dialog = dialogOf(handle.root());
  assert.equal(dialog.open, true);
  assert.equal(dialog.getAttribute("data-modal"), "false");

  const outside = document.createElement("button");
  document.body.append(outside);
  outside.dispatchEvent(
    new PointerEvent("pointerdown", { bubbles: true, cancelable: true, pointerId: 1 }),
  );
  await settle();
  assert.equal(dialog.open, false);
  outside.remove();
  handle.unmount();
});

test("a native close (form method=dialog) is mirrored into state", async () => {
  const handle = mountDrawer({ defaultOpen: true });
  await settle();
  const dialog = dialogOf(handle.root());
  dialog.close();
  await settle();
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[false]]);
  handle.unmount();
});

test("dragging publishes offsets and releases to the nearest snap point", async () => {
  const ends: DrawerDragEndEvent[] = [];
  const handle = mountDrawer({
    defaultOpen: true,
    snapPoints: [0.5, 1],
    "onDrag-end": (event: DrawerDragEndEvent) => ends.push(event),
  });
  await settle();
  const dialog = dialogOf(handle.root());
  assert.equal(dialog.getAttribute("data-snap-point"), "0.5");
  assert.equal(dialog.style.getPropertyValue("--vize-drawer-snap-offset"), "200px");

  pointer(dialog, "pointerdown", 600);
  clock += 100;
  pointer(dialog, "pointermove", 540);
  await settle();
  assert.equal(dialog.getAttribute("data-dragging"), "true");
  assert.equal(dialog.style.getPropertyValue("--vize-drawer-drag-offset"), "-60px");
  assert.equal(dialog.style.getPropertyValue("--vize-drawer-offset"), "140px");
  clock += 100;
  pointer(dialog, "pointermove", 450);
  clock += 100;
  pointer(dialog, "pointerup", 450);
  await settle();

  assert.equal(dialog.getAttribute("data-dragging"), null);
  assert.equal(dialog.getAttribute("data-snap-point"), "1");
  assert.equal(dialog.style.getPropertyValue("--vize-drawer-offset"), "0px");
  assert.deepEqual(handle.wrapper.emitted("update:activeSnapPoint"), [[1]]);
  assert.equal(ends[0]?.outcome, "snap");
  assert.equal(ends[0]?.snapPoint, 1);
  assert.equal(handle.wrapper.emitted("drag-start")?.length, 1);
  handle.unmount();
});

test("dragging past the threshold or flicking dismisses; short drags snap back", async () => {
  const handle = mountDrawer({ defaultOpen: true });
  await settle();
  const dialog = dialogOf(handle.root());

  await drag(dialog, 450, 500);
  assert.equal(dialog.open, true);
  assert.equal(dialog.style.getPropertyValue("--vize-drawer-offset"), "0px");

  await drag(dialog, 450, 600);
  assert.equal(dialog.open, false);
  handle.unmount();

  const flick = mountDrawer({ defaultOpen: true });
  await settle();
  const flickDialog = dialogOf(flick.root());
  await drag(flickDialog, 450, 490, 5);
  assert.equal(flickDialog.open, false);
  flick.unmount();
});

test("non-dismissible drawers never close from drags and pointercancel restores position", async () => {
  const ends: DrawerDragEndEvent[] = [];
  const handle = mountDrawer({
    defaultOpen: true,
    dismissible: false,
    "onDrag-end": (event: DrawerDragEndEvent) => ends.push(event),
  });
  await settle();
  const dialog = dialogOf(handle.root());
  await drag(dialog, 450, 780);
  assert.equal(dialog.open, true);
  assert.equal(ends[0]?.outcome, "snap");

  pointer(dialog, "pointerdown", 450);
  pointer(dialog, "pointermove", 520);
  pointer(dialog, "pointercancel", 520);
  await settle();
  assert.equal(ends[1]?.outcome, "cancel");
  assert.equal(dialog.style.getPropertyValue("--vize-drawer-drag-offset"), "0px");
  handle.unmount();
});

test("clicks survive taps but are suppressed after a drag; form fields never start drags", async () => {
  let clicks = 0;
  const handle = mountDrawer({ defaultOpen: true });
  await settle();
  const dialog = dialogOf(handle.root());
  const inner = dialog.querySelector(".inner");
  const input = dialog.querySelector("input");
  assert.ok(inner instanceof HTMLButtonElement);
  assert.ok(input instanceof HTMLInputElement);
  inner.addEventListener("click", () => clicks++);

  pointer(inner, "pointerdown", 500);
  pointer(inner, "pointerup", 500);
  inner.click();
  assert.equal(clicks, 1);

  await drag(inner, 450, 500);
  inner.click();
  assert.equal(clicks, 1);

  pointer(input, "pointerdown", 450);
  pointer(input, "pointermove", 700);
  await settle();
  assert.equal(dialog.getAttribute("data-dragging"), null);
  pointer(input, "pointerup", 700);
  handle.unmount();
});

test("dragFromContent=false limits drags to the handle", async () => {
  const handle = mountDrawer({ defaultOpen: true }, { dragFromContent: false });
  await settle();
  const dialog = dialogOf(handle.root());
  const inner = dialog.querySelector(".inner");
  const grip = dialog.querySelector('[data-vize-ui="drawer-handle"]');
  assert.ok(inner instanceof HTMLElement);
  assert.ok(grip instanceof HTMLElement);

  await drag(inner, 450, 700);
  assert.equal(dialog.open, true);
  await drag(grip, 450, 700);
  assert.equal(dialog.open, false);
  handle.unmount();
});

test("the handle cycles snap points on click and steps with side-aware arrow keys", async () => {
  const handle = mountDrawer({ defaultOpen: true, snapPoints: ["120px", 0.5, 1] });
  await settle();
  const dialog = dialogOf(handle.root());
  const grip = dialog.querySelector('[data-vize-ui="drawer-handle"]');
  assert.ok(grip instanceof HTMLButtonElement);
  assert.equal(grip.getAttribute("aria-label"), "Resize drawer");
  assert.equal(grip.getAttribute("aria-controls"), "cart-content");
  assert.equal(grip.getAttribute("data-snap-point"), "120px");

  await handle.click(grip);
  assert.equal(grip.getAttribute("data-snap-point"), "0.5");
  await handle.press(grip, "ArrowUp");
  assert.equal(grip.getAttribute("data-snap-point"), "1");
  await handle.press(grip, "ArrowUp");
  assert.equal(grip.getAttribute("data-snap-point"), "1", "keys do not wrap");
  await handle.click(grip);
  assert.equal(grip.getAttribute("data-snap-point"), "120px", "clicks wrap");
  await handle.press(grip, "ArrowDown");
  assert.equal(grip.getAttribute("data-snap-point"), "120px");
  handle.unmount();

  const right = mountDrawer({ defaultOpen: true, side: "right", snapPoints: [0.5, 1] });
  await settle();
  const rightGrip = dialogOf(right.root()).querySelector('[data-vize-ui="drawer-handle"]');
  assert.ok(rightGrip instanceof HTMLButtonElement);
  await right.press(rightGrip, "ArrowLeft");
  assert.equal(rightGrip.getAttribute("data-snap-point"), "1");
  await right.press(rightGrip, "ArrowUp");
  assert.equal(rightGrip.getAttribute("data-snap-point"), "1");
  right.unmount();
});

test("controlled snap points emit requests and follow the parent", async () => {
  const handle = mountDrawer({ defaultOpen: true, snapPoints: [0.5, 1], activeSnapPoint: 0.5 });
  await settle();
  const exposed = handle.exposes<DrawerRootExpose>();
  assert.equal(exposed.snapTo(1), true);
  await settle();
  assert.equal(exposed.activeSnapPoint, 0.5);
  assert.deepEqual(handle.wrapper.emitted("update:activeSnapPoint"), [[1]]);
  assert.equal(exposed.snapTo(0.75), false, "unknown snap points are ignored");
  await handle.wrapper.setProps({ activeSnapPoint: 1 });
  assert.equal(exposed.activeSnapPoint, 1);
  handle.unmount();
});

test("invalid snap points and missing providers throw stable diagnostics", () => {
  assert.throws(() => mountDrawer({ snapPoints: [1.5] }), /VIZE_UI_DRAWER_SNAP_POINT/);
  assert.throws(() => mountInteraction(DrawerContent), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(DrawerHandle), /VIZE_UI_CONTEXT_MISSING/);
});

test("snap math resolves pixel and fractional points with velocity projection", () => {
  assert.equal(drawerSnapOffset("100px", 400), 300);
  assert.equal(drawerSnapOffset(0.25, 400), 300);
  assert.equal(drawerSnapOffset(null, 400), 0);
  const base = {
    closeThreshold: 0.25,
    dismissible: true,
    size: 400,
    snapPoints: [0.25, 1] as const,
    startOffset: 0,
    velocityThreshold: 0.5,
  };
  assert.deepEqual(resolveDrawerRelease({ ...base, distance: 100, velocity: 0 }), {
    outcome: "snap",
    snapPoint: 1,
  });
  assert.deepEqual(resolveDrawerRelease({ ...base, distance: 100, velocity: 1.2 }), {
    outcome: "snap",
    snapPoint: 0.25,
  });
  assert.deepEqual(resolveDrawerRelease({ ...base, distance: 330, velocity: 0 }), {
    outcome: "dismiss",
  });
  assert.equal(stepDrawerSnapPoint([1, "80px", 0.5], "80px", 1, 400, false), 0.5);
  assert.equal(stepDrawerSnapPoint([1, "80px", 0.5], 1, 1, 400, true), "80px");
  assert.equal(stepDrawerSnapPoint([], null, 1, 400, true), null);
});
