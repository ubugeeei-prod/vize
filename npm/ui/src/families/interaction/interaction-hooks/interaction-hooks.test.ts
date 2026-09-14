import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { mountInteraction } from "../../../testing/mount.ts";
import { createInteractionHooks } from "./interaction-hooks.ts";
import InteractionHooksExample from "./interaction-hooks-example.vue";

function bindProps(element: Element, props: Readonly<Record<string, unknown>>): () => void {
  const removals: Array<() => void> = [];
  for (const [key, value] of Object.entries(props)) {
    if (!key.startsWith("on") || typeof value !== "function") continue;
    const eventName = key.slice(2).toLowerCase();
    const listener = value as EventListener;
    element.addEventListener(eventName, listener);
    removals.push(() => element.removeEventListener(eventName, listener));
  }
  return () => {
    for (const remove of removals.splice(0)) remove();
  };
}

function pointer(type: string): PointerEvent {
  return new PointerEvent(type, { bubbles: true, pointerId: 7, pointerType: "mouse" });
}

test("composes press, hover, and focus-ring handlers for one host", () => {
  const phases: string[] = [];
  const controller = createInteractionHooks({
    press: { onPress: (event) => phases.push(event.pointerType) },
    hover: { onHoverStart: (event) => phases.push(event.type) },
    focusRing: { autoFocus: true, onFocus: (event) => phases.push(event.type) },
  });
  const button = document.createElement("button");
  document.body.append(button);
  const release = bindProps(button, controller.interactionProps);

  try {
    button.dispatchEvent(pointer("pointerenter"));
    button.focus();
    button.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 0 }));

    assert.equal(controller.isHovered.value, true);
    assert.equal(controller.isFocused.value, true);
    assert.deepEqual(phases, ["hoverstart", "focus", "virtual"]);
  } finally {
    controller.dispose();
    release();
    button.remove();
  }
});

test("literal false removes feature controllers and leaves remaining props stable", () => {
  const controller = createInteractionHooks({
    focusRing: false,
    focusWithin: {},
    hover: false,
    press: false,
  });

  assert.equal(controller.press, null);
  assert.equal(controller.hover, null);
  assert.equal(controller.focusRing, null);
  assert.notEqual(controller.focusWithin, null);
  assert.deepEqual(Object.keys(controller.interactionProps).sort(), ["onFocusin", "onFocusout"]);
  assert.deepEqual(controller.cancel(), {
    focusRing: false,
    focusWithin: false,
    hover: false,
    press: false,
  });
  controller.dispose();
});

test("example fixture reflects composed activation state in mounted DOM", async () => {
  const handle = mountInteraction(InteractionHooksExample);
  const button = handle.getByRole("button", { name: "Activated 0 times" });

  assert.equal(button.getAttribute("data-vize-ui"), "interaction-hooks-example");
  assert.equal(button.getAttribute("data-focused"), null);
  await handle.click(button);

  assert.equal(button.textContent?.trim(), "Activated 1 times");
  handle.unmount();
});
