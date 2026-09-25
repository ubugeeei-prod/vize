import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import CascaderColumn from "./cascader-column.vue";
import CascaderContent from "./cascader-content.vue";
import CascaderItem from "./cascader-item.vue";
import CascaderRoot from "./cascader-root.vue";
import CascaderTrigger from "./cascader-trigger.vue";
import CascaderValue from "./cascader-value.vue";
import { cascaderFixtureProps, cascaderFixtureSlot, places } from "./cascader-fixture-tree.ts";
import type { Place } from "./cascader-fixture-tree.ts";
import {
  findCascaderPath,
  flattenCascaderPaths,
  searchCascaderPaths,
  toCascaderSelection,
} from "./cascader-model.ts";
import type { CascaderLoadContext, CascaderSlotState } from "./cascader-types.ts";

const japan = places[0] as Place;
const tokyo = japan.children?.[0] as Place;
const shibuya = tokyo.children?.[0] as Place;
const shinjuku = tokyo.children?.[1] as Place;
const france = places[2] as Place;
const paris = france.children?.[0] as Place;

function mountCascader(props: Record<string, unknown> = {}) {
  return mountInteraction(CascaderRoot, {
    props: { ...cascaderFixtureProps, id: "place", ...props },
    record: ["update:modelValue", "change", "update:open", "load-error"],
    slots: { default: cascaderFixtureSlot({ portalDisabled: true }) },
  });
}

async function settle(): Promise<void> {
  for (let tick = 0; tick < 4; tick++) await nextTick();
}

function keydown(target: Element, key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init });
  target.dispatchEvent(event);
  return event;
}

function trigger(handle: { getByRole: (role: string, filter?: { name?: string }) => HTMLElement }) {
  return handle.getByRole("combobox", { name: "Place" });
}

function active(handle: { root: () => HTMLElement }, button: HTMLElement): string {
  const id = button.getAttribute("aria-activedescendant");
  return id === null ? "" : (handle.root().querySelector(`[id="${id}"]`)?.textContent ?? "");
}

function columnTexts(handle: { root: () => HTMLElement }): string[][] {
  return [...handle.root().querySelectorAll("[data-vize-ui='cascader-column']")].map((column) =>
    [...column.querySelectorAll("[role='option']")].map((option) => option.textContent ?? ""),
  );
}

test("renders a combobox trigger with placeholder and no popup while closed", async () => {
  const handle = mountCascader({ placeholder: "Choose a place" });
  const button = trigger(handle);
  assert.equal(button.id, "place-trigger");
  assert.equal(button.getAttribute("aria-haspopup"), "listbox");
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(button.textContent, "Choose a place");
  assert.equal(handle.root().querySelector("[data-vize-ui='cascader-column']"), null);
  handle.unmount();
});

test("click expands branches into new columns and selecting a leaf closes", async () => {
  const handle = mountCascader();
  const button = trigger(handle);
  await handle.click(button);
  await settle();
  assert.deepEqual(columnTexts(handle), [["Japan", "Mars", "France"]]);
  assert.equal(button.getAttribute("aria-controls"), "place-column-0");
  const column = handle.root().querySelector("[data-vize-ui='cascader-column']");
  assert.equal(column?.getAttribute("role"), "listbox");
  assert.equal(column?.getAttribute("aria-label"), "Options");

  await handle.click(handle.getByRole("option", { name: "Japan" }));
  await handle.click(handle.getByRole("option", { name: "Tokyo" }));
  await settle();
  assert.deepEqual(columnTexts(handle), [
    ["Japan", "Mars", "France"],
    ["Tokyo", "Osaka"],
    ["Shibuya", "Shinjuku"],
  ]);
  assert.equal(handle.getByRole("option", { name: "Japan" }).getAttribute("aria-expanded"), "true");
  assert.equal(handle.getByRole("listbox", { name: "Tokyo" }).id, "place-column-2");

  await handle.click(handle.getByRole("option", { name: "Shinjuku" }));
  await settle();
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(button.textContent, "Japan / Tokyo / Shinjuku");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[[japan, tokyo, shinjuku]]]);
  handle.unmount();
});

test("reopening restores the selected path and marks ancestors as partial", async () => {
  const handle = mountCascader({ defaultValue: [japan, tokyo, shibuya] });
  const button = trigger(handle);
  keydown(button, "Enter");
  await settle();
  assert.equal(columnTexts(handle).length, 3);
  assert.equal(active(handle, button), "Shibuya");
  assert.equal(
    handle.getByRole("option", { name: "Shibuya" }).getAttribute("aria-selected"),
    "true",
  );
  assert.equal(handle.getByRole("option", { name: "Japan" }).getAttribute("data-state"), "partial");
  handle.unmount();
});

test("keyboard matrix: open, arrows skip disabled, right/left traverse levels, Enter selects", async () => {
  const handle = mountCascader();
  const button = trigger(handle);
  assert.equal(keydown(button, "ArrowDown").defaultPrevented, true);
  await settle();
  assert.equal(active(handle, button), "Japan");
  keydown(button, "ArrowDown");
  await settle();
  assert.equal(active(handle, button), "France", "Mars is disabled");
  keydown(button, "ArrowDown");
  assert.equal(active(handle, button), "France", "edges hold");
  keydown(button, "Home");
  await settle();
  assert.equal(active(handle, button), "Japan");
  keydown(button, "End");
  keydown(button, "Home");
  keydown(button, "ArrowRight");
  await settle();
  assert.equal(active(handle, button), "Tokyo");
  keydown(button, "Enter");
  await settle();
  assert.equal(active(handle, button), "Shibuya", "Enter on a branch descends");
  keydown(button, "ArrowLeft");
  await settle();
  assert.equal(active(handle, button), "Tokyo");
  assert.equal(columnTexts(handle).length, 2, "ArrowLeft closes the child column");
  keydown(button, "ArrowDown");
  await settle();
  assert.equal(active(handle, button), "Osaka");
  keydown(button, "ArrowRight");
  await settle();
  const space = keydown(button, " ");
  await settle();
  assert.equal(space.defaultPrevented, true);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [
    [[japan, japan.children?.[1], japan.children?.[1]?.children?.[0]]],
  ]);
  assert.equal(button.getAttribute("aria-expanded"), "false");
  handle.unmount();
});

test("typeahead moves within the active column and Escape closes without selecting", async () => {
  const handle = mountCascader();
  const button = trigger(handle);
  keydown(button, "ArrowDown");
  await settle();
  keydown(button, "f");
  await settle();
  assert.equal(active(handle, button), "France");
  keydown(button, "ArrowRight");
  await settle();
  keydown(button, "l");
  await settle();
  assert.equal(active(handle, button), "Lyon");
  const escape = keydown(button, "Escape");
  await settle();
  assert.equal(escape.defaultPrevented, true);
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
  handle.unmount();
});

test("changeOnSelect commits branch paths while keeping the popup open", async () => {
  const handle = mountCascader({ changeOnSelect: true });
  await handle.click(trigger(handle));
  await settle();
  await handle.click(handle.getByRole("option", { name: "France" }));
  await settle();
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[[france]]]);
  assert.equal(trigger(handle).getAttribute("aria-expanded"), "true");
  assert.equal(trigger(handle).textContent, "France");
  handle.unmount();
});

test("multiple mode toggles leaf paths and stays open", async () => {
  const handle = mountCascader({ defaultValue: [[france, paris]], multiple: true, name: "places" });
  const button = trigger(handle);
  await handle.click(button);
  await settle();
  assert.equal(
    handle
      .root()
      .querySelector("[data-vize-ui='cascader-column']")
      ?.getAttribute("aria-multiselectable"),
    "true",
  );
  await handle.click(handle.getByRole("option", { name: "Japan" }));
  await handle.click(handle.getByRole("option", { name: "Tokyo" }));
  await handle.click(handle.getByRole("option", { name: "Shibuya" }));
  await settle();
  await handle.click(handle.getByRole("option", { name: "France" }));
  await handle.click(handle.getByRole("option", { name: "Paris" }));
  await settle();
  assert.equal(button.getAttribute("aria-expanded"), "true");
  assert.deepEqual(
    handle.wrapper.emitted("update:modelValue")?.map(([value]) => value),
    [
      [
        [france, paris],
        [japan, tokyo, shibuya],
      ],
      [[japan, tokyo, shibuya]],
    ],
  );
  const hidden = [
    ...handle.root().querySelectorAll<HTMLInputElement>("[data-vize-ui='cascader-native']"),
  ].map((input) => [input.name, input.value]);
  assert.deepEqual(hidden, [["places", "jp / tokyo / shibuya"]]);
  handle.unmount();
});

test("hover expandTrigger expands branches on pointer movement", async () => {
  const handle = mountCascader({ defaultOpen: true, expandTrigger: "hover" });
  await settle();
  handle
    .getByRole("option", { name: "France" })
    .dispatchEvent(new PointerEvent("pointermove", { bubbles: true, pointerType: "mouse" }));
  await settle();
  assert.deepEqual(columnTexts(handle)[1], ["Paris", "Lyon"]);
  assert.equal(active(handle, trigger(handle)), "France");
  handle.unmount();
});

interface Region {
  readonly id: string;
  readonly name: string;
  readonly leaf?: boolean;
}

test("loadChildren loads lazily, marks loading, descends by keyboard, and aborts superseded loads", async () => {
  const calls: {
    node: Region;
    signal: AbortSignal;
    resolve: (value: readonly Region[]) => void;
  }[] = [];
  const roots: readonly Region[] = [
    { id: "a", name: "Alpha" },
    { id: "b", name: "Beta" },
  ];
  const handle = mountInteraction(CascaderRoot, {
    props: {
      by: "id",
      defaultOpen: true,
      isLeaf: (region: Region) => region.leaf === true,
      itemText: (region: Region) => region.name,
      loadChildren: (node: Region, context: CascaderLoadContext) =>
        new Promise<readonly Region[]>((resolve) => {
          calls.push({ node, resolve, signal: context.signal });
        }),
      options: roots,
    },
    slots: {
      default: (state: CascaderSlotState<Region>) => [
        h(CascaderTrigger, { ariaLabel: "Region" }, () => h(CascaderValue)),
        h(CascaderContent, { portalDisabled: true }, () =>
          state.columns.map((column) =>
            h(CascaderColumn, { key: column.level, level: column.level }, () =>
              column.options.map((region) =>
                h(CascaderItem<Region>, { key: region.id, value: region }, () => region.name),
              ),
            ),
          ),
        ),
      ],
    },
  });
  await settle();
  const button = handle.getByRole("combobox", { name: "Region" });
  keydown(button, "ArrowRight");
  await settle();
  assert.equal(calls.length, 1);
  const alpha = handle.getByRole("option", { name: "Alpha" });
  assert.equal(alpha.getAttribute("data-loading"), "true");
  const loadingColumn = handle.root().querySelectorAll("[data-vize-ui='cascader-column']")[1];
  assert.equal(loadingColumn?.getAttribute("aria-busy"), "true");

  keydown(button, "ArrowDown");
  await settle();
  keydown(button, "ArrowRight");
  await settle();
  assert.equal(calls[0]?.signal.aborted, true, "expanding Beta supersedes Alpha's load");
  calls[1]?.resolve([{ id: "b1", leaf: true, name: "Beta One" }]);
  await settle();
  const id = button.getAttribute("aria-activedescendant");
  assert.equal(handle.root().querySelector(`[id="${id}"]`)?.textContent, "Beta One");
  keydown(button, "Enter");
  await settle();
  assert.equal(button.textContent, "Beta / Beta One");
  handle.unmount();
});

test("unmount aborts pending loads and rejected loads emit load-error", async () => {
  const signals: AbortSignal[] = [];
  let reject: (error: unknown) => void = () => {};
  const handle = mountCascader({
    defaultOpen: true,
    getChildren: () => undefined,
    loadChildren: (_node: Place, context: CascaderLoadContext) => {
      signals.push(context.signal);
      return new Promise<readonly Place[]>((_, onReject) => {
        reject = onReject;
      });
    },
  });
  await settle();
  await handle.click(handle.getByRole("option", { name: "Japan" }));
  await settle();
  reject(new Error("offline"));
  await settle();
  assert.equal(handle.wrapper.emitted("load-error")?.[0]?.[1] instanceof Error, true);
  await handle.click(handle.getByRole("option", { name: "France" }));
  await settle();
  handle.unmount();
  assert.equal(signals.at(-1)?.aborted, true);
});

test("search publishes matching paths that can be selected", async () => {
  let latest: CascaderSlotState<Place> | null = null;
  const handle = mountInteraction(CascaderRoot, {
    props: { ...cascaderFixtureProps, search: "shi" },
    slots: {
      default: (state: CascaderSlotState<Place>) => {
        latest = state;
        return [h(CascaderTrigger, { ariaLabel: "Place" }, () => h(CascaderValue))];
      },
    },
    record: ["update:modelValue"],
  });
  await settle();
  const state: CascaderSlotState<Place> | null = latest;
  if (state === null) assert.fail("slot state expected");
  assert.deepEqual(
    state.searchResults.map((path) => path.map((place) => place.label).join("/")),
    ["Japan/Tokyo/Shibuya", "Japan/Tokyo/Shinjuku"],
  );
  state.selectPath(state.searchResults[1] ?? []);
  await settle();
  assert.equal(trigger(handle).textContent, "Japan / Tokyo / Shinjuku");
  handle.unmount();
});

test("controlled values wait for the parent; disabled cascaders never open", async () => {
  const handle = mountCascader({ defaultOpen: true, modelValue: [] });
  await settle();
  await handle.click(handle.getByRole("option", { name: "France" }));
  await handle.click(handle.getByRole("option", { name: "Lyon" }));
  await settle();
  assert.equal(handle.wrapper.emitted("update:modelValue")?.length, 1);
  assert.equal(trigger(handle).getAttribute("data-placeholder"), "true");
  handle.unmount();

  const disabled = mountCascader({ disabled: true });
  keydown(trigger(disabled), "ArrowDown");
  await settle();
  assert.equal(trigger(disabled).getAttribute("aria-expanded"), "false");
  disabled.unmount();
});

test("parts require a Cascader provider", () => {
  assert.throws(
    () => mountInteraction(CascaderColumn, { props: { level: 0 } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});

test("pure helpers resolve, flatten, search, and normalize paths", () => {
  const path = findCascaderPath(places, (node) => node.value === "lyon");
  assert.deepEqual(
    path?.map((node) => node.value),
    ["fr", "lyon"],
  );
  assert.equal(
    findCascaderPath(places, () => false),
    null,
  );
  const leaves = flattenCascaderPaths(places);
  assert.equal(leaves.length, 6);
  assert.equal(flattenCascaderPaths(places, undefined, true).length, 10);
  assert.deepEqual(
    searchCascaderPaths(leaves, "lyo", (node: Place) => node.label).map((entry) => entry.length),
    [2],
  );
  assert.deepEqual(
    searchCascaderPaths(leaves, " ", (node: Place) => node.label),
    [],
  );
  assert.deepEqual(toCascaderSelection([france, paris], false), [[france, paris]]);
  assert.deepEqual(toCascaderSelection([], true), []);
});
