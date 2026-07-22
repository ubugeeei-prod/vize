import assert from "node:assert/strict";
import { test } from "node:test";
import { h, nextTick, ref } from "@vue/runtime-core";

import { dispatchKey, firstChild, mountFresco, typeChars } from "../testing/mount.js";
import { PasswordInput, TextInput } from "./TextInput.js";

/** Mount TextInput with v-model wiring and emit capture. */
function mountInput(initial = "", props: Record<string, unknown> = {}) {
  const value = ref(initial);
  const emitted: Array<[string, unknown]> = [];
  const mounted = mountFresco(() =>
    h(TextInput, {
      modelValue: value.value,
      focused: true,
      ...props,
      "onUpdate:modelValue": (next: string) => {
        value.value = next;
      },
      onSubmit: (submitted: string) => emitted.push(["submit", submitted]),
      onCancel: () => emitted.push(["cancel", undefined]),
    }),
  );
  return { mounted, value, emitted, node: () => firstChild(mounted) };
}

void test("renders value, cursor, and placeholder state onto the input node", () => {
  const { mounted, node } = mountInput("hi", { placeholder: "name..." });

  const props = node().props;
  assert.equal(node().type, "input");
  assert.equal(props.value, "hi");
  assert.equal(props.cursor, 2, "cursor starts after the last grapheme");
  assert.equal(props.placeholder, "name...");
  assert.equal(props.focused, true);
  assert.equal(props.mask, false);
  assert.equal(props.maskChar, "*");
  mounted.unmount();
});

void test("typed characters update the model and the mounted node", async () => {
  const { mounted, value, node } = mountInput();

  await typeChars("abc");
  assert.equal(value.value, "abc");
  assert.equal(node().props.value, "abc");
  assert.equal(node().props.cursor, 3);
  mounted.unmount();
});

void test("ignores keyboard input while unfocused", async () => {
  const { mounted, value } = mountInput("keep", { focused: false });

  await typeChars("x");
  await dispatchKey({ key: "backspace" });
  assert.equal(value.value, "keep");
  mounted.unmount();
});

void test("moves the cursor with arrows, home, and end before editing", async () => {
  const { mounted, value, node } = mountInput("ac");

  await dispatchKey({ key: "left" });
  assert.equal(node().props.cursor, 1);
  await typeChars("b");
  assert.equal(value.value, "abc");

  await dispatchKey({ key: "home" });
  assert.equal(node().props.cursor, 0);
  await typeChars("_");
  assert.equal(value.value, "_abc");

  await dispatchKey({ key: "end" });
  assert.equal(node().props.cursor, 4);
  mounted.unmount();
});

void test("backspace and delete edit by grapheme cluster", async () => {
  const { mounted, value, node } = mountInput("a👋🏽b");

  assert.equal(node().props.cursor, 3, "emoji with modifier counts as one grapheme");
  await dispatchKey({ key: "backspace" });
  assert.equal(value.value, "a👋🏽");
  await dispatchKey({ key: "backspace" });
  assert.equal(value.value, "a");

  await dispatchKey({ key: "home" });
  await dispatchKey({ key: "delete" });
  assert.equal(value.value, "");
  await dispatchKey({ key: "delete" });
  assert.equal(value.value, "", "delete at the end is a no-op");
  mounted.unmount();
});

void test("emits submit with the current value and cancel on escape", async () => {
  const { mounted, emitted } = mountInput("ok");

  await dispatchKey({ key: "enter" });
  await dispatchKey({ key: "escape" });
  assert.deepEqual(emitted, [
    ["submit", "ok"],
    ["cancel", undefined],
  ]);
  mounted.unmount();
});

void test("external model updates clamp the cursor into range", async () => {
  const value = ref("longer");
  const mounted = mountFresco(() => h(TextInput, { modelValue: value.value, focused: true }));

  assert.equal(firstChild(mounted).props.cursor, 6);
  value.value = "ab";
  await nextTick();
  assert.equal(firstChild(mounted).props.value, "ab");
  assert.equal(firstChild(mounted).props.cursor, 2, "cursor clamps to the shorter value");

  await dispatchKey({ key: "left" });
  assert.equal(firstChild(mounted).props.cursor, 1, "clamped cursor keeps moving normally");
  mounted.unmount();
});

void test("PasswordInput masks by default and forwards its model", async () => {
  const value = ref("");
  const mounted = mountFresco(() =>
    h(PasswordInput, {
      modelValue: value.value,
      focus: true,
      "onUpdate:modelValue": (next: string) => {
        value.value = next;
      },
    }),
  );

  const node = firstChild(mounted);
  assert.equal(node.type, "input");
  assert.equal(node.props.mask, true);
  assert.equal(node.props.placeholder, "Enter password...");

  await typeChars("pw");
  assert.equal(value.value, "pw");
  mounted.unmount();
});
