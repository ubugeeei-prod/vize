import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import ActionButton from "./action-button.vue";
import { getButtonKeyboardAction } from "./button-keyboard.ts";
import { mountInteraction } from "./testing/mount.ts";

test("matches native keyboard activation timing", () => {
  assert.equal(getButtonKeyboardAction("Enter", "keydown"), "activate");
  assert.equal(getButtonKeyboardAction("Enter", "keyup"), "ignore");
  assert.equal(getButtonKeyboardAction(" ", "keydown"), "prevent");
  assert.equal(getButtonKeyboardAction(" ", "keyup"), "activate");
  assert.equal(getButtonKeyboardAction("Escape", "keydown"), "ignore");
});

test("renders a native button with an accessible name", () => {
  const handle = mountInteraction(ActionButton, { slots: { default: "Save" } });
  const button = handle.getByRole("button", { name: "Save" });

  assert.equal(button.tagName, "BUTTON");
  assert.equal(button.getAttribute("type"), "button");
  assert.equal(button.getAttribute("data-vize-ui"), "button");
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("aria-disabled"), null);
  assert.equal(button.getAttribute("aria-busy"), null);
  handle.unmount();
});

test("joins the tab order and focuses programmatically", async () => {
  const handle = mountInteraction(ActionButton, { slots: { default: "Save" } });
  const button = handle.getByRole("button");

  assert.ok((await handle.tab()) === button, "Tab must move focus to the button");
  assert.ok(handle.activeElement() === button);

  button.blur();
  handle.exposes<{ focus: (options?: FocusOptions) => void }>().focus();
  assert.ok(handle.activeElement() === button, "exposed focus() must focus the control");
  handle.unmount();
});

test("pointer click fires exactly one press", async () => {
  const handle = mountInteraction(ActionButton, { slots: { default: "Save" } });

  await handle.click(handle.getByRole("button"));

  const presses = handle.wrapper.emitted("press");
  assert.equal(presses?.length, 1);
  assert.ok(presses?.[0]?.[0] instanceof MouseEvent);
  handle.unmount();
});

test("Enter and Space each fire exactly one press on a native button", async () => {
  const handle = mountInteraction(ActionButton, { slots: { default: "Save" } });
  const button = handle.getByRole("button");
  button.focus();

  const enter = await handle.press(button, "Enter");
  assert.equal(enter.activated, true);
  assert.equal(handle.wrapper.emitted("press")?.length, 1);

  const space = await handle.press(button, " ");
  assert.equal(space.activated, true);
  assert.equal(handle.wrapper.emitted("press")?.length, 2);
  handle.unmount();
});

test("Enter and Space activate a non-native button through its own handlers", async () => {
  const handle = mountInteraction(ActionButton, {
    props: { as: "div" },
    slots: { default: "Save" },
  });
  const button = handle.getByRole("button", { name: "Save" });

  assert.equal(button.tagName, "DIV");
  assert.equal(button.getAttribute("role"), "button");
  assert.equal(button.getAttribute("tabindex"), "0");

  const enter = await handle.press(button, "Enter");
  assert.equal(enter.activated, false, "the component, not the harness, must synthesize clicks");
  assert.equal(handle.wrapper.emitted("press")?.length, 1);

  const space = await handle.press(button, " ");
  assert.equal(space.keydownPrevented, true, "Space keydown must be canceled to prevent scrolling");
  assert.equal(handle.wrapper.emitted("press")?.length, 2);

  const escape = await handle.press(button, "Escape");
  assert.equal(escape.keydownPrevented, false);
  assert.equal(handle.wrapper.emitted("press")?.length, 2);
  handle.unmount();
});

test("keyboard and pointer presses are indistinguishable MouseEvents in order", async () => {
  const handle = mountInteraction(ActionButton, {
    slots: { default: "Save" },
    record: ["press"],
  });
  const button = handle.getByRole("button");
  button.focus();

  await handle.click(button);
  await handle.press(button, "Enter");
  await handle.press(button, " ");

  const recorded = handle.recorded();
  assert.equal(recorded.length, 3);
  for (const emit of recorded) {
    assert.equal(emit.event, "press");
    assert.ok(emit.payload[0] instanceof MouseEvent);
  }
  handle.unmount();
});

test("disabled native button removes activation and keeps native semantics", async () => {
  const handle = mountInteraction(ActionButton, {
    props: { disabled: true },
    slots: { default: "Save" },
  });
  const button = handle.getByRole("button");

  assert.ok(button.hasAttribute("disabled"), "native disabled must be forwarded");
  assert.equal(button.getAttribute("aria-disabled"), null, "native disabled needs no aria mirror");
  assert.equal(button.getAttribute("data-state"), "disabled");

  await handle.click(button);
  await handle.press(button, "Enter");
  await handle.press(button, " ");
  assert.equal(handle.wrapper.emitted("press"), undefined);

  assert.ok((await handle.tab()) === null, "a disabled button must leave the tab order");
  handle.unmount();
});

test("disabled non-native button leaves the tab order and announces aria-disabled", async () => {
  const handle = mountInteraction(ActionButton, {
    props: { as: "div", disabled: true },
    slots: { default: "Save" },
  });
  const button = handle.getByRole("button");

  assert.equal(button.getAttribute("tabindex"), "-1");
  assert.equal(button.getAttribute("aria-disabled"), "true");

  await handle.click(button);
  await handle.press(button, "Enter");
  await handle.press(button, " ");
  assert.equal(handle.wrapper.emitted("press"), undefined);
  assert.ok((await handle.tab()) === null, "tabindex -1 must remove it from the tab order");
  handle.unmount();
});

test("loading button announces busy, stays focusable, and suppresses press", async () => {
  const handle = mountInteraction(ActionButton, {
    props: { loading: true },
    slots: { default: "Save" },
  });
  const button = handle.getByRole("button");

  assert.equal(button.getAttribute("aria-busy"), "true");
  assert.equal(button.getAttribute("aria-disabled"), "true");
  assert.equal(button.getAttribute("data-state"), "loading");
  assert.equal(button.hasAttribute("disabled"), false, "loading must not steal focus");

  button.focus();
  assert.ok(handle.activeElement() === button, "a busy button must keep accepting focus");

  await handle.click(button);
  await handle.press(button, "Enter");
  await handle.press(button, " ");
  assert.equal(handle.wrapper.emitted("press"), undefined);
  handle.unmount();
});

test("exposes disabled, loading, and unavailable to the default slot", async () => {
  const handle = mountInteraction(ActionButton, {
    slots: {
      default: (state: { disabled: boolean; loading: boolean; unavailable: boolean }) =>
        `disabled:${state.disabled} loading:${state.loading} unavailable:${state.unavailable}`,
    },
  });

  assert.equal(handle.root().textContent, "disabled:false loading:false unavailable:false");
  await handle.wrapper.setProps({ loading: true });
  assert.equal(handle.root().textContent, "disabled:false loading:true unavailable:true");
  handle.unmount();
});
