import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import { hueFromPoint, pointFromHue, stepHue } from "../color-wheel-geometry.ts";
import ColorWheel from "../color-wheel.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function close(actual: number, expected: number): void {
  assert.ok(Math.abs(actual - expected) < 1e-9, `${actual} ~= ${expected}`);
}

test("maps screen axes to a clockwise hue", () => {
  close(hueFromPoint(1, 0), 0);
  close(hueFromPoint(0, 1), 90);
  close(hueFromPoint(-1, 0), 180);
  close(hueFromPoint(0, -1), 270);
  close(hueFromPoint(0, 0), 0);
  close(pointFromHue(0, 36).x, 36);
  close(pointFromHue(0, 36).y, 0);
  close(pointFromHue(90, 36).y, 36);
  assert.equal(stepHue(0, -1), 359);
  assert.equal(stepHue(359, 1), 0);
  assert.equal(stepHue(10, -30), 340);
});

test("renders hue 0 at the right of the wheel", () => {
  const handle = mountInteraction(ColorWheel, { props: { modelValue: 0 } });
  const slider = handle.getByRole("slider", { name: "Hue" });
  const marker = slider.querySelector("[data-part='marker']");

  assert.equal(slider.getAttribute("data-vize-ui"), "color-wheel");
  assert.equal(slider.getAttribute("aria-valuemin"), "0");
  assert.equal(slider.getAttribute("aria-valuemax"), "360");
  assert.equal(slider.getAttribute("aria-valuenow"), "0");
  assert.equal(slider.getAttribute("aria-valuetext"), "0 degrees");
  assert.equal(slider.getAttribute("tabindex"), "0");
  assert.equal(marker?.getAttribute("cx"), "86.00");
  assert.equal(marker?.getAttribute("cy"), "50.00");
  handle.unmount();
});

test("steps hue from the arrow keys and wraps", async () => {
  const handle = mountInteraction(ColorWheel, { props: { modelValue: 0 } });
  const slider = handle.getByRole("slider");

  const right = await handle.press(slider, "ArrowRight");
  assert.equal(right.keydownPrevented, true);
  assert.equal(handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0], 1);
  handle.unmount();

  const wrapped = mountInteraction(ColorWheel, { props: { modelValue: 0 } });
  const left = await wrapped.press(wrapped.getByRole("slider"), "ArrowLeft");
  assert.equal(left.keydownPrevented, true);
  assert.equal(wrapped.wrapper.emitted("update:modelValue")?.at(-1)?.[0], 359);
  wrapped.unmount();
});

test("sends Home to 0 and End to 360", async () => {
  const handle = mountInteraction(ColorWheel, { props: { modelValue: 40 } });
  const slider = handle.getByRole("slider");

  await handle.press(slider, "Home");
  assert.equal(handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0], 0);
  await handle.press(slider, "End");
  assert.equal(handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0], 360);
  assert.equal(slider.getAttribute("aria-valuetext"), "360 degrees");
  handle.unmount();
});

test("uses a ten-degree step for Shift and Page keys", async () => {
  const handle = mountInteraction(ColorWheel, { props: { modelValue: 0 } });
  const slider = handle.getByRole("slider");

  slider.dispatchEvent(
    new KeyboardEvent("keydown", {
      key: "ArrowRight",
      shiftKey: true,
      bubbles: true,
      cancelable: true,
    }),
  );
  await nextTick();
  assert.equal(handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0], 10);

  slider.dispatchEvent(
    new KeyboardEvent("keydown", { key: "PageDown", bubbles: true, cancelable: true }),
  );
  await nextTick();
  assert.equal(handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0], 0);

  slider.dispatchEvent(
    new KeyboardEvent("keydown", { key: "PageUp", bubbles: true, cancelable: true }),
  );
  await nextTick();
  assert.equal(handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0], 10);
  handle.unmount();
});

test("reads the pointer as a clockwise hue", async () => {
  const handle = mountInteraction(ColorWheel, { props: { modelValue: 90 } });
  const slider = handle.getByRole("slider");
  const rect = slider.getBoundingClientRect();
  slider.dispatchEvent(
    new PointerEvent("pointerdown", {
      clientX: rect.left + rect.width / 2 + 20,
      clientY: rect.top + rect.height / 2,
      bubbles: true,
      pointerId: 1,
    }),
  );
  await nextTick();
  const hue = handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0];
  assert.equal(typeof hue, "number");
  close(hue as number, 0);
  handle.unmount();
});

test("disabled wheel ignores keyboard and pointer", async () => {
  const handle = mountInteraction(ColorWheel, { props: { modelValue: 20, disabled: true } });
  const slider = handle.getByRole("slider");

  assert.equal(slider.getAttribute("aria-disabled"), "true");
  assert.equal(slider.getAttribute("tabindex"), "-1");
  await handle.press(slider, "ArrowRight");
  slider.dispatchEvent(new PointerEvent("pointerdown", { clientX: 10, clientY: 0, bubbles: true }));
  await nextTick();
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
  handle.unmount();
});

test("passes hue and color to the slot", () => {
  const handle = mountInteraction(ColorWheel, {
    props: { modelValue: 200 },
    slots: {
      default: (props: { hue: number; color: string }) =>
        h("span", { "data-color": props.color }, String(props.hue)),
    },
  });

  assert.equal(handle.root().querySelector("[data-color]")?.textContent, "200");
  assert.equal(
    handle.root().querySelector("[data-color]")?.getAttribute("data-color"),
    "hsl(200 100% 50%)",
  );
  handle.unmount();
});
