import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import RangeSlider from "./range-slider.vue";
import RangeSliderRange from "./range-slider-range.vue";
import RangeSliderThumb from "./range-slider-thumb.vue";
import RangeSliderTrack from "./range-slider-track.vue";
import {
  closestRangeSliderThumb,
  normalizeRangeSliderBounds,
  normalizeRangeSliderValue,
  setRangeSliderThumb,
} from "./range-slider-state.ts";
import type {
  RangeSliderExpose,
  RangeSliderProps,
  RangeSliderSlotState,
  RangeSliderThumbSlotState,
} from "./range-slider-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

const labels = ["Minimum", "Maximum", "Third"];

function renderParts(state: RangeSliderSlotState) {
  return h(RangeSliderTrack, null, {
    default: () => [
      h(RangeSliderRange),
      ...state.values.map((_, index) =>
        h(
          RangeSliderThumb,
          { key: index, index, ariaLabel: labels[index] },
          { default: (thumb: RangeSliderThumbSlotState) => String(thumb.value) },
        ),
      ),
    ],
  });
}

function mountRange(props: RangeSliderProps & Record<string, unknown> = {}) {
  return mountInteraction(RangeSlider, {
    props: { ariaLabel: "Price", ...props },
    record: ["update:modelValue", "change"],
    slots: { default: renderParts },
  });
}

function thumbs(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>('[data-vize-ui="range-slider-thumb"]')];
}

function track(root: HTMLElement): HTMLElement {
  const element = root.querySelector('[data-vize-ui="range-slider-track"]');
  assert.ok(element instanceof HTMLElement);
  // happy-dom has no layout; give the track a 200px horizontal/vertical box.
  element.getBoundingClientRect = () => new DOMRect(0, 0, 200, 200);
  return element;
}

async function key(target: HTMLElement, name: string): Promise<boolean> {
  const event = new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  await nextTick();
  return event.defaultPrevented;
}

function pointer(target: Element, type: string, clientX: number, clientY = 0): PointerEvent {
  const event = new PointerEvent(type, {
    bubbles: true,
    cancelable: true,
    button: 0,
    clientX,
    clientY,
    pointerId: 1,
  });
  target.dispatchEvent(event);
  return event;
}

test("normalizes, sorts, spaces, and moves values without crossing neighbors", () => {
  const bounds = normalizeRangeSliderBounds({
    min: 0,
    max: 100,
    step: 5,
    minStepsBetweenThumbs: 2,
  });
  assert.deepEqual(bounds, { min: 0, max: 100, step: 5, largeStep: 50, minDistance: 10 });
  assert.deepEqual(normalizeRangeSliderValue([70, 12, 71], bounds), [10, 70, 80]);
  assert.deepEqual(normalizeRangeSliderValue([100, 100], bounds), [90, 100]);
  assert.deepEqual(normalizeRangeSliderValue(undefined, bounds), [0, 100]);
  assert.deepEqual(normalizeRangeSliderValue([Number.NaN], bounds), [0]);
  assert.deepEqual(setRangeSliderThumb([20, 60], 0, 95, bounds), [50, 60]);
  assert.deepEqual(setRangeSliderThumb([20, 60], 1, -5, bounds), [20, 30]);
  const same = [20, 60];
  assert.ok(setRangeSliderThumb(same, 3, 50, bounds) === same);
  assert.equal(closestRangeSliderThumb([20, 60], 45), 1);
  assert.equal(closestRangeSliderThumb([20, 60], 30), 0);
  assert.equal(closestRangeSliderThumb([50, 50], 70), 1, "ties move the thumb that can follow");
  assert.equal(closestRangeSliderThumb([50, 50], 30), 0);
  assert.deepEqual(normalizeRangeSliderBounds({ min: 10, max: 0, step: Number.NaN }), {
    min: 10,
    max: 11,
    step: 1,
    largeStep: 10,
    minDistance: 0,
  });
});

test("renders a labelled group of APG slider thumbs with range hooks", () => {
  const handle = mountRange({
    defaultValue: [20, 80],
    name: "price",
    minStepsBetweenThumbs: 5,
    getValueText: (value: number) => `$${value}`,
  });
  const root = handle.root();
  const [low, high] = thumbs(root);
  assert.ok(low && high);

  assert.equal(root.getAttribute("role"), "group");
  assert.equal(root.getAttribute("aria-label"), "Price");
  assert.equal(root.getAttribute("data-vize-ui"), "range-slider");
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.equal(root.style.getPropertyValue("--vize-range-slider-range-start"), "20%");
  assert.equal(root.style.getPropertyValue("--vize-range-slider-range-end"), "80%");
  assert.ok(handle.getByRole("slider", { name: "Minimum" }) === low);
  assert.equal(low.tabIndex, 0);
  assert.equal(low.getAttribute("aria-valuenow"), "20");
  assert.equal(low.getAttribute("aria-valuemin"), "0");
  assert.equal(low.getAttribute("aria-valuemax"), "75", "neighbor minus the minimum gap");
  assert.equal(high.getAttribute("aria-valuemin"), "25");
  assert.equal(high.getAttribute("aria-valuemax"), "100");
  assert.equal(high.getAttribute("aria-valuetext"), "$80");
  assert.equal(high.getAttribute("aria-orientation"), "horizontal");
  assert.equal(high.style.getPropertyValue("--vize-range-slider-thumb-percent"), "80%");
  assert.equal(high.textContent, "80");
  assert.deepEqual(
    [...root.querySelectorAll<HTMLInputElement>('input[type="hidden"]')].map((input) => [
      input.name,
      input.value,
    ]),
    [
      ["price", "20"],
      ["price", "80"],
    ],
  );
  handle.unmount();
});

test("keyboard moves one thumb by step, page, and bounds and commits changes", async () => {
  const handle = mountRange({ defaultValue: [20, 80], step: 5 });
  const [low, high] = thumbs(handle.root());
  assert.ok(low && high);

  assert.equal(await key(low, "ArrowRight"), true);
  assert.equal(low.getAttribute("aria-valuenow"), "25");
  await key(low, "ArrowUp");
  await key(low, "ArrowDown");
  await key(low, "ArrowLeft");
  assert.equal(low.getAttribute("aria-valuenow"), "20");
  await key(low, "PageUp");
  assert.equal(low.getAttribute("aria-valuenow"), "70", "largeStep defaults to step * 10");
  await key(low, "End");
  assert.equal(low.getAttribute("aria-valuenow"), "80", "End stops at the next thumb");
  await key(high, "Home");
  assert.equal(high.getAttribute("aria-valuenow"), "80", "Home stops at the previous thumb");
  await key(high, "PageDown");
  assert.equal(await key(high, "a"), false);
  assert.deepEqual(
    handle
      .recorded()
      .filter((entry) => entry.event === "change")
      .map((entry) => [entry.payload[0], entry.payload[1], entry.payload[2]]),
    [
      [[25, 80], 0, "keyboard"],
      [[30, 80], 0, "keyboard"],
      [[25, 80], 0, "keyboard"],
      [[20, 80], 0, "keyboard"],
      [[70, 80], 0, "keyboard"],
      [[80, 80], 0, "keyboard"],
    ],
    "Home on the upper thumb and PageDown at the neighbor do not emit changes",
  );
  handle.unmount();
});

test("RTL mirrors horizontal arrows and vertical sliders use up for increase", async () => {
  const rtl = mountRange({ defaultValue: [50], dir: "rtl" });
  const [rtlThumb] = thumbs(rtl.root());
  assert.ok(rtlThumb);
  await key(rtlThumb, "ArrowLeft");
  assert.equal(rtlThumb.getAttribute("aria-valuenow"), "51");
  rtl.unmount();

  const vertical = mountRange({ defaultValue: [50], orientation: "vertical" });
  const [verticalThumb] = thumbs(vertical.root());
  assert.ok(verticalThumb);
  assert.equal(verticalThumb.getAttribute("aria-orientation"), "vertical");
  await key(verticalThumb, "ArrowUp");
  assert.equal(verticalThumb.getAttribute("aria-valuenow"), "51");
  vertical.unmount();
});

test("pointer down moves the closest thumb, drags it, and commits on release", async () => {
  const handle = mountRange({ defaultValue: [20, 80] });
  const root = handle.root();
  const surface = track(root);
  const [low, high] = thumbs(root);
  assert.ok(low && high);

  const down = pointer(surface, "pointerdown", 140);
  await nextTick();
  assert.equal(down.defaultPrevented, true);
  assert.equal(high.getAttribute("aria-valuenow"), "70");
  assert.ok(document.activeElement === high);
  assert.equal(root.getAttribute("data-state"), "dragging");
  assert.equal(high.getAttribute("data-active"), "true");
  pointer(surface, "pointermove", 180);
  await nextTick();
  assert.equal(high.getAttribute("aria-valuenow"), "90");
  pointer(surface, "pointermove", 10);
  await nextTick();
  assert.equal(high.getAttribute("aria-valuenow"), "20", "drag stops at the neighbor");
  pointer(surface, "pointermove", 400);
  await nextTick();
  assert.equal(high.getAttribute("aria-valuenow"), "100", "clamps beyond the track");
  pointer(surface, "pointerup", 400);
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "idle");
  pointer(surface, "pointermove", 0);
  await nextTick();
  assert.equal(high.getAttribute("aria-valuenow"), "100", "moves after release are ignored");

  const updates = handle.recorded().filter((entry) => entry.event === "update:modelValue");
  const changes = handle.recorded().filter((entry) => entry.event === "change");
  assert.equal(updates.length, 4);
  assert.deepEqual(changes, [{ event: "change", payload: [[20, 100], 1, "pointer"] }]);
  handle.unmount();
});

test("pointer mapping honors RTL and vertical tracks", async () => {
  const rtl = mountRange({ defaultValue: [50], dir: "rtl" });
  pointer(track(rtl.root()), "pointerdown", 50);
  pointer(track(rtl.root()), "pointerup", 50);
  await nextTick();
  assert.equal(thumbs(rtl.root())[0]?.getAttribute("aria-valuenow"), "75");
  rtl.unmount();

  const vertical = mountRange({ defaultValue: [50], orientation: "vertical" });
  pointer(track(vertical.root()), "pointerdown", 0, 50);
  pointer(track(vertical.root()), "pointerup", 0, 50);
  await nextTick();
  assert.equal(thumbs(vertical.root())[0]?.getAttribute("aria-valuenow"), "75");
  vertical.unmount();
});

test("controlled values win until the parent accepts the request", async () => {
  const handle = mountRange({ modelValue: [10, 90] });
  const [low] = thumbs(handle.root());
  assert.ok(low);

  await key(low, "ArrowRight");
  assert.deepEqual(handle.recorded()[0], { event: "update:modelValue", payload: [[11, 90]] });
  assert.equal(low.getAttribute("aria-valuenow"), "10");
  await handle.wrapper.setProps({ modelValue: [11, 90] });
  assert.equal(low.getAttribute("aria-valuenow"), "11");
  handle.unmount();
});

test("submits one value per thumb and restores defaults on form reset", async () => {
  const Probe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          RangeSlider,
          { name: "range", ariaLabel: "Range", defaultValue: [30, 60] },
          { default: renderParts },
        ),
      ]),
  });
  const handle = mountInteraction(Probe);
  const form = handle.root();
  assert.ok(form instanceof HTMLFormElement);
  const [low] = thumbs(form);
  assert.ok(low);

  await key(low, "Home");
  assert.deepEqual(new FormData(form).getAll("range"), ["0", "60"]);
  form.reset();
  await nextTick();
  assert.deepEqual(new FormData(form).getAll("range"), ["30", "60"]);
  handle.unmount();
});

test("disabled sliders stay discoverable but ignore input and submit nothing", async () => {
  const handle = mountRange({ defaultValue: [20, 80], disabled: true, name: "price" });
  const root = handle.root();
  const [low] = thumbs(root);
  assert.ok(low);

  assert.equal(root.getAttribute("data-state"), "disabled");
  assert.equal(low.getAttribute("tabindex"), "0", "disabled thumbs stay discoverable");
  assert.equal(low.getAttribute("aria-disabled"), "true");
  assert.equal(await key(low, "ArrowRight"), false);
  pointer(track(root), "pointerdown", 100);
  await nextTick();
  assert.equal(low.getAttribute("aria-valuenow"), "20");
  assert.ok(
    [...root.querySelectorAll<HTMLInputElement>('input[type="hidden"]')].every(
      (input) => input.disabled,
    ),
  );
  assert.deepEqual(handle.recorded(), []);
  handle.unmount();
});

test("exposes setThumbValue, setValue, focusThumb, reset, and normalized state", async () => {
  const handle = mountRange({ defaultValue: [20, 80], step: 10 });
  const api = handle.exposes<RangeSliderExpose>();
  const [low] = thumbs(handle.root());

  assert.deepEqual(api.values, [20, 80]);
  assert.deepEqual(api.percents, [20, 80]);
  assert.equal(api.setThumbValue(0, 44), true);
  assert.deepEqual(api.values, [40, 80]);
  assert.equal(api.setValue([90, 10, 50]), true);
  assert.deepEqual(api.values, [10, 50, 90]);
  await nextTick();
  assert.equal(thumbs(handle.root()).length, 3);
  api.focusThumb(0);
  assert.ok(document.activeElement === low);
  assert.equal(api.reset(), true);
  assert.deepEqual(api.values, [20, 80]);
  assert.equal(api.state, "idle");
  assert.ok(api.root === handle.root());
  assert.deepEqual(
    handle
      .recorded()
      .filter((entry) => entry.event === "change")
      .map((entry) => entry.payload[2]),
    ["api", "api"],
  );
  handle.unmount();
});

test("thumbs and parts require a RangeSlider provider", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(
      () => mountInteraction(RangeSliderThumb, { props: { index: 0 } }),
      /VIZE_UI_CONTEXT_MISSING: RangeSlider/,
    );
    assert.throws(() => mountInteraction(RangeSliderTrack), /VIZE_UI_CONTEXT_MISSING: RangeSlider/);
    assert.throws(() => mountInteraction(RangeSliderRange), /VIZE_UI_CONTEXT_MISSING: RangeSlider/);
  } finally {
    console.warn = warn;
  }
});
