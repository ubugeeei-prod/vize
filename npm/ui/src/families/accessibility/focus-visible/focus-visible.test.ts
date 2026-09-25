import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { FocusVisibleProviderExpose, FocusVisibleState } from "./focus-visible.ts";
import FocusVisibleProvider from "./focus-visible-provider.vue";
import {
  isTextEntryElement,
  shouldShowFocusRing,
  useFocusVisible,
} from "./focus-visible-runtime.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function keyboard(target: EventTarget = document.body): void {
  target.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Tab" }));
}

function pointer(target: EventTarget = document.body, pointerType = "mouse"): void {
  target.dispatchEvent(
    new PointerEvent("pointerdown", { bubbles: true, pointerId: 1, pointerType }),
  );
}

function mountProvider(props: Record<string, unknown> = {}) {
  return mountInteraction(FocusVisibleProvider, {
    props,
    slots: {
      default: () => [
        h("button", { type: "button", id: "first" }, "First"),
        h("button", { type: "button", id: "second" }, "Second"),
        h("input", { id: "name", "aria-label": "Name" }),
      ],
    },
  });
}

function byId(id: string): HTMLElement {
  const element = document.getElementById(id);
  assert.ok(element instanceof HTMLElement);
  return element;
}

test("keyboard focus marks the focused descendant and publishes the modality", async () => {
  const handle = mountProvider();
  await nextTick();
  const root = handle.root();
  const first = byId("first");

  keyboard();
  first.focus();
  await nextTick();
  assert.equal(root.getAttribute("data-vize-ui"), "focus-visible-provider");
  assert.equal(root.getAttribute("data-vize-modality"), "keyboard");
  assert.equal(first.getAttribute("data-focus-visible"), "true");
  assert.equal(root.getAttribute("data-focus-visible-within"), "true");

  const second = byId("second");
  second.focus();
  await nextTick();
  assert.equal(first.hasAttribute("data-focus-visible"), false, "moving focus clears the old mark");
  assert.equal(second.getAttribute("data-focus-visible"), "true");

  handle.unmount();
});

test("pointer focus on buttons is not marked and a pointer press clears the mark", async () => {
  const handle = mountProvider();
  await nextTick();
  const first = byId("first");

  keyboard();
  first.focus();
  await nextTick();
  assert.equal(first.getAttribute("data-focus-visible"), "true");

  pointer(first);
  await nextTick();
  assert.equal(first.hasAttribute("data-focus-visible"), false);
  assert.equal(handle.root().getAttribute("data-vize-modality"), "pointer");

  const second = byId("second");
  pointer(second);
  second.focus();
  await nextTick();
  assert.equal(second.hasAttribute("data-focus-visible"), false);

  handle.unmount();
});

test("text entry fields are marked for every modality, touch included", async () => {
  const handle = mountProvider();
  await nextTick();
  const input = byId("name");

  pointer(input, "touch");
  input.focus();
  await nextTick();
  assert.equal(handle.root().getAttribute("data-vize-modality"), "touch");
  assert.equal(input.getAttribute("data-focus-visible"), "true");

  handle.unmount();
});

test("blur leaving the subtree removes the mark", async () => {
  const outside = document.createElement("button");
  document.body.append(outside);
  const handle = mountProvider();
  await nextTick();
  const first = byId("first");

  keyboard();
  first.focus();
  await nextTick();
  outside.focus();
  await nextTick();
  assert.equal(first.hasAttribute("data-focus-visible"), false);
  assert.equal(handle.root().hasAttribute("data-focus-visible-within"), false);

  handle.unmount();
  outside.remove();
});

test("custom attribute names and disabled providers", async () => {
  const handle = mountProvider({ attribute: "data-ring" });
  await nextTick();
  const first = byId("first");

  keyboard();
  first.focus();
  await nextTick();
  assert.equal(first.getAttribute("data-ring"), "true");
  assert.equal(first.hasAttribute("data-focus-visible"), false);

  await handle.wrapper.setProps({ attribute: "data-outline" });
  assert.equal(first.hasAttribute("data-ring"), false, "renaming removes the old attribute");
  assert.equal(first.getAttribute("data-outline"), "true");

  await handle.wrapper.setProps({ disabled: true });
  assert.equal(first.hasAttribute("data-outline"), false);
  assert.equal(handle.root().hasAttribute("data-vize-modality"), false);
  assert.equal(handle.root().getAttribute("data-disabled"), "true");

  handle.unmount();
});

test("focus already inside the subtree at mount is picked up", async () => {
  const SelfFocus = defineComponent({
    name: "FocusVisibleSelfFocus",
    mounted() {
      const button = this.$el;
      if (button instanceof HTMLElement) button.focus();
    },
    render: () => h("button", { type: "button", id: "auto" }, "Auto"),
  });
  keyboard();
  const handle = mountInteraction(
    defineComponent({
      name: "FocusVisibleMountProbe",
      setup: () => () => h(FocusVisibleProvider, null, () => h(SelfFocus)),
    }),
  );
  await nextTick();
  const auto = byId("auto");
  assert.ok(document.activeElement === auto);
  assert.equal(auto.getAttribute("data-focus-visible"), "true");
  handle.unmount();
});

test("useFocusVisible reads the provider or tracks the document standalone", async () => {
  let inside: FocusVisibleState | null = null;
  let standalone: FocusVisibleState | null = null;
  let exposed: FocusVisibleProviderExpose | null = null;
  const Inner = defineComponent({
    name: "FocusVisibleInner",
    setup() {
      inside = useFocusVisible();
      return () => h("button", { type: "button", id: "inner" }, "Inner");
    },
  });
  const Outer = defineComponent({
    name: "FocusVisibleOuter",
    setup() {
      standalone = useFocusVisible();
      return () =>
        h(
          FocusVisibleProvider,
          {
            ref: (value) => {
              exposed = value as FocusVisibleProviderExpose | null;
            },
          },
          () => h(Inner),
        );
    },
  });
  const handle = mountInteraction(Outer);
  await nextTick();
  if (inside === null || standalone === null || exposed === null) assert.fail("state must exist");
  const provided: FocusVisibleState = inside;
  const document_: FocusVisibleState = standalone;
  const api: FocusVisibleProviderExpose = exposed;

  keyboard();
  byId("inner").focus();
  await nextTick();
  assert.equal(provided.isFocusVisible.value, true);
  assert.equal(document_.isFocusVisible.value, true);
  assert.equal(document_.modality.value, "keyboard");
  assert.ok(api.focusVisibleElement === byId("inner"));
  assert.equal(api.modality, "keyboard");

  pointer();
  await nextTick();
  assert.equal(provided.isFocusVisible.value, false);
  assert.equal(document_.isFocusVisible.value, false);

  handle.unmount();
  assert.throws(() => useFocusVisible(), /VIZE_UI_FOCUS_VISIBLE_SETUP/);
});

test("heuristics mirror browser focus-visible rules", () => {
  const button = document.createElement("button");
  const input = document.createElement("input");
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  const editable = document.createElement("div");
  editable.setAttribute("contenteditable", "");

  assert.equal(isTextEntryElement(input), true);
  assert.equal(isTextEntryElement(checkbox), false);
  assert.equal(isTextEntryElement(editable), true);
  assert.equal(isTextEntryElement(document.createElement("textarea")), true);
  assert.equal(shouldShowFocusRing(button, "keyboard"), true);
  assert.equal(shouldShowFocusRing(button, "virtual"), true);
  assert.equal(shouldShowFocusRing(button, "pointer"), false);
  assert.equal(shouldShowFocusRing(input, "pointer"), true);
  assert.equal(shouldShowFocusRing(button, null), true);
});
