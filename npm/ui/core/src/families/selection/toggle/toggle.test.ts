import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { nextTick } from "vue";

import ToggleButton from "./toggle-button.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("renders a native toggle button with pressed semantics", () => {
  const handle = mountInteraction(ToggleButton, { slots: { default: "Bold" } });
  const toggle = handle.getByRole("button", { name: "Bold" });

  assert.equal(toggle.tagName, "BUTTON");
  assert.equal(toggle.getAttribute("type"), "button");
  assert.equal(toggle.getAttribute("aria-pressed"), "false");
  assert.equal(toggle.getAttribute("data-vize-ui"), "toggle");
  assert.equal(toggle.getAttribute("data-state"), "unpressed");
  handle.unmount();
});

test("toggles with pointer activation and emits the requested value", async () => {
  const handle = mountInteraction(ToggleButton, {
    record: ["update:modelValue", "change"],
    slots: { default: "Bold" },
  });
  const toggle = handle.getByRole("button");

  await handle.click(toggle);
  assert.equal(toggle.getAttribute("aria-pressed"), "true");
  assert.equal(toggle.getAttribute("data-state"), "pressed");

  await handle.click(toggle);
  assert.equal(toggle.getAttribute("aria-pressed"), "false");

  const recorded = handle.recorded();
  assert.deepEqual(
    recorded.map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", true],
      ["change", true],
      ["update:modelValue", false],
      ["change", false],
    ],
  );
  assert.ok(recorded[1]?.payload[1] instanceof MouseEvent);
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountInteraction(ToggleButton, {
    props: { modelValue: false },
    record: ["update:modelValue"],
    slots: { default: "Bold" },
  });
  const toggle = handle.getByRole("button");

  await handle.click(toggle);
  await nextTick();
  assert.deepEqual(
    handle.recorded().map((emit) => emit.payload[0]),
    [true],
  );
  assert.equal(toggle.getAttribute("aria-pressed"), "false");

  await handle.wrapper.setProps({ modelValue: true });
  assert.equal(toggle.getAttribute("aria-pressed"), "true");
  assert.equal(toggle.getAttribute("data-state"), "pressed");
  handle.unmount();
});

test("uncontrolled defaultPressed seeds state and reset restores it", async () => {
  const handle = mountInteraction(ToggleButton, {
    props: { defaultPressed: true },
    slots: { default: "Bold" },
  });
  const toggle = handle.getByRole("button");
  assert.equal(toggle.getAttribute("aria-pressed"), "true");

  await handle.click(toggle);
  assert.equal(toggle.getAttribute("aria-pressed"), "false");

  handle.exposes<{ reset: () => boolean }>().reset();
  await nextTick();
  assert.equal(toggle.getAttribute("aria-pressed"), "true");
  handle.unmount();
});

test("non-native toggle emulates Enter and Space activation timing", async () => {
  const handle = mountInteraction(ToggleButton, {
    props: { as: "div" },
    slots: { default: "Bold" },
  });
  const toggle = handle.getByRole("button", { name: "Bold" });

  assert.equal(toggle.tagName, "DIV");
  assert.equal(toggle.getAttribute("tabindex"), "0");
  assert.equal(toggle.getAttribute("aria-pressed"), "false");

  const enter = await handle.press(toggle, "Enter");
  assert.equal(enter.activated, false);
  assert.equal(toggle.getAttribute("aria-pressed"), "true");

  const space = await handle.press(toggle, " ");
  assert.equal(space.keydownPrevented, true);
  assert.equal(toggle.getAttribute("aria-pressed"), "false");
  assert.equal(handle.wrapper.emitted("change")?.length, 2);
  handle.unmount();
});

test("disabled native and non-native toggles suppress activation", async () => {
  const native = mountInteraction(ToggleButton, {
    props: { disabled: true },
    slots: { default: "Bold" },
  });
  const nativeToggle = native.getByRole("button");
  assert.ok(nativeToggle.hasAttribute("disabled"));
  assert.equal(nativeToggle.getAttribute("aria-disabled"), null);

  await native.click(nativeToggle);
  await native.press(nativeToggle, "Enter");
  assert.equal(nativeToggle.getAttribute("aria-pressed"), "false");
  assert.equal(native.wrapper.emitted("change"), undefined);
  assert.ok((await native.tab()) === null);
  native.unmount();

  const nonNative = mountInteraction(ToggleButton, {
    props: { as: "div", disabled: true },
    slots: { default: "Bold" },
  });
  const nonNativeToggle = nonNative.getByRole("button");
  assert.equal(nonNativeToggle.getAttribute("tabindex"), "-1");
  assert.equal(nonNativeToggle.getAttribute("aria-disabled"), "true");

  await nonNative.click(nonNativeToggle);
  await nonNative.press(nonNativeToggle, " ");
  assert.equal(nonNativeToggle.getAttribute("aria-pressed"), "false");
  assert.equal(nonNative.wrapper.emitted("change"), undefined);
  assert.ok((await nonNative.tab()) === null);
  nonNative.unmount();
});

test("exposes focus, setPressed, and slot state", async () => {
  const handle = mountInteraction(ToggleButton, {
    slots: {
      default: (state: { disabled: boolean; pressed: boolean }) =>
        `pressed:${state.pressed} disabled:${state.disabled}`,
    },
  });
  const toggle = handle.getByRole("button");

  assert.equal(handle.root().textContent, "pressed:false disabled:false");
  handle.exposes<{ setPressed: (value: boolean) => boolean }>().setPressed(true);
  await nextTick();
  assert.equal(toggle.getAttribute("aria-pressed"), "true");
  assert.equal(handle.root().textContent, "pressed:true disabled:false");

  toggle.blur();
  handle.exposes<{ focus: (options?: FocusOptions) => void }>().focus();
  assert.ok(handle.activeElement() === toggle);
  handle.unmount();
});
