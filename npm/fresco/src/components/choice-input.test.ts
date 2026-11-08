import assert from "node:assert/strict";
import { test } from "vitest";
import { h, ref } from "vue";

import { dispatchKey, mountFresco, typeChars } from "../testing/mount.js";
import { Checkbox } from "./Checkbox.js";
import { Confirm } from "./Confirm.js";
import { RadioGroup } from "./RadioGroup.js";

test("Checkbox toggles with Space and Enter only while focused and enabled", async () => {
  const value = ref(false);
  const disabled = ref(false);
  const changes: boolean[] = [];
  const mounted = mountFresco(() =>
    h(Checkbox, {
      modelValue: value.value,
      focused: true,
      disabled: disabled.value,
      "onUpdate:modelValue": (next: boolean) => {
        value.value = next;
      },
      onChange: (next: boolean) => changes.push(next),
    }),
  );

  await typeChars(" ");
  assert.equal(value.value, true);
  await dispatchKey({ key: "enter" });
  assert.equal(value.value, false);

  disabled.value = true;
  await typeChars(" ");
  assert.equal(value.value, false);
  assert.deepEqual(changes, [true, false]);
  mounted.unmount();
});

test("RadioGroup navigates by orientation, skips disabled options, and activates", async () => {
  const value = ref("alpha");
  const focusedIndex = ref(0);
  const mounted = mountFresco(() =>
    h(RadioGroup, {
      options: [
        { label: "Alpha", value: "alpha" },
        { label: "Beta", value: "beta", disabled: true },
        { label: "Charlie", value: "charlie" },
      ],
      modelValue: value.value,
      focusedIndex: focusedIndex.value,
      focused: true,
      "onUpdate:modelValue": (next: string) => {
        value.value = next;
      },
      "onUpdate:focusedIndex": (next: number) => {
        focusedIndex.value = next;
      },
    }),
  );

  await dispatchKey({ key: "down" });
  assert.equal(focusedIndex.value, 2);
  await dispatchKey({ key: "enter" });
  assert.equal(value.value, "charlie");
  await typeChars("a");
  assert.equal(focusedIndex.value, 0);
  await typeChars(" ");
  assert.equal(value.value, "alpha");
  mounted.unmount();
});

test("Confirm switches choices and emits the selected outcome", async () => {
  const outcomes: Array<[string, boolean | undefined]> = [];
  const mounted = mountFresco(() =>
    h(Confirm, {
      message: "Continue?",
      focused: true,
      onSelect: (confirmed: boolean) => outcomes.push(["select", confirmed]),
      onConfirm: () => outcomes.push(["confirm", undefined]),
      onCancel: () => outcomes.push(["cancel", undefined]),
    }),
  );

  await dispatchKey({ key: "right" });
  await dispatchKey({ key: "enter" });
  await dispatchKey({ key: "escape" });
  assert.deepEqual(outcomes, [
    ["select", false],
    ["cancel", undefined],
    ["cancel", undefined],
  ]);
  mounted.unmount();
});
