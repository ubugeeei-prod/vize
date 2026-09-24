import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { PanZoomRootExpose, PanZoomSlotState, PanZoomTransform } from "./pan-zoom.ts";
import PanZoomContent from "./pan-zoom-content.vue";
import PanZoomFit from "./pan-zoom-fit.vue";
import PanZoomReset from "./pan-zoom-reset.vue";
import PanZoomRoot from "./pan-zoom-root.vue";
import PanZoomStatus from "./pan-zoom-status.vue";
import PanZoomViewport from "./pan-zoom-viewport.vue";
import PanZoomZoomIn from "./pan-zoom-zoom-in.vue";
import PanZoomZoomOut from "./pan-zoom-zoom-out.vue";
import { mountInteraction } from "../../../testing/mount.ts";

const VIEWPORT = { left: 10, top: 20, width: 400, height: 300 } as const;

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function stubViewport(element: Element): void {
  element.getBoundingClientRect = () => ({
    x: VIEWPORT.left,
    y: VIEWPORT.top,
    left: VIEWPORT.left,
    top: VIEWPORT.top,
    right: VIEWPORT.left + VIEWPORT.width,
    bottom: VIEWPORT.top + VIEWPORT.height,
    width: VIEWPORT.width,
    height: VIEWPORT.height,
    toJSON: () => ({}),
  });
}

function mountPanZoom(
  props: Record<string, unknown> = {},
  viewportProps: Record<string, unknown> = {},
) {
  const handle = mountInteraction(PanZoomRoot, {
    props: { contentSize: { width: 200, height: 100 }, ...props },
    record: ["update:modelValue", "change", "transformStart", "transformEnd"],
    slots: {
      default: (state: PanZoomSlotState) => [
        h("output", { "data-root-scale": String(state.scale) }, state.state),
        h(PanZoomViewport, { ariaLabel: "Floor plan", ...viewportProps }, () =>
          h(PanZoomContent, null, () => h("img", { alt: "Plan", src: "/plan.png" })),
        ),
        h(PanZoomZoomIn),
        h(PanZoomZoomOut),
        h(PanZoomReset),
        h(PanZoomFit),
        h(PanZoomStatus),
      ],
    },
  });
  const viewport = handle
    .root()
    .querySelector<HTMLDivElement>('[data-vize-ui="pan-zoom-viewport"]');
  const content = handle.root().querySelector<HTMLDivElement>('[data-vize-ui="pan-zoom-content"]');
  assert.ok(viewport && content);
  stubViewport(viewport);
  return { handle, viewport, content };
}

function lastTransform(handle: ReturnType<typeof mountPanZoom>["handle"]): unknown {
  return handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0];
}

function pointer(target: Element, type: string, init: PointerEventInit): PointerEvent {
  const event = new PointerEvent(type, {
    bubbles: true,
    cancelable: true,
    pointerType: "touch",
    button: 0,
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

function wheel(target: Element, init: WheelEventInit): WheelEvent {
  const event = new WheelEvent("wheel", { bubbles: true, cancelable: true, ...init });
  // happy-dom's WheelEvent ignores the inherited MouseEvent init members.
  for (const name of ["clientX", "clientY", "ctrlKey", "metaKey"] as const) {
    const value = init[name];
    if (value !== undefined) Object.defineProperty(event, name, { value });
  }
  target.dispatchEvent(event);
  return event;
}

function key(target: Element, value: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    key: value,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

test("renders a labelled focusable viewport, transformed content, controls, and live status", () => {
  const { handle, viewport, content } = mountPanZoom({
    defaultValue: { x: 10, y: -5, scale: 1.5 },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("data-vize-ui"), "pan-zoom-root");
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.equal(viewport.getAttribute("role"), "group");
  assert.equal(viewport.getAttribute("aria-roledescription"), "pan and zoom area");
  assert.equal(viewport.getAttribute("aria-label"), "Floor plan");
  assert.equal(viewport.getAttribute("tabindex"), "0");
  assert.equal(viewport.style.touchAction, "none");
  assert.equal(content.style.transform, "translate(10px, -5px) scale(1.5)");
  assert.equal(content.style.transformOrigin, "0 0");
  assert.equal(content.style.getPropertyValue("--vize-ui-pan-zoom-scale"), "1.5");
  assert.equal(content.style.getPropertyValue("--vize-ui-pan-zoom-x"), "10px");
  assert.ok(handle.getByRole("button", { name: "Zoom in" }));
  assert.ok(handle.getByRole("button", { name: "Zoom out" }));
  assert.ok(handle.getByRole("button", { name: "Reset zoom" }));
  assert.ok(handle.getByRole("button", { name: "Fit to view" }));
  const status = handle.getByRole("status");
  assert.equal(status.getAttribute("aria-live"), "polite");
  assert.equal(status.textContent, "150%");
  handle.unmount();
});

test("single-pointer drags pan with start and end events", async () => {
  const { handle, viewport } = mountPanZoom();
  pointer(viewport, "pointerdown", { pointerId: 1, clientX: 110, clientY: 120 });
  await nextTick();
  assert.equal(handle.root().getAttribute("data-state"), "panning");
  const move = pointer(viewport, "pointermove", { pointerId: 1, clientX: 140, clientY: 100 });
  assert.equal(move.defaultPrevented, true);
  assert.deepEqual(lastTransform(handle), { x: 30, y: -20, scale: 1 });
  pointer(viewport, "pointerup", { pointerId: 1, clientX: 140, clientY: 100 });
  await nextTick();
  assert.equal(handle.root().getAttribute("data-state"), "idle");
  assert.deepEqual(handle.wrapper.emitted("transformStart"), [["pointer"]]);
  assert.deepEqual(handle.wrapper.emitted("transformEnd"), [["pointer"]]);
  assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], "pointer");

  pointer(viewport, "pointerdown", { pointerId: 2, pointerType: "mouse", button: 2, clientX: 0 });
  pointer(viewport, "pointermove", { pointerId: 2, pointerType: "mouse", clientX: 50 });
  assert.equal(handle.wrapper.emitted("change")?.length, 1, "secondary mouse buttons do not pan");
  handle.unmount();
});

test("two pointers pinch around their midpoint and fall back to panning", async () => {
  const { handle, viewport } = mountPanZoom();
  pointer(viewport, "pointerdown", { pointerId: 1, clientX: 110, clientY: 120 });
  pointer(viewport, "pointerdown", { pointerId: 2, clientX: 210, clientY: 120 });
  await nextTick();
  assert.equal(handle.root().getAttribute("data-state"), "pinching");
  pointer(viewport, "pointermove", { pointerId: 1, clientX: 60, clientY: 120 });
  pointer(viewport, "pointermove", { pointerId: 2, clientX: 260, clientY: 120 });
  assert.deepEqual(lastTransform(handle), { x: -150, y: -100, scale: 2 });
  assert.equal(handle.wrapper.emitted("change")?.at(-1)?.[2], "pinch");

  pointer(viewport, "pointerup", { pointerId: 2, clientX: 260, clientY: 120 });
  await nextTick();
  assert.equal(handle.root().getAttribute("data-state"), "panning");
  pointer(viewport, "pointermove", { pointerId: 1, clientX: 70, clientY: 130 });
  assert.deepEqual(lastTransform(handle), { x: -140, y: -90, scale: 2 });
  pointer(viewport, "pointercancel", { pointerId: 1 });
  await nextTick();
  assert.deepEqual(handle.wrapper.emitted("transformStart"), [["pointer"]]);
  assert.deepEqual(handle.wrapper.emitted("transformEnd"), [["pointer"]]);
  handle.unmount();
});

test("wheel zooms around the cursor, trackpad pinches zoom, and bursts settle once", async () => {
  const { handle, viewport } = mountPanZoom({}, { wheelSettleDelay: 10 });
  const event = wheel(viewport, { deltaY: -100, clientX: 110, clientY: 120 });
  assert.equal(event.defaultPrevented, true);
  const zoomed = lastTransform(handle) as PanZoomTransform;
  assert.ok(zoomed.scale > 1);
  // Content point (100, 100) stays under the cursor at viewport (100, 100).
  assert.ok(Math.abs(100 * zoomed.scale + zoomed.x - 100) < 1e-6);
  wheel(viewport, { deltaY: 1, deltaMode: 1, ctrlKey: true, clientX: 110, clientY: 120 });
  assert.ok((lastTransform(handle) as PanZoomTransform).scale < zoomed.scale);
  assert.equal(handle.getByRole("status").textContent, "100%", "status waits for the burst to end");
  await wait(25);
  await nextTick();
  assert.deepEqual(handle.wrapper.emitted("transformStart"), [["wheel"]]);
  assert.deepEqual(handle.wrapper.emitted("transformEnd"), [["wheel"]]);
  assert.notEqual(handle.getByRole("status").textContent, "100%");
  handle.unmount();
});

test("wheel modes pan or defer to page scrolling without modifiers", () => {
  const pan = mountPanZoom({ wheelMode: "pan" });
  wheel(pan.viewport, { deltaX: 10, deltaY: 2, deltaMode: 1 });
  assert.deepEqual(lastTransform(pan.handle), { x: -160, y: -32, scale: 1 });
  wheel(pan.viewport, { deltaY: -50, ctrlKey: true, clientX: 10, clientY: 20 });
  assert.ok((lastTransform(pan.handle) as PanZoomTransform).scale > 1, "ctrl wheels still zoom");
  pan.handle.unmount();

  const gated = mountPanZoom({ wheelMode: "zoom-with-ctrl" });
  const plain = wheel(gated.viewport, { deltaY: -50 });
  assert.equal(plain.defaultPrevented, false);
  assert.equal(gated.handle.wrapper.emitted("change"), undefined);
  const withCtrl = wheel(gated.viewport, { deltaY: -50, metaKey: true });
  assert.equal(withCtrl.defaultPrevented, true);
  assert.equal(gated.handle.wrapper.emitted("change")?.length, 1);
  gated.handle.unmount();
});

test("double-click zooms in one step at the pointer and Shift zooms out", () => {
  const { handle, viewport } = mountPanZoom({ zoomStep: 2 });
  viewport.dispatchEvent(
    new MouseEvent("dblclick", { bubbles: true, cancelable: true, clientX: 110, clientY: 120 }),
  );
  assert.deepEqual(lastTransform(handle), { x: -100, y: -100, scale: 2 });
  viewport.dispatchEvent(
    new MouseEvent("dblclick", {
      bubbles: true,
      cancelable: true,
      shiftKey: true,
      clientX: 110,
      clientY: 120,
    }),
  );
  assert.deepEqual(lastTransform(handle), { x: 0, y: 0, scale: 1 });
  assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], "double-click");
  handle.unmount();

  const off = mountPanZoom({ doubleClickZoom: false });
  off.viewport.dispatchEvent(new MouseEvent("dblclick", { bubbles: true, cancelable: true }));
  assert.equal(off.handle.wrapper.emitted("change"), undefined);
  off.handle.unmount();
});

test("keyboard pans, zooms around the center, resets, and fits", () => {
  const { handle, viewport } = mountPanZoom({ zoomStep: 2, panStep: 10, maxScale: 8 });
  assert.equal(key(viewport, "ArrowRight").defaultPrevented, true);
  assert.deepEqual(lastTransform(handle), { x: -10, y: 0, scale: 1 });
  key(viewport, "ArrowDown", { shiftKey: true });
  assert.deepEqual(lastTransform(handle), { x: -10, y: -40, scale: 1 });
  key(viewport, "ArrowLeft");
  key(viewport, "ArrowUp");
  assert.deepEqual(lastTransform(handle), { x: 0, y: -30, scale: 1 });
  key(viewport, "=");
  assert.deepEqual(lastTransform(handle), { x: -200, y: -210, scale: 2 });
  key(viewport, "-");
  assert.deepEqual(lastTransform(handle), { x: 0, y: -30, scale: 1 });
  key(viewport, "+");
  key(viewport, "0");
  assert.deepEqual(lastTransform(handle), { x: 0, y: 0, scale: 1 });
  key(viewport, "Home");
  assert.deepEqual(lastTransform(handle), { x: 0, y: 50, scale: 2 });
  assert.ok(handle.wrapper.emitted("change")?.every((payload) => payload[2] === "keyboard"));
  assert.equal(key(viewport, "x").defaultPrevented, false);
  assert.equal(key(viewport, "=", { ctrlKey: true }).defaultPrevented, false);
  const inner = viewport.querySelector("img");
  assert.ok(inner);
  assert.equal(key(inner, "ArrowRight").defaultPrevented, false, "keys from content are ignored");
  handle.unmount();
});

test("buttons zoom by one step, reset, fit, and disable at the scale limits", async () => {
  const { handle } = mountPanZoom({ zoomStep: 2, maxScale: 2, minScale: 0.5 });
  const zoomIn = handle.getByRole("button", { name: "Zoom in" }) as HTMLButtonElement;
  const zoomOut = handle.getByRole("button", { name: "Zoom out" }) as HTMLButtonElement;
  await handle.click(zoomIn);
  assert.deepEqual(lastTransform(handle), { x: -200, y: -150, scale: 2 });
  assert.equal(zoomIn.disabled, true);
  assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], "button");
  await handle.click(zoomOut);
  await handle.click(zoomOut);
  assert.equal((lastTransform(handle) as PanZoomTransform).scale, 0.5);
  assert.equal(zoomOut.disabled, true);
  await handle.click(handle.getByRole("button", { name: "Fit to view" }));
  assert.deepEqual(lastTransform(handle), { x: 0, y: 50, scale: 2 });
  assert.equal(handle.getByRole("status").textContent, "200%");
  await handle.click(handle.getByRole("button", { name: "Reset zoom" }));
  assert.deepEqual(lastTransform(handle), { x: 0, y: 0, scale: 1 });
  handle.unmount();
});

test("bounds constrain every change", async () => {
  const contain = mountPanZoom({ bounds: "contain", defaultValue: { x: 100, y: 100, scale: 1 } });
  assert.equal(contain.handle.exposes<PanZoomRootExpose>().panBy(500, 500), false);
  assert.equal(
    contain.handle.wrapper.emitted("change"),
    undefined,
    "smaller content stays centered",
  );
  contain.handle.exposes<PanZoomRootExpose>().zoomTo(4, { x: 0, y: 0 });
  assert.deepEqual(lastTransform(contain.handle), { x: 0, y: 0, scale: 4 });
  contain.handle.exposes<PanZoomRootExpose>().panBy(-10_000, -10_000);
  assert.deepEqual(lastTransform(contain.handle), { x: -400, y: -100, scale: 4 });
  contain.handle.unmount();

  const cover = mountPanZoom({ bounds: "cover" });
  cover.handle.exposes<PanZoomRootExpose>().zoomTo(1);
  cover.handle.exposes<PanZoomRootExpose>().panBy(1, 1);
  assert.equal((lastTransform(cover.handle) as PanZoomTransform).scale, 3);
  await nextTick();
  const zoomOut = cover.handle.getByRole("button", { name: "Zoom out" }) as HTMLButtonElement;
  assert.equal(zoomOut.disabled, true, "cover scale is the effective minimum");
  cover.handle.unmount();
});

test("controlled transforms win until the parent accepts the request", async () => {
  const { handle, content } = mountPanZoom({ modelValue: { x: 5, y: 5, scale: 1 } });
  handle.exposes<PanZoomRootExpose>().panBy(10, 0);
  await nextTick();
  assert.deepEqual(lastTransform(handle), { x: 15, y: 5, scale: 1 });
  assert.equal(content.style.transform, "translate(5px, 5px) scale(1)");
  await handle.wrapper.setProps({ modelValue: { x: 15, y: 5, scale: 1 } });
  assert.equal(content.style.transform, "translate(15px, 5px) scale(1)");
  handle.unmount();
});

test("disabled roots ignore input but keep the imperative API", async () => {
  const { handle, viewport } = mountPanZoom({ disabled: true });
  await nextTick();
  assert.equal(handle.root().getAttribute("data-state"), "disabled");
  assert.equal(viewport.getAttribute("tabindex"), null);
  assert.equal(viewport.getAttribute("aria-disabled"), "true");
  pointer(viewport, "pointerdown", { pointerId: 1, clientX: 0 });
  pointer(viewport, "pointermove", { pointerId: 1, clientX: 50 });
  wheel(viewport, { deltaY: -100 });
  key(viewport, "ArrowRight");
  viewport.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
  for (const button of handle.root().querySelectorAll("button")) {
    assert.equal(button.disabled, true);
  }
  assert.equal(handle.wrapper.emitted("change"), undefined);
  assert.equal(handle.exposes<PanZoomRootExpose>().zoomTo(2), true);
  handle.unmount();
});

test("controls honor preventDefault from click listeners", async () => {
  const blocked = (event: MouseEvent) => event.preventDefault();
  const handle = mountInteraction(PanZoomRoot, {
    record: ["change"],
    slots: {
      default: () => [
        h(PanZoomZoomIn, { onClick: blocked }),
        h(PanZoomZoomOut, { onClick: blocked }),
        h(PanZoomReset, { onClick: blocked }),
        h(PanZoomFit, { onClick: blocked }),
      ],
    },
  });
  for (const button of handle.root().querySelectorAll("button")) {
    button.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  }
  await nextTick();
  assert.equal(handle.wrapper.emitted("change"), undefined);
  handle.unmount();
});

test("messages localize the role description, labels, and zoom announcement", () => {
  const { handle, viewport } = mountPanZoom({
    defaultValue: { x: 0, y: 0, scale: 2 },
    messages: {
      roleDescription: "ズーム領域",
      zoomIn: "拡大",
      zoomOut: "縮小",
      reset: "リセット",
      fit: "全体表示",
      zoomLevel: (percent: number) => `倍率 ${percent}%`,
    },
  });
  assert.equal(viewport.getAttribute("aria-roledescription"), "ズーム領域");
  assert.ok(handle.getByRole("button", { name: "拡大" }));
  assert.ok(handle.getByRole("button", { name: "全体表示" }));
  assert.equal(handle.getByRole("status").textContent, "倍率 200%");
  handle.unmount();
});

test("exposes typed state and imperative transform controls", () => {
  const { handle } = mountPanZoom({ zoomStep: 2 });
  const exposed = handle.exposes<PanZoomRootExpose>();
  assert.deepEqual(exposed.transform, { x: 0, y: 0, scale: 1 });
  assert.equal(exposed.scale, 1);
  assert.equal(exposed.state, "idle");
  assert.equal(exposed.canZoomIn, true);
  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.zoomIn({ x: 0, y: 0 }), true);
  assert.equal(exposed.scale, 2);
  assert.equal(exposed.zoomOut({ x: 0, y: 0 }), true);
  assert.equal(exposed.setTransform({ x: 1, y: 2, scale: 1 }), true);
  assert.equal(exposed.setTransform({ x: 1, y: 2, scale: 1 }), false);
  assert.equal(exposed.fit(), true);
  assert.equal(exposed.reset(), true);
  assert.equal(exposed.zoomTo(100), true);
  assert.equal(exposed.scale, 8);
  assert.ok(handle.wrapper.emitted("change")?.every((payload) => payload[2] === "api"));
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const part of [
    PanZoomViewport,
    PanZoomContent,
    PanZoomZoomIn,
    PanZoomZoomOut,
    PanZoomReset,
    PanZoomFit,
    PanZoomStatus,
  ]) {
    assert.throws(
      () => mountInteraction(part),
      /VIZE_UI_CONTEXT_MISSING: PanZoom requires a matching provider/,
    );
  }
});
