import assert from "node:assert/strict";
import { defineComponent, h, nextTick, ref } from "vue";
import { test } from "vitest";

import { useFocus } from "../composables/useFocus.js";
import { dispatchKey, getByRole, mountFresco } from "../testing/index.js";
import Popover from "./FrescoPopover.vue";

function focusTarget(id: string, autoFocus = false) {
  return defineComponent({
    name: "FocusTarget",
    setup() {
      const focus = useFocus({ id, autoFocus });
      return () => h("text", { text: id, focused: focus.isFocused.value });
    },
  });
}

test("Popover anchors an overlay, traps initial focus, and restores the previous owner", async () => {
  const open = ref(false);
  const closes: string[] = [];
  const Outside = focusTarget("outside", true);
  const First = focusTarget("first");
  const Second = focusTarget("second");
  const mounted = mountFresco(() =>
    h("box", [
      h(Outside),
      h(
        Popover,
        {
          open: open.value,
          placement: "right",
          offset: 2,
          focusId: "actions-popover",
          initialFocusId: "first",
          accessibilityLabel: "Actions",
          onClose: () => {
            closes.push("close");
            open.value = false;
          },
        },
        {
          anchor: () => h("text", { text: "Open actions" }),
          default: () => [h(First), h(Second)],
        },
      ),
    ]),
  );

  assert.equal(mounted.focusManager.focusedId.value, "outside");
  open.value = true;
  await nextTick();
  await nextTick();

  assert.equal(mounted.focusManager.activeTrapId.value, "actions-popover");
  assert.equal(mounted.focusManager.focusedId.value, "first");
  mounted.focusManager.focus("outside");
  assert.equal(mounted.focusManager.focusedId.value, "first", "focus remains in the popover");
  mounted.focusManager.focusNext();
  assert.equal(mounted.focusManager.focusedId.value, "second");

  const panel = getByRole(mounted.root, "dialog", {
    name: "Actions",
    state: { expanded: true },
  });
  assert.equal((panel.props.style as Record<string, unknown>).position, "absolute");
  assert.equal((panel.props.style as Record<string, unknown>).left, "100%");
  assert.equal((panel.props.style as Record<string, unknown>).marginLeft, 2);

  await dispatchKey({ key: "escape" });
  await nextTick();
  assert.deepEqual(closes, ["close"]);
  assert.equal(mounted.focusManager.activeTrapId.value, undefined);
  assert.equal(mounted.focusManager.focusedId.value, "outside");
  mounted.unmount();
});

test("Popover can retain an open, non-trapping panel and clamps an invalid offset", async () => {
  const closes: string[] = [];
  const mounted = mountFresco(() =>
    h(
      Popover,
      {
        open: true,
        placement: "left",
        offset: Number.NaN,
        focusId: "persistent-popover",
        trapFocus: false,
        closeOnEscape: false,
        accessibilityRole: "menu",
        accessibilityLabel: "Persistent actions",
        border: "none",
        onClose: () => closes.push("close"),
      },
      { default: () => h("text", { text: "Contents" }) },
    ),
  );

  await dispatchKey({ key: "escape" });
  assert.deepEqual(closes, []);
  assert.equal(mounted.focusManager.activeTrapId.value, undefined);
  const panel = getByRole(mounted.root, "menu", { name: "Persistent actions" });
  assert.equal((panel.props.style as Record<string, unknown>).right, "100%");
  assert.equal((panel.props.style as Record<string, unknown>).marginRight, 0);
  assert.equal(panel.props.border, undefined);
  mounted.unmount();
});
