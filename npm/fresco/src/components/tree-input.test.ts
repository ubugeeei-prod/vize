import assert from "node:assert/strict";
import { test } from "vitest";
import { h, ref, shallowRef } from "vue";

import { dispatchKey, findNodes, mountFresco, typeChars } from "../testing/mount.js";
import { Tree } from "./Tree.js";

function highlightedText(mounted: ReturnType<typeof mountFresco>): string | undefined {
  return findNodes(
    mounted.root,
    (node) => node.type === "text" && node.props.underline === true,
  ).at(0)?.text;
}

function textLines(mounted: ReturnType<typeof mountFresco>): string[] {
  return findNodes(mounted.root, (node) => node.type === "text")
    .map((node) => node.text)
    .filter((text): text is string => text !== undefined);
}

test("Tree navigates visible enabled nodes and owns expansion through controlled updates", async () => {
  const expanded = ref<string[]>([]);
  const selected = ref("root");
  const selections: string[] = [];
  const toggles: Array<[string, boolean]> = [];
  const mounted = mountFresco(() =>
    h(Tree, {
      data: [
        {
          key: "root",
          label: "Root",
          children: [
            { key: "disabled", label: "Disabled", disabled: true },
            { key: "child", label: "Child" },
            { key: "second", label: "Second" },
          ],
        },
        { key: "sibling", label: "Sibling" },
      ],
      expanded: expanded.value,
      selected: selected.value,
      focused: true,
      "onUpdate:expanded": (next: string[]) => {
        expanded.value = next;
      },
      "onUpdate:selected": (next: string) => {
        selected.value = next;
      },
      onSelect: (key: string) => selections.push(key),
      onToggle: (key: string, open: boolean) => toggles.push([key, open]),
    }),
  );

  assert.equal(highlightedText(mounted), "▶ Root");
  await dispatchKey({ key: "right" });
  assert.deepEqual(expanded.value, ["root"]);
  assert.equal(highlightedText(mounted), "▼ Root");

  await dispatchKey({ key: "right" });
  assert.equal(highlightedText(mounted), "  ├─• Child");
  await dispatchKey({ key: "enter" });
  assert.equal(selected.value, "child");
  assert.deepEqual(selections, ["child"]);

  await dispatchKey({ key: "left" });
  assert.equal(highlightedText(mounted), "▼ Root");
  await dispatchKey({ key: "left" });
  assert.deepEqual(expanded.value, []);
  assert.deepEqual(toggles, [
    ["root", true],
    ["root", false],
  ]);
  mounted.unmount();
});

test("Tree wraps around disabled nodes and supports visible-label typeahead", async () => {
  const mounted = mountFresco(() =>
    h(Tree, {
      data: [
        { key: "alpha", label: "Alpha" },
        { key: "blocked", label: "Blocked", disabled: true },
        { key: "charlie", label: "Charlie" },
      ],
      focused: true,
    }),
  );

  await dispatchKey({ key: "up" });
  assert.equal(highlightedText(mounted), "• Charlie");
  await typeChars("a");
  assert.equal(highlightedText(mounted), "• Alpha");
  await dispatchKey({ key: "end" });
  assert.equal(highlightedText(mounted), "• Charlie");
  mounted.unmount();
});

test("Tree preserves object values while keeping the active node visible", async () => {
  const alpha = { id: 1, name: "Alpha" };
  const charlie = { id: 3, name: "Charlie" };
  const delta = { id: 4, name: "Delta" };
  const selected = shallowRef({ id: 1, name: "Alpha copy" });
  const selections: (typeof alpha)[] = [];
  const mounted = mountFresco(() =>
    h(Tree, {
      data: [
        { key: "alpha", label: "Alpha", value: alpha },
        { key: "bravo", label: "Bravo", disabled: true, value: { id: 2, name: "Bravo" } },
        { key: "charlie", label: "Charlie", value: charlie },
        { key: "delta", label: "Delta", value: delta },
      ],
      selected: selected.value,
      by: "id",
      focused: true,
      maxHeight: 2,
      "onUpdate:selected": (value: typeof alpha) => {
        selected.value = value;
      },
      onSelect: (value: typeof alpha) => selections.push(value),
    }),
  );

  assert.deepEqual(textLines(mounted), ["• Alpha", "• Bravo", "  ..."]);
  await dispatchKey({ key: "down" });
  assert.deepEqual(textLines(mounted), ["  ...", "• Bravo", "• Charlie", "  ..."]);
  await dispatchKey({ key: "enter" });
  assert.equal(selected.value, charlie);
  assert.equal(selections[0], charlie);

  await typeChars("d");
  assert.deepEqual(textLines(mounted), ["  ...", "• Charlie", "• Delta"]);
  mounted.unmount();
});
