import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import Knob from "./knob.vue";
import {
  angleToValue,
  normalizeRotaryBounds,
  normalizeRotarySweep,
  pointerAngle,
  snapAngle,
  valueToAngle,
  wrapAngle,
} from "./knob-geometry.ts";
import type { KnobExpose, KnobSlotState } from "./knob-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function mountKnob(props: Record<string, unknown> = {}) {
  return mountInteraction(Knob, {
    props: { ariaLabel: "Volume", ...props },
    record: ["update:modelValue", "change"],
    slots: { default: (state: KnobSlotState) => `${state.value}@${state.angle}` },
  });
}

async function key(target: HTMLElement, name: string): Promise<boolean> {
  const event = new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  await nextTick();
  return event.defaultPrevented;
}

function box(target: HTMLElement): void {
  // happy-dom has no layout: a 100px square centered at (50, 50).
  target.getBoundingClientRect = () => new DOMRect(0, 0, 100, 100);
}

function pointer(target: HTMLElement, type: string, x: number, y: number): PointerEvent {
  const event = new PointerEvent(type, {
    bubbles: true,
    cancelable: true,
    button: 0,
    clientX: x,
    clientY: y,
    pointerId: 7,
  });
  target.dispatchEvent(event);
  return event;
}

test("maps angles and values with a dead zone that never jumps across ends", () => {
  const bounds = normalizeRotaryBounds({ min: 0, max: 100, step: 1 });
  const sweep = normalizeRotarySweep({});
  assert.deepEqual(sweep, { start: -135, end: 135 });
  assert.deepEqual(normalizeRotarySweep({ startAngle: 10, endAngle: 10 }), sweep);
  assert.deepEqual(normalizeRotaryBounds({ min: 5, max: 1, step: 0 }), {
    min: 5,
    max: 6,
    step: 1,
    largeStep: 10,
  });
  assert.equal(pointerAngle({ x: 0, y: 0 }, { x: 0, y: -1 }), 0, "12 o'clock");
  assert.equal(pointerAngle({ x: 0, y: 0 }, { x: 1, y: 0 }), 90);
  assert.equal(pointerAngle({ x: 0, y: 0 }, { x: 0, y: 1 }), 180);
  assert.equal(pointerAngle({ x: 0, y: 0 }, { x: -1, y: 0 }), -90);
  assert.equal(valueToAngle(50, bounds, sweep), 0);
  assert.equal(valueToAngle(0, bounds, sweep), -135);
  assert.equal(angleToValue(0, bounds, sweep), 50);
  assert.equal(angleToValue(135, bounds, sweep), 100);
  assert.equal(angleToValue(170, bounds, sweep), 100, "dead zone near max stays at max");
  assert.equal(angleToValue(-170, bounds, sweep), 0, "dead zone near min stays at min");
  assert.equal(wrapAngle(-90), 270);
  assert.equal(wrapAngle(720), 0);
  assert.equal(snapAngle(359.6, 1), 0);
  assert.equal(snapAngle(52, 15), 45);
});

test("renders an APG slider with angle hooks and a hidden form value", () => {
  const handle = mountKnob({
    defaultValue: 50,
    name: "volume",
    getValueText: (v: number) => `${v}%`,
  });
  const root = handle.root();

  assert.ok(handle.getByRole("slider", { name: "Volume" }) === root);
  assert.equal(root.tabIndex, 0);
  assert.equal(root.getAttribute("aria-valuenow"), "50");
  assert.equal(root.getAttribute("aria-valuemin"), "0");
  assert.equal(root.getAttribute("aria-valuemax"), "100");
  assert.equal(root.getAttribute("aria-valuetext"), "50%");
  assert.equal(root.getAttribute("data-vize-ui"), "knob");
  assert.equal(root.style.getPropertyValue("--vize-knob-angle"), "0deg");
  assert.equal(root.style.getPropertyValue("--vize-knob-percent"), "50%");
  assert.equal(root.querySelector<HTMLInputElement>('input[type="hidden"]')?.value, "50");
  assert.equal(root.textContent, "50@0");
  handle.unmount();
});

test("keyboard steps, pages, and jumps to bounds, committing each change", async () => {
  const handle = mountKnob({ defaultValue: 50, step: 5 });
  const root = handle.root();

  assert.equal(await key(root, "ArrowUp"), true);
  await key(root, "ArrowRight");
  assert.equal(root.getAttribute("aria-valuenow"), "60");
  await key(root, "ArrowLeft");
  await key(root, "PageDown");
  assert.equal(root.getAttribute("aria-valuenow"), "5");
  await key(root, "Home");
  await key(root, "Home");
  await key(root, "End");
  assert.equal(root.getAttribute("aria-valuenow"), "100");
  assert.equal(await key(root, "a"), false);
  assert.deepEqual(
    handle
      .recorded()
      .filter((entry) => entry.event === "change")
      .map((entry) => entry.payload[0]),
    [55, 60, 55, 5, 0, 100],
    "Home at the minimum does not emit",
  );
  handle.unmount();
});

test("dragging rotates around the center and commits once on release", async () => {
  const handle = mountKnob({ defaultValue: 0 });
  const root = handle.root();
  box(root);

  const down = pointer(root, "pointerdown", 50, 0);
  await nextTick();
  assert.equal(down.defaultPrevented, true);
  assert.ok(document.activeElement === root);
  assert.equal(root.getAttribute("aria-valuenow"), "50", "12 o'clock is the middle of the sweep");
  assert.equal(root.getAttribute("data-state"), "dragging");
  pointer(root, "pointermove", 100, 50);
  await nextTick();
  assert.equal(root.getAttribute("aria-valuenow"), "83");
  pointer(root, "pointermove", 55, 100);
  await nextTick();
  assert.equal(root.getAttribute("aria-valuenow"), "100", "the dead zone clamps to the nearer end");
  pointer(root, "pointerup", 55, 100);
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.deepEqual(
    handle.recorded().filter((entry) => entry.event === "change"),
    [{ event: "change", payload: [100, "pointer"] }],
  );
  handle.unmount();
});

test("wheel is opt-in and needs focus; disabled and read-only knobs ignore input", async () => {
  const wheel = (target: HTMLElement, deltaY: number) => {
    const event = new WheelEvent("wheel", { deltaY, bubbles: true, cancelable: true });
    target.dispatchEvent(event);
    return event.defaultPrevented;
  };
  const opted = mountKnob({ defaultValue: 10, allowWheel: true });
  await nextTick();
  assert.equal(wheel(opted.root(), -1), false, "unfocused knobs let the page scroll");
  opted.root().focus();
  assert.equal(wheel(opted.root(), -1), true);
  await nextTick();
  assert.equal(opted.root().getAttribute("aria-valuenow"), "11");
  opted.unmount();

  for (const flag of ["disabled", "readOnly"]) {
    const locked = mountKnob({ defaultValue: 10, [flag]: true, name: "v" });
    box(locked.root());
    await key(locked.root(), "ArrowUp");
    pointer(locked.root(), "pointerdown", 100, 50);
    await nextTick();
    assert.equal(locked.root().getAttribute("aria-valuenow"), "10");
    assert.equal(
      locked.root().getAttribute("data-state"),
      flag === "disabled" ? "disabled" : "readonly",
    );
    assert.deepEqual(locked.recorded(), []);
    locked.unmount();
  }
});

test("controlled values, form reset, and the imperative API", async () => {
  const controlled = mountKnob({ modelValue: 20 });
  await key(controlled.root(), "ArrowUp");
  assert.deepEqual(controlled.recorded()[0], { event: "update:modelValue", payload: [21] });
  assert.equal(controlled.root().getAttribute("aria-valuenow"), "20");
  await controlled.wrapper.setProps({ modelValue: 21 });
  assert.equal(controlled.root().getAttribute("aria-valuenow"), "21");
  controlled.unmount();

  const Probe = defineComponent({
    setup: () => () => h("form", [h(Knob, { name: "gain", ariaLabel: "Gain", defaultValue: 30 })]),
  });
  const form = mountInteraction(Probe);
  const formElement = form.root();
  assert.ok(formElement instanceof HTMLFormElement);
  const knob = formElement.querySelector<HTMLElement>('[role="slider"]');
  assert.ok(knob);
  await key(knob, "End");
  assert.equal(new FormData(formElement).get("gain"), "100");
  formElement.reset();
  await nextTick();
  assert.equal(new FormData(formElement).get("gain"), "30");
  form.unmount();

  const api = mountKnob({ min: -10, max: 10, step: 0.5 });
  const exposed = api.exposes<KnobExpose>();
  assert.equal(exposed.setValue(3.3), true);
  assert.equal(exposed.value, 3.5);
  assert.equal(exposed.angle, 47.25);
  assert.equal(exposed.reset(), true);
  assert.equal(exposed.value, -10);
  exposed.focus();
  assert.ok(document.activeElement === api.root());
  api.unmount();
});
