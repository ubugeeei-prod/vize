import assert from "node:assert/strict";
import { test } from "vitest";
import { h, ref } from "vue";

import { dispatchKey, findNodes, mountFresco, typeChars } from "../testing/mount.js";
import { Menu } from "./Menu.js";
import { Tabs } from "./Tabs.js";

function textLines(mounted: ReturnType<typeof mountFresco>): string[] {
  return findNodes(mounted.root, (node) => node.type === "text")
    .map((node) => node.text)
    .filter((text): text is string => text !== undefined);
}

test("Tabs navigates enabled labels and activates with Enter or Space", async () => {
  const value = ref("overview");
  const changes: string[] = [];
  const mounted = mountFresco(() =>
    h(Tabs, {
      tabs: [
        { key: "overview", label: "Overview" },
        { key: "logs", label: "Logs", disabled: true },
        { key: "settings", label: "Settings" },
      ],
      modelValue: value.value,
      focused: true,
      "onUpdate:modelValue": (next: string) => {
        value.value = next;
      },
      onChange: (next: string) => changes.push(next),
    }),
  );

  await dispatchKey({ key: "right" });
  assert.equal(textLines(mounted).at(4), "Settings");
  await dispatchKey({ key: "enter" });
  assert.equal(value.value, "settings");

  await dispatchKey({ key: "home" });
  await dispatchKey({ char: " " });
  assert.equal(value.value, "overview");
  assert.deepEqual(changes, ["settings", "overview"]);
  mounted.unmount();
});

test("Menu skips separators and disabled commands during navigation and typeahead", async () => {
  const focusedIndex = ref(0);
  const selected: string[] = [];
  const mounted = mountFresco(() =>
    h(Menu, {
      items: [
        { key: "open", label: "Open" },
        { key: "separator", label: "", separator: true },
        { key: "delete", label: "Delete", disabled: true },
        { key: "save", label: "Save" },
      ],
      focused: true,
      focusedIndex: focusedIndex.value,
      "onUpdate:focusedIndex": (next: number) => {
        focusedIndex.value = next;
      },
      onSelect: (key: string) => selected.push(key),
    }),
  );

  await dispatchKey({ key: "down" });
  assert.equal(focusedIndex.value, 3);
  await dispatchKey({ key: "enter" });
  assert.deepEqual(selected, ["save"]);

  await typeChars("o");
  assert.equal(focusedIndex.value, 0);
  await dispatchKey({ key: "enter" });
  assert.deepEqual(selected, ["save", "open"]);
  mounted.unmount();
});

test("Menu preserves object values while keeping the active command visible", async () => {
  const alpha = { id: 1, name: "Alpha" };
  const charlie = { id: 3, name: "Charlie" };
  const delta = { id: 4, name: "Delta" };
  const selected: (typeof alpha)[] = [];
  const mounted = mountFresco(() =>
    h(Menu, {
      items: [
        { key: "alpha", label: "Alpha", value: alpha },
        { key: "bravo", label: "Bravo", disabled: true, value: { id: 2, name: "Bravo" } },
        { key: "charlie", label: "Charlie", value: charlie },
        { key: "delta", label: "Delta", value: delta },
      ],
      focused: true,
      maxHeight: 2,
      onSelect: (value: typeof alpha) => selected.push(value),
    }),
  );

  assert.deepEqual(textLines(mounted), ["Alpha", "Bravo", "  ..."]);
  await dispatchKey({ key: "down" });
  assert.deepEqual(textLines(mounted), ["  ...", "Bravo", "Charlie", "  ..."]);
  await dispatchKey({ key: "enter" });
  assert.equal(selected[0], charlie);

  await typeChars("d");
  assert.deepEqual(textLines(mounted), ["  ...", "Charlie", "Delta"]);
  await dispatchKey({ key: "enter" });
  assert.equal(selected[1], delta);
  mounted.unmount();
});

test("Tabs does not paint a pending highlight as active after focus leaves", async () => {
  const focused = ref(true);
  const mounted = mountFresco(() =>
    h(Tabs, {
      tabs: [
        { key: "overview", label: "Overview" },
        { key: "settings", label: "Settings" },
      ],
      modelValue: "overview",
      focused: focused.value,
    }),
  );

  await dispatchKey({ key: "right" });
  focused.value = false;
  await dispatchKey({ key: "noop" });

  const labels = findNodes(mounted.root, (node) => node.type === "text");
  assert.equal(labels.at(0)?.props.fg, "cyan", "the active tab retains active styling");
  assert.equal(labels.at(2)?.props.fg, undefined, "the pending highlight loses active styling");
  mounted.unmount();
});
