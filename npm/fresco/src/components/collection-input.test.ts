import assert from "node:assert/strict";
import { test } from "vitest";
import { h, ref, shallowRef } from "vue";

import { dispatchKey, findNodes, mountFresco, typeChars } from "../testing/mount.js";
import { List } from "./List.js";
import { Select } from "./Select.js";

function textLines(mounted: ReturnType<typeof mountFresco>): string[] {
  return findNodes(mounted.root, (node) => node.type === "text")
    .map((node) => node.text)
    .filter((text): text is string => text !== undefined);
}

test("Select skips disabled options, supports boundaries and activates the highlight", async () => {
  const value = ref("alpha");
  const selected: string[] = [];
  const mounted = mountFresco(() =>
    h(Select, {
      options: [
        { label: "Alpha", value: "alpha" },
        { label: "Beta", value: "beta", disabled: true },
        { label: "Gamma", value: "gamma" },
      ],
      modelValue: value.value,
      focused: true,
      "onUpdate:modelValue": (next: string) => {
        value.value = next;
      },
      onSelect: (next: string) => selected.push(next),
    }),
  );

  await dispatchKey({ key: "down" });
  assert.equal(textLines(mounted).at(2), "> Gamma");
  await dispatchKey({ key: "enter" });
  assert.equal(value.value, "gamma");
  assert.deepEqual(selected, ["gamma"]);

  await dispatchKey({ key: "home" });
  assert.equal(textLines(mounted).at(0), "> Alpha");
  await dispatchKey({ key: "end" });
  assert.equal(textLines(mounted).at(2), "> Gamma (selected)");
  mounted.unmount();
});

test("Select typeahead cycles matching enabled options and ignores input while blurred", async () => {
  const focused = ref(true);
  const mounted = mountFresco(() =>
    h(Select, {
      options: [
        { label: "Alpha", value: "alpha" },
        { label: "Bravo", value: "bravo" },
        { label: "Beta", value: "beta", disabled: true },
        { label: "Brook", value: "brook" },
      ],
      focused: focused.value,
    }),
  );

  await typeChars("b");
  assert.equal(textLines(mounted).at(1), "> Bravo");
  await typeChars("b");
  assert.equal(textLines(mounted).at(3), "> Brook");

  focused.value = false;
  await typeChars("a");
  assert.equal(textLines(mounted).at(3), "> Brook");
  mounted.unmount();
});

test("Select keeps keyboard and typeahead targets inside a bounded viewport", async () => {
  const mounted = mountFresco(() =>
    h(Select, {
      options: [
        { label: "Alpha", value: "alpha" },
        { label: "Beta", value: "beta", disabled: true },
        { label: "Charlie", value: "charlie" },
        { label: "Delta", value: "delta" },
      ],
      focused: true,
      maxHeight: 2,
    }),
  );

  assert.deepEqual(textLines(mounted), ["> Alpha", "  Beta", "  ..."]);
  await dispatchKey({ key: "down" });
  assert.deepEqual(textLines(mounted), ["  ...", "  Beta", "> Charlie", "  ..."]);
  await typeChars("d");
  assert.deepEqual(textLines(mounted), ["  ...", "  Charlie", "> Delta"]);
  await dispatchKey({ key: "home" });
  assert.deepEqual(textLines(mounted), ["> Alpha", "  Beta", "  ..."]);
  mounted.unmount();
});

test("Select preserves object values through keyed equality and activation", async () => {
  const ada = { id: 1, name: "Ada" };
  const grace = { id: 2, name: "Grace" };
  const modelValue = shallowRef({ id: 2, name: "Grace copy" });
  const selected: (typeof ada)[] = [];
  const mounted = mountFresco(() =>
    h(Select, {
      options: [
        { label: "Ada", value: ada },
        { label: "Grace", value: grace },
      ],
      modelValue: modelValue.value,
      by: "id",
      focused: true,
      "onUpdate:modelValue": (next: typeof ada) => {
        modelValue.value = next;
      },
      onSelect: (next: typeof ada) => selected.push(next),
    }),
  );

  assert.equal(textLines(mounted).at(1), "> Grace (selected)");
  await dispatchKey({ key: "home" });
  await dispatchKey({ key: "enter" });
  assert.equal(modelValue.value, ada);
  assert.deepEqual(selected, [ada]);
  mounted.unmount();
});

test("List keeps keyboard navigation visible and selects by stable key", async () => {
  const value = ref("a");
  const mounted = mountFresco(() =>
    h(List, {
      items: [
        { key: "a", label: "Alpha" },
        { key: "b", label: "Beta", disabled: true },
        { key: "c", label: "Charlie" },
        { key: "d", label: "Delta" },
      ],
      modelValue: value.value,
      focused: true,
      maxHeight: 2,
      "onUpdate:modelValue": (next: string) => {
        value.value = next;
      },
    }),
  );

  assert.deepEqual(textLines(mounted), ["> Alpha", "  Beta", "  ..."]);
  await dispatchKey({ key: "down" });
  assert.deepEqual(textLines(mounted), ["  ...", "  Beta", "> Charlie", "  ..."]);
  await dispatchKey({ key: "enter" });
  assert.equal(value.value, "c");

  await typeChars("d");
  assert.deepEqual(textLines(mounted), ["  ...", "  Charlie", "> Delta"]);

  await dispatchKey({ key: "pagedown" });
  assert.deepEqual(textLines(mounted), ["  ...", "  Charlie", "> Delta"]);
  await dispatchKey({ key: "pageup" });
  assert.deepEqual(textLines(mounted), ["> Alpha", "  Beta", "  ..."]);
  mounted.unmount();
});
