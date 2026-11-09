import assert from "node:assert/strict";
import { defineComponent, h, nextTick, ref } from "vue";
import { test } from "vitest";

import { useFocus } from "../composables/useFocus.js";
import { dispatchKey, findNodes, mountFresco } from "../testing/index.js";
import Modal from "./FrescoModal.vue";

function focusTarget(id: string, autoFocus = false) {
  return defineComponent({
    name: "FocusTarget",
    setup() {
      const focus = useFocus({ id, autoFocus });
      return () => h("text", { text: id, focused: focus.isFocused.value });
    },
  });
}

test("Modal traps nested focus, selects an initial target, and restores ownership", async () => {
  const visible = ref(false);
  const closes: string[] = [];
  const Outside = focusTarget("outside", true);
  const First = focusTarget("first");
  const Second = focusTarget("second");
  const mounted = mountFresco(() =>
    h("box", [
      h(Outside),
      h(
        Modal,
        {
          visible: visible.value,
          title: "Settings",
          focusId: "dialog",
          initialFocusId: "first",
          onClose: () => {
            closes.push("close");
            visible.value = false;
          },
        },
        () => [h(First), h(Second)],
      ),
    ]),
  );

  assert.equal(mounted.focusManager.focusedId.value, "outside");
  visible.value = true;
  await nextTick();
  await nextTick();

  assert.equal(mounted.focusManager.activeTrapId.value, "dialog");
  assert.equal(mounted.focusManager.focusedId.value, "first");
  assert.equal(mounted.focusManager.hasFocusedDescendant("dialog"), true);
  mounted.focusManager.focus("outside");
  assert.equal(mounted.focusManager.focusedId.value, "first", "focus cannot escape the modal");
  mounted.focusManager.focusNext();
  assert.equal(mounted.focusManager.focusedId.value, "second");

  const dialog = findNodes(mounted.root, (node) => node.props["aria-role"] === "dialog").at(0);
  assert.equal(dialog?.props["aria-label"], "Settings");
  assert.deepEqual(dialog?.props["aria-state"], { expanded: true });

  await dispatchKey({ key: "escape" });
  await nextTick();
  assert.deepEqual(closes, ["close"]);
  assert.equal(mounted.focusManager.activeTrapId.value, undefined);
  assert.equal(mounted.focusManager.focusedId.value, "outside");
  mounted.unmount();
});

test("Modal can retain visibility when the host disables Escape closing", async () => {
  const closes: string[] = [];
  const mounted = mountFresco(() =>
    h(Modal, {
      visible: true,
      focusId: "persistent-dialog",
      closeOnEscape: false,
      onClose: () => closes.push("close"),
    }),
  );

  await dispatchKey({ key: "escape" });
  assert.deepEqual(closes, []);
  assert.equal(mounted.focusManager.activeTrapId.value, "persistent-dialog");
  mounted.unmount();
});
