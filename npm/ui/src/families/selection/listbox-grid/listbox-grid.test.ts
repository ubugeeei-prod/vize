import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import ListboxGrid from "./listbox-grid.vue";
import ListboxGridItem from "./listbox-grid-item.vue";
import { gridMoveFromKey, moveInGrid, serializeGridValue } from "./listbox-grid-model.ts";
import type { ListboxGridItemSlotState } from "./listbox-grid-types.ts";

interface Swatch {
  readonly id: string;
  readonly label: string;
  readonly disabled?: boolean;
}

// 3 columns:
// red    green  blue
// cyan   [gray] pink      (gray disabled)
// black  white
const swatches: readonly Swatch[] = [
  { id: "red", label: "Red" },
  { id: "green", label: "Green" },
  { id: "blue", label: "Blue" },
  { id: "cyan", label: "Cyan" },
  { disabled: true, id: "gray", label: "Gray" },
  { id: "pink", label: "Pink" },
  { id: "black", label: "Black" },
  { id: "white", label: "White" },
];

function mountGrid(props: Record<string, unknown> = {}) {
  return mountInteraction(ListboxGrid, {
    props: { ariaLabel: "Color", by: "id", columns: 3, id: "colors", ...props },
    record: ["update:modelValue", "change"],
    slots: {
      default: () =>
        swatches.map((swatch) =>
          h(
            ListboxGridItem<Swatch>,
            {
              ariaLabel: swatch.label,
              disabled: swatch.disabled === true,
              key: swatch.id,
              textValue: swatch.label,
              value: swatch,
            },
            {
              default: (state: ListboxGridItemSlotState<Swatch>) =>
                h("span", { "data-cell": `${state.row}:${state.column}` }),
            },
          ),
        ),
    },
  });
}

function keydown(target: Element, key: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...init });
  target.dispatchEvent(event);
  return event;
}

function active(handle: ReturnType<typeof mountGrid>): string | null {
  const id = handle.root().getAttribute("aria-activedescendant");
  return id === null ? null : (document.getElementById(id)?.getAttribute("aria-label") ?? null);
}

async function press(handle: ReturnType<typeof mountGrid>, key: string, init?: KeyboardEventInit) {
  const event = keydown(handle.root(), key, init);
  await nextTick();
  return event;
}

test("renders listbox semantics with row and column slot state", async () => {
  const handle = mountGrid({ defaultValue: swatches[2], name: "color" });
  await nextTick();
  const root = handle.root();
  assert.equal(root.getAttribute("role"), "listbox");
  assert.equal(root.getAttribute("data-vize-ui"), "listbox-grid");
  assert.equal(root.getAttribute("data-columns"), "3");
  assert.equal(root.tabIndex, 0);
  assert.equal(root.getAttribute("aria-multiselectable"), null);
  const blue = handle.getByRole("option", { name: "Blue" });
  assert.equal(blue.getAttribute("aria-selected"), "true");
  assert.equal(blue.querySelector("[data-cell]")?.getAttribute("data-cell"), "0:2");
  assert.equal(
    handle
      .getByRole("option", { name: "White" })
      .querySelector("[data-cell]")
      ?.getAttribute("data-cell"),
    "2:1",
  );
  const hidden = root.querySelector<HTMLInputElement>("[data-vize-ui='listbox-grid-native']");
  assert.equal(hidden?.name, "color");
  assert.equal(hidden?.value, "blue");
  handle.unmount();
});

test("focus highlights the selected option, then arrows move in two dimensions", async () => {
  const handle = mountGrid({ defaultValue: swatches[1] });
  handle.root().focus();
  await nextTick();
  assert.equal(active(handle), "Green");
  assert.equal((await press(handle, "ArrowDown")).defaultPrevented, true);
  assert.equal(active(handle), "White", "gray is disabled, so down skips to the next row");
  await press(handle, "ArrowUp");
  assert.equal(active(handle), "Green");
  await press(handle, "ArrowRight");
  assert.equal(active(handle), "Blue");
  await press(handle, "ArrowRight");
  assert.equal(active(handle), "Cyan", "right wraps to the next row");
  await press(handle, "ArrowRight");
  assert.equal(active(handle), "Pink", "disabled options are skipped horizontally");
  await press(handle, "Home");
  assert.equal(active(handle), "Cyan");
  await press(handle, "End");
  assert.equal(active(handle), "Pink");
  await press(handle, "Home", { ctrlKey: true });
  assert.equal(active(handle), "Red");
  await press(handle, "End", { ctrlKey: true });
  assert.equal(active(handle), "White");
  await press(handle, "PageUp");
  assert.equal(active(handle), "Green");
  await press(handle, "PageDown");
  assert.equal(active(handle), "White");
  await press(handle, "ArrowDown");
  assert.equal(active(handle), "White", "edges hold");
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
  handle.unmount();
});

test("rtl mirrors horizontal arrows", async () => {
  const handle = mountGrid({ dir: "rtl" });
  handle.root().focus();
  await nextTick();
  await press(handle, "ArrowLeft");
  assert.equal(active(handle), "Green");
  await press(handle, "ArrowRight");
  assert.equal(active(handle), "Red");
  handle.unmount();
});

test("Enter and Space select in single mode; selectionFollowsFocus selects on arrows", async () => {
  const handle = mountGrid();
  handle.root().focus();
  await nextTick();
  await press(handle, "ArrowRight");
  const enter = await press(handle, "Enter");
  assert.equal(enter.defaultPrevented, true);
  await press(handle, "ArrowRight");
  await press(handle, " ");
  assert.deepEqual(
    handle.wrapper.emitted("update:modelValue")?.map(([value]) => value),
    [swatches[1], swatches[2]],
  );
  assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], enter);
  handle.unmount();

  const follow = mountGrid({ selectionFollowsFocus: true });
  follow.root().focus();
  await nextTick();
  await press(follow, "ArrowDown");
  assert.deepEqual(follow.wrapper.emitted("update:modelValue"), [[swatches[3]]]);
  follow.unmount();
});

test("multiple mode toggles, extends with Shift+arrows, and selects all with Ctrl+A", async () => {
  const handle = mountGrid({ multiple: true });
  handle.root().focus();
  await nextTick();
  assert.equal(handle.root().getAttribute("aria-multiselectable"), "true");
  await press(handle, " ");
  await press(handle, "ArrowRight", { shiftKey: true });
  await press(handle, " ");
  await press(handle, "a", { ctrlKey: true });
  assert.deepEqual(
    handle.wrapper.emitted("update:modelValue")?.map(([value]) => value),
    [
      [swatches[0]],
      [swatches[0], swatches[1]],
      [swatches[0]],
      swatches.filter((swatch) => swatch.disabled !== true),
    ],
  );
  handle.unmount();
});

test("click selects; disabled options ignore pointer input", async () => {
  const handle = mountGrid();
  const pointerdown = new PointerEvent("pointerdown", { bubbles: true, cancelable: true });
  handle.getByRole("option", { name: "Pink" }).dispatchEvent(pointerdown);
  assert.equal(pointerdown.defaultPrevented, true);
  await handle.click(handle.getByRole("option", { name: "Pink" }));
  assert.equal(active(handle), "Pink");
  await handle.click(handle.getByRole("option", { name: "Gray" }));
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[swatches[5]]]);
  assert.equal(handle.getByRole("option", { name: "Gray" }).getAttribute("aria-disabled"), "true");
  handle.unmount();
});

test("typeahead moves the highlight by option text", async () => {
  const handle = mountGrid();
  handle.root().focus();
  await nextTick();
  await press(handle, "w");
  assert.equal(active(handle), "White");
  handle.unmount();
});

test("disabled grids leave the tab order and ignore keys", async () => {
  const handle = mountGrid({ disabled: true });
  assert.equal(handle.root().hasAttribute("tabindex"), false);
  assert.equal(handle.root().getAttribute("data-state"), "disabled");
  assert.equal((await press(handle, "ArrowDown")).defaultPrevented, false);
  handle.unmount();
});

test("controlled values wait for the parent", async () => {
  const handle = mountGrid({ modelValue: swatches[0] });
  await handle.click(handle.getByRole("option", { name: "Blue" }));
  assert.equal(handle.getByRole("option", { name: "Red" }).getAttribute("aria-selected"), "true");
  await handle.wrapper.setProps({ modelValue: { id: "blue", label: "Blue" } });
  assert.equal(handle.getByRole("option", { name: "Blue" }).getAttribute("aria-selected"), "true");
  handle.unmount();
});

test("items require a ListboxGrid provider", () => {
  assert.throws(
    () => mountInteraction(ListboxGridItem, { props: { value: "x" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});

test("pure grid navigation handles empty grids, ragged rows, and page moves", () => {
  assert.equal(moveInGrid("down", { columns: 3, count: 0, index: -1 }), null);
  assert.equal(moveInGrid("down", { columns: 3, count: 8, index: -1 }), 0);
  assert.equal(moveInGrid("up", { columns: 3, count: 8, index: -1 }), 7);
  assert.equal(moveInGrid("down", { columns: 3, count: 8, index: 5 }), 5, "no cell below");
  assert.equal(moveInGrid("page-down", { columns: 3, count: 30, index: 1, pageRows: 3 }), 10);
  assert.equal(moveInGrid("page-up", { columns: 3, count: 30, index: 10, pageRows: 5 }), 1);
  assert.equal(
    moveInGrid("right", { columns: 3, count: 8, index: 2, wrapRows: false }),
    2,
    "row wrapping can be disabled",
  );
  assert.equal(gridMoveFromKey(new KeyboardEvent("keydown", { key: "Home" })), "row-start");
  assert.equal(gridMoveFromKey(new KeyboardEvent("keydown", { key: "x" })), null);
  assert.equal(serializeGridValue({ id: "a" }, "id"), "a");
  assert.equal(serializeGridValue(3, undefined), "3");
});
