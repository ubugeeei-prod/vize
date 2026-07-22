import assert from "node:assert/strict";
import { test } from "node:test";
import { defineComponent, h, nextTick, ref, type Ref } from "@vue/runtime-core";

import { mountFresco } from "../testing/mount.js";
import { createFocusManager, useFocus, type FocusManager } from "./useFocus.js";

void test("focusNext and focusPrevious traverse registration order and wrap", () => {
  const manager = createFocusManager();
  manager.register("a");
  manager.register("b");
  manager.register("c");

  assert.equal(manager.focusedId.value, null);
  manager.focusNext();
  assert.equal(manager.focusedId.value, "a");
  manager.focusNext();
  assert.equal(manager.focusedId.value, "b");
  manager.focusNext();
  manager.focusNext();
  assert.equal(manager.focusedId.value, "a", "traversal wraps past the end");

  manager.focusPrevious();
  assert.equal(manager.focusedId.value, "c", "backwards traversal wraps to the end");
});

void test("inactive focusables are skipped and lose focus when deactivated", () => {
  const manager = createFocusManager();
  manager.register("a");
  manager.register("b");
  manager.register("c");

  manager.setActive("b", false);
  assert.deepEqual(manager.focusableIds.value, ["a", "c"]);

  manager.focus("b");
  assert.equal(manager.focusedId.value, null, "inactive targets cannot be focused");

  manager.focus("a");
  manager.focusNext();
  assert.equal(manager.focusedId.value, "c", "traversal skips inactive entries");

  manager.setActive("c", false);
  assert.equal(manager.focusedId.value, null, "deactivation releases focus");
});

void test("disableFocus clears and blocks focus until re-enabled", () => {
  const manager = createFocusManager();
  manager.register("a", { autoFocus: true });
  assert.equal(manager.focusedId.value, "a");

  manager.disableFocus();
  assert.equal(manager.focusedId.value, null);
  manager.focus("a");
  manager.focusNext();
  assert.equal(manager.focusedId.value, null, "focus is inert while disabled");

  manager.enableFocus();
  manager.focus("a");
  assert.equal(manager.focusedId.value, "a");
});

interface FocusProbe {
  id: string;
  isFocused: Ref<boolean>;
  focus: (targetId?: string) => void;
  blur: () => void;
}

function focusableStub(id: string, probes: Map<string, FocusProbe>, isActive?: Ref<boolean>) {
  return defineComponent({
    name: `Focusable${id}`,
    setup() {
      probes.set(id, useFocus(isActive === undefined ? { id } : { id, isActive }));
      return () => h("text", { text: id });
    },
  });
}

void test("mounted components register with the app focus manager and unregister on unmount", async () => {
  const probes = new Map<string, FocusProbe>();
  const showSecond = ref(true);
  const First = focusableStub("first", probes);
  const Second = focusableStub("second", probes);

  const mounted = mountFresco(() => h("box", [h(First), showSecond.value ? h(Second) : undefined]));
  const manager: FocusManager = mounted.focusManager;

  assert.deepEqual(manager.focusableIds.value, ["first", "second"]);

  manager.focusNext();
  manager.focusNext();
  assert.equal(manager.focusedId.value, "second");
  assert.equal(probes.get("second")?.isFocused.value, true);
  assert.equal(probes.get("first")?.isFocused.value, false);

  showSecond.value = false;
  await nextTick();
  assert.deepEqual(manager.focusableIds.value, ["first"], "unmount unregisters the target");
  assert.equal(manager.focusedId.value, null, "focus is released with the unmounted target");

  mounted.unmount();
  assert.deepEqual(manager.focusableIds.value, []);
});

void test("focus and blur from useFocus move ownership through the manager", () => {
  const probes = new Map<string, FocusProbe>();
  const First = focusableStub("first", probes);
  const Second = focusableStub("second", probes);
  const mounted = mountFresco(() => h("box", [h(First), h(Second)]));

  probes.get("first")?.focus();
  assert.equal(mounted.focusManager.focusedId.value, "first");

  probes.get("first")?.focus("second");
  assert.equal(mounted.focusManager.focusedId.value, "second", "focus() can target another id");

  probes.get("second")?.blur();
  assert.equal(mounted.focusManager.focusedId.value, null);

  probes.get("first")?.blur();
  assert.equal(mounted.focusManager.focusedId.value, null, "blurring unfocused ids is a no-op");
  mounted.unmount();
});

void test("autoFocus focuses the first mounted target only", () => {
  const probes = new Map<string, FocusProbe>();
  const First = defineComponent({
    setup() {
      probes.set("first", useFocus({ id: "first", autoFocus: true }));
      return () => h("text", { text: "first" });
    },
  });
  const Second = defineComponent({
    setup() {
      probes.set("second", useFocus({ id: "second", autoFocus: true }));
      return () => h("text", { text: "second" });
    },
  });

  const mounted = mountFresco(() => h("box", [h(First), h(Second)]));
  assert.equal(mounted.focusManager.focusedId.value, "first");
  assert.equal(probes.get("first")?.isFocused.value, true);
  mounted.unmount();
});

void test("a reactive isActive toggles traversal membership from inside the tree", async () => {
  const probes = new Map<string, FocusProbe>();
  const active = ref(false);
  const First = focusableStub("first", probes);
  const Toggle = focusableStub("toggle", probes, active);
  const mounted = mountFresco(() => h("box", [h(First), h(Toggle)]));

  assert.deepEqual(mounted.focusManager.focusableIds.value, ["first"]);

  active.value = true;
  await nextTick();
  assert.deepEqual(mounted.focusManager.focusableIds.value, ["first", "toggle"]);

  mounted.focusManager.focus("toggle");
  active.value = false;
  await nextTick();
  assert.equal(mounted.focusManager.focusedId.value, null, "deactivation releases focus");
  assert.deepEqual(mounted.focusManager.focusableIds.value, ["first"]);
  mounted.unmount();
});
