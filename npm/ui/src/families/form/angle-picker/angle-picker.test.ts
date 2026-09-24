import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import AnglePicker from "./angle-picker.vue";
import type { AnglePickerExpose } from "./angle-picker-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

async function key(target: HTMLElement, name: string): Promise<void> {
  target.dispatchEvent(
    new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true }),
  );
  await nextTick();
}

function pointer(target: HTMLElement, type: string, x: number, y: number): void {
  target.dispatchEvent(
    new PointerEvent(type, {
      bubbles: true,
      cancelable: true,
      button: 0,
      clientX: x,
      clientY: y,
      pointerId: 3,
    }),
  );
}

test("renders a full-circle slider with degree value text and a CSS angle", () => {
  const handle = mountInteraction(AnglePicker, {
    props: { ariaLabel: "Hue", defaultValue: -90, name: "hue" },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("role"), "slider");
  assert.equal(root.getAttribute("aria-valuenow"), "270", "angles wrap into [0, 360)");
  assert.equal(root.getAttribute("aria-valuemin"), "0");
  assert.equal(root.getAttribute("aria-valuemax"), "359");
  assert.equal(root.getAttribute("aria-valuetext"), "270 degrees");
  assert.equal(root.style.getPropertyValue("--vize-angle-picker-angle"), "270deg");
  assert.equal(root.querySelector<HTMLInputElement>('input[type="hidden"]')?.value, "270");
  handle.unmount();
});

test("keyboard steps wrap around the circle", async () => {
  const handle = mountInteraction(AnglePicker, {
    props: { ariaLabel: "Angle", step: 5 },
    record: ["change"],
  });
  const root = handle.root();

  await key(root, "ArrowLeft");
  assert.equal(root.getAttribute("aria-valuenow"), "355");
  await key(root, "ArrowUp");
  assert.equal(root.getAttribute("aria-valuenow"), "0");
  await key(root, "PageUp");
  assert.equal(root.getAttribute("aria-valuenow"), "15");
  await key(root, "End");
  assert.equal(root.getAttribute("aria-valuenow"), "355");
  await key(root, "Home");
  assert.deepEqual(
    handle.recorded().map((entry) => entry.payload),
    [
      [355, "keyboard"],
      [0, "keyboard"],
      [15, "keyboard"],
      [355, "keyboard"],
      [0, "keyboard"],
    ],
  );
  handle.unmount();
});

test("pointer rotation follows the pointer and snaps to the step", async () => {
  const handle = mountInteraction(AnglePicker, {
    props: { ariaLabel: "Angle", step: 15, getValueText: (angle: number) => `${angle}°` },
    record: ["change"],
  });
  const root = handle.root();
  root.getBoundingClientRect = () => new DOMRect(0, 0, 100, 100);

  pointer(root, "pointerdown", 100, 50);
  await nextTick();
  assert.equal(root.getAttribute("aria-valuenow"), "90");
  assert.equal(root.getAttribute("aria-valuetext"), "90°");
  pointer(root, "pointermove", 10, 60);
  await nextTick();
  assert.equal(root.getAttribute("aria-valuenow"), "255");
  pointer(root, "pointerup", 10, 60);
  await nextTick();
  assert.deepEqual(handle.recorded(), [{ event: "change", payload: [255, "pointer"] }]);
  handle.unmount();
});

test("controlled, disabled, and imperative behavior", async () => {
  const controlled = mountInteraction(AnglePicker, {
    props: { ariaLabel: "Angle", modelValue: 10 },
    record: ["update:modelValue"],
  });
  await key(controlled.root(), "ArrowUp");
  assert.deepEqual(controlled.recorded()[0]?.payload, [11]);
  assert.equal(controlled.root().getAttribute("aria-valuenow"), "10");
  controlled.unmount();

  const disabled = mountInteraction(AnglePicker, { props: { ariaLabel: "Angle", disabled: true } });
  await key(disabled.root(), "ArrowUp");
  assert.equal(disabled.root().getAttribute("aria-valuenow"), "0");
  assert.equal(disabled.root().getAttribute("aria-disabled"), "true");
  disabled.unmount();

  const api = mountInteraction(AnglePicker, { props: { ariaLabel: "Angle", defaultValue: 45 } });
  const exposed = api.exposes<AnglePickerExpose>();
  assert.equal(exposed.setValue(725), true);
  assert.equal(exposed.value, 5);
  assert.equal(exposed.reset(), true);
  assert.equal(exposed.angle, 45);
  api.unmount();
});
