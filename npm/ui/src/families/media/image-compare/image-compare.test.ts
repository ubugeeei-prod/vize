import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { ImageCompareRootExpose, ImageCompareSlotState } from "./image-compare.ts";
import ImageCompareAfter from "./image-compare-after.vue";
import ImageCompareBefore from "./image-compare-before.vue";
import ImageCompareHandle from "./image-compare-handle.vue";
import ImageCompareLabel from "./image-compare-label.vue";
import ImageCompareRoot from "./image-compare-root.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountCompare(
  props: Record<string, unknown> = {},
  handleProps: Record<string, unknown> = {},
) {
  return mountInteraction(ImageCompareRoot, {
    props: { id: "compare", ...props },
    record: ["update:modelValue", "change", "commit"],
    slots: {
      default: (state: ImageCompareSlotState) => [
        h(ImageCompareBefore, null, () => h("img", { alt: "Before", src: "/before.jpg" })),
        h(ImageCompareAfter, null, () => h("img", { alt: "After", src: "/after.jpg" })),
        h(ImageCompareLabel, { side: "before" }, () => "Before"),
        h(ImageCompareLabel, { side: "after" }, () => "After"),
        h(ImageCompareHandle, handleProps, () =>
          h("span", { "data-slot-position": String(state.position) }),
        ),
      ],
    },
  });
}

function stubRect(element: Element, width = 200, height = 100): void {
  element.getBoundingClientRect = () =>
    ({
      x: 0,
      y: 0,
      left: 0,
      top: 0,
      right: width,
      bottom: height,
      width,
      height,
      toJSON: () => ({}),
    }) satisfies DOMRect;
}

function pointer(target: Element, type: string, init: PointerEventInit = {}): PointerEvent {
  const event = new PointerEvent(type, {
    bubbles: true,
    cancelable: true,
    pointerId: 1,
    button: 0,
    pointerType: "mouse",
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

function slider(handle: ReturnType<typeof mountCompare>): HTMLElement {
  return handle.getByRole("slider");
}

test("renders slider semantics, position custom property, parts, and slot state", () => {
  const handle = mountCompare({ defaultValue: 40 });
  const root = handle.root();
  const thumb = slider(handle);

  assert.equal(root.id, "compare");
  assert.equal(root.getAttribute("data-vize-ui"), "image-compare-root");
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.equal(root.getAttribute("data-orientation"), "horizontal");
  assert.equal(root.getAttribute("data-mode"), "drag");
  assert.equal(root.style.getPropertyValue("--vize-ui-image-compare-position"), "40%");
  assert.equal(thumb.id, "compare-handle");
  assert.equal(thumb.tabIndex, 0);
  assert.equal(thumb.getAttribute("aria-label"), "Comparison position");
  assert.equal(thumb.getAttribute("aria-orientation"), "horizontal");
  assert.equal(thumb.getAttribute("aria-valuemin"), "0");
  assert.equal(thumb.getAttribute("aria-valuemax"), "100");
  assert.equal(thumb.getAttribute("aria-valuenow"), "40");
  assert.equal(thumb.getAttribute("aria-valuetext"), "40%");
  assert.equal(
    thumb.querySelector("[data-slot-position]")?.getAttribute("data-slot-position"),
    "40",
  );
  const before = root.querySelector('[data-vize-ui="image-compare-before"]');
  const after = root.querySelector('[data-vize-ui="image-compare-after"]');
  assert.equal(before?.getAttribute("data-side"), "before");
  assert.equal(after?.getAttribute("part"), "after");
  const labels = [...root.querySelectorAll('[data-vize-ui="image-compare-label"]')];
  assert.deepEqual(
    labels.map((label) => label.getAttribute("data-side")),
    ["before", "after"],
  );
  handle.unmount();
});

test("keyboard follows the slider pattern with step, page step, Home, and End", async () => {
  const handle = mountCompare({ defaultValue: 50, step: 5, pageStep: 20 });
  const thumb = slider(handle);

  const right = await handle.press(thumb, "ArrowRight");
  assert.equal(right.keydownPrevented, true);
  assert.equal(thumb.getAttribute("aria-valuenow"), "55");
  await handle.press(thumb, "ArrowLeft");
  await handle.press(thumb, "ArrowLeft");
  assert.equal(thumb.getAttribute("aria-valuenow"), "45");
  thumb.dispatchEvent(new KeyboardEvent("keydown", { key: "PageUp", bubbles: true }));
  await nextTick();
  assert.equal(thumb.getAttribute("aria-valuenow"), "65");
  thumb.dispatchEvent(new KeyboardEvent("keydown", { key: "PageDown", bubbles: true }));
  await nextTick();
  assert.equal(thumb.getAttribute("aria-valuenow"), "45");
  await handle.press(thumb, "End");
  assert.equal(thumb.getAttribute("aria-valuenow"), "100");
  await handle.press(thumb, "ArrowRight");
  assert.equal(thumb.getAttribute("aria-valuenow"), "100", "positions clamp at the ends");
  await handle.press(thumb, "Home");
  assert.equal(thumb.getAttribute("aria-valuenow"), "0");
  const ignored = new KeyboardEvent("keydown", { key: "a", bubbles: true, cancelable: true });
  thumb.dispatchEvent(ignored);
  assert.equal(ignored.defaultPrevented, false);
  assert.deepEqual(
    handle.wrapper.emitted("change")?.map((payload) => payload[2]),
    ["keyboard", "keyboard", "keyboard", "keyboard", "keyboard", "keyboard", "keyboard"],
  );
  assert.ok((handle.wrapper.emitted("commit")?.length ?? 0) > 0);
  handle.unmount();
});

test("right-to-left and vertical dividers map keys and pointers to the visual direction", async () => {
  const rtl = mountCompare({ dir: "rtl" });
  const rtlRoot = rtl.root();
  assert.equal(rtlRoot.getAttribute("dir"), "rtl");
  await rtl.press(slider(rtl), "ArrowLeft");
  assert.equal(slider(rtl).getAttribute("aria-valuenow"), "51");
  stubRect(rtlRoot, 200, 100);
  pointer(rtlRoot, "pointerdown", { clientX: 50 });
  await nextTick();
  assert.equal(slider(rtl).getAttribute("aria-valuenow"), "75");
  pointer(rtlRoot, "pointerup");
  rtl.unmount();

  const vertical = mountCompare({ orientation: "vertical" });
  const verticalRoot = vertical.root();
  assert.equal(slider(vertical).getAttribute("aria-orientation"), "vertical");
  await vertical.press(slider(vertical), "ArrowDown");
  assert.equal(slider(vertical).getAttribute("aria-valuenow"), "51");
  await vertical.press(slider(vertical), "ArrowUp");
  assert.equal(slider(vertical).getAttribute("aria-valuenow"), "50");
  stubRect(verticalRoot, 100, 200);
  pointer(verticalRoot, "pointerdown", { clientY: 150 });
  await nextTick();
  assert.equal(slider(vertical).getAttribute("aria-valuenow"), "75");
  vertical.unmount();
});

test("pointer drags move the divider with capture, focus the handle, and commit on release", async () => {
  const handle = mountCompare();
  const root = handle.root();
  stubRect(root);
  const captured: number[] = [];
  root.setPointerCapture = (pointerId: number) => {
    captured.push(pointerId);
  };

  const down = pointer(root, "pointerdown", { clientX: 30 });
  await nextTick();
  assert.equal(down.defaultPrevented, true);
  assert.deepEqual(captured, [1]);
  assert.equal(root.getAttribute("data-state"), "dragging");
  assert.ok(handle.activeElement() === slider(handle));
  assert.equal(slider(handle).getAttribute("aria-valuenow"), "15");
  pointer(root, "pointermove", { clientX: 150 });
  await nextTick();
  assert.equal(root.style.getPropertyValue("--vize-ui-image-compare-position"), "75%");
  pointer(root, "pointermove", { clientX: 190, pointerId: 2 });
  await nextTick();
  assert.equal(slider(handle).getAttribute("aria-valuenow"), "75", "other pointers are ignored");
  pointer(root, "pointerup", { clientX: 150 });
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.deepEqual(handle.wrapper.emitted("commit"), [[75]]);
  pointer(root, "pointermove", { clientX: 10 });
  await nextTick();
  assert.equal(slider(handle).getAttribute("aria-valuenow"), "75", "released drags stop tracking");
  pointer(root, "pointerdown", { clientX: 10, button: 2 });
  await nextTick();
  assert.equal(slider(handle).getAttribute("aria-valuenow"), "75", "secondary buttons are ignored");
  handle.unmount();
});

test("hover mode follows mouse pointers without pressing and ignores touch", async () => {
  const handle = mountCompare({ mode: "hover" });
  const root = handle.root();
  stubRect(root);
  assert.equal(root.getAttribute("data-mode"), "hover");
  pointer(root, "pointermove", { clientX: 40 });
  await nextTick();
  assert.equal(slider(handle).getAttribute("aria-valuenow"), "20");
  pointer(root, "pointermove", { clientX: 180, pointerType: "touch" });
  await nextTick();
  assert.equal(slider(handle).getAttribute("aria-valuenow"), "20");
  pointer(root, "pointerdown", { clientX: 180 });
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "idle", "hover mode never drags");
  handle.unmount();
});

test("unmeasured roots ignore pointers", async () => {
  const handle = mountCompare();
  pointer(handle.root(), "pointerdown", { clientX: 30 });
  await nextTick();
  assert.equal(handle.root().getAttribute("data-state"), "idle");
  assert.equal(handle.wrapper.emitted("change"), undefined);
  handle.unmount();
});

test("controlled position wins until the parent accepts the request", async () => {
  const handle = mountCompare({ modelValue: 30 });
  const thumb = slider(handle);
  await handle.press(thumb, "ArrowRight");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[31]]);
  assert.equal(thumb.getAttribute("aria-valuenow"), "30");
  await handle.wrapper.setProps({ modelValue: 31 });
  assert.equal(thumb.getAttribute("aria-valuenow"), "31");
  await handle.wrapper.setProps({ modelValue: 140 });
  assert.equal(thumb.getAttribute("aria-valuenow"), "100", "out-of-range values clamp");
  handle.unmount();
});

test("disabled roots leave the tab order and ignore keys and pointers", async () => {
  const handle = mountCompare({ disabled: true });
  const root = handle.root();
  const thumb = slider(handle);
  stubRect(root);
  assert.equal(root.getAttribute("data-state"), "disabled");
  assert.equal(root.getAttribute("data-disabled"), "true");
  assert.equal(thumb.tabIndex, -1);
  assert.equal(thumb.getAttribute("aria-disabled"), "true");
  await handle.press(thumb, "ArrowRight");
  pointer(root, "pointerdown", { clientX: 10 });
  await nextTick();
  assert.equal(thumb.getAttribute("aria-valuenow"), "50");
  assert.equal(handle.wrapper.emitted("change"), undefined);
  const exposed = handle.exposes<ImageCompareRootExpose>();
  assert.equal(exposed.setPosition(20), true, "the API still moves a disabled divider");
  handle.unmount();
});

test("messages and explicit names localize the handle", () => {
  const localized = mountCompare({
    messages: {
      handleLabel: "比較位置",
      valueText: (position: number) => `前 ${position}% / 後 ${100 - position}%`,
    },
  });
  assert.equal(slider(localized).getAttribute("aria-label"), "比較位置");
  assert.equal(slider(localized).getAttribute("aria-valuetext"), "前 50% / 後 50%");
  localized.unmount();

  const labelled = mountCompare({}, { ariaLabelledby: "caption", ariaDescribedby: "hint" });
  assert.equal(slider(labelled).getAttribute("aria-label"), null);
  assert.equal(slider(labelled).getAttribute("aria-labelledby"), "caption");
  assert.equal(slider(labelled).getAttribute("aria-describedby"), "hint");
  labelled.unmount();

  const named = mountCompare({}, { ariaLabel: "Reveal" });
  assert.equal(slider(named).getAttribute("aria-label"), "Reveal");
  named.unmount();
});

test("exposes typed state and imperative position and focus controls", async () => {
  const handle = mountCompare({ step: 10 });
  const exposed = handle.exposes<ImageCompareRootExpose>();
  assert.equal(exposed.position, 50);
  assert.equal(exposed.orientation, "horizontal");
  assert.equal(exposed.state, "idle");
  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.setPosition(33), true);
  await nextTick();
  assert.equal(exposed.position, 30, "requests snap to the step");
  assert.equal(exposed.setPosition(31), false);
  exposed.focus();
  assert.ok(handle.activeElement() === slider(handle));
  assert.deepEqual(handle.wrapper.emitted("change")?.[0], [30, 50, "api"]);
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const [part, props] of [
    [ImageCompareBefore, {}],
    [ImageCompareAfter, {}],
    [ImageCompareHandle, {}],
    [ImageCompareLabel, { side: "before" }],
  ] as const) {
    assert.throws(
      () => mountInteraction(part, { props }),
      /VIZE_UI_CONTEXT_MISSING: ImageCompare requires a matching provider/,
    );
  }
});
