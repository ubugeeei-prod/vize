import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick, shallowRef } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import { canSelectViewportScroll } from "./select-scroll.ts";
import { itemAlignedReferenceRect } from "./select-positioning.ts";
import SelectContent from "./select-content.vue";
import SelectItem from "./select-item.vue";
import SelectRoot from "./select-root.vue";
import SelectScrollButton from "./select-scroll-button.vue";
import SelectTrigger from "./select-trigger.vue";
import SelectValue from "./select-value.vue";
import SelectViewport from "./select-viewport.vue";
import SelectVirtualizer from "./select-virtualizer.vue";
import { keydown, fruitSelectOptions, settle, trigger } from "./select-test-utils.ts";

function mountFruitSelect(
  rootProps: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(SelectRoot, fruitSelectOptions(rootProps, contentProps));
}

test("an outside pointer-down dismisses the popup through the dismissable layer", async () => {
  const handle = mountFruitSelect({ defaultOpen: true });
  await settle();
  const outside = document.createElement("button");
  document.body.append(outside);
  outside.dispatchEvent(
    new PointerEvent("pointerdown", { bubbles: true, cancelable: true, pointerType: "mouse" }),
  );
  await settle();
  assert.equal(trigger(handle).getAttribute("aria-expanded"), "false");
  assert.deepEqual(handle.wrapper.emitted("update:open")?.at(-1), [false]);
  outside.remove();
  handle.unmount();
});

test("pointer-down on the trigger is a layer branch and does not dismiss", async () => {
  const handle = mountFruitSelect({ defaultOpen: true });
  await settle();
  trigger(handle).dispatchEvent(
    new PointerEvent("pointerdown", { bubbles: true, cancelable: true, pointerType: "mouse" }),
  );
  await settle();
  assert.equal(trigger(handle).getAttribute("aria-expanded"), "true");
  handle.unmount();
});

test("the popup teleports to body by default and stays wired to the trigger", async () => {
  const handle = mountInteraction(SelectRoot, {
    props: { defaultOpen: true, id: "portal-select" },
    slots: {
      default: () => [
        h(SelectTrigger, { ariaLabel: "Portal" }, () => h(SelectValue)),
        h(SelectContent, null, () => h(SelectItem<string>, { value: "one" }, () => "One")),
      ],
    },
  });
  await settle();
  const listbox = document.getElementById("portal-select-listbox");
  assert.ok(listbox instanceof HTMLElement);
  assert.equal(handle.root().contains(listbox), false);
  assert.equal(listbox.getAttribute("data-position"), "popper");
  assert.equal(
    handle.getByRole("combobox", { name: "Portal" }).getAttribute("aria-controls"),
    listbox.id,
  );
  handle.unmount();
  assert.equal(document.getElementById("portal-select-listbox"), null);
});

test("item-aligned mode publishes its strategy and centres the anchor option on the trigger", async () => {
  const handle = mountFruitSelect({ defaultOpen: true }, { position: "item-aligned" });
  await settle();
  const listbox = handle.root().querySelector("[data-vize-ui='select-content']");
  assert.equal(listbox?.getAttribute("data-position"), "item-aligned");

  const trigger = { height: 40, width: 200, x: 10, y: 300 };
  assert.deepEqual(
    itemAlignedReferenceRect(
      trigger,
      { height: 200, width: 200, x: 0, y: 100 },
      {
        height: 32,
        width: 200,
        x: 0,
        y: 196,
      },
    ),
    { height: 0, width: 200, x: 10, y: 208 },
  );
  assert.deepEqual(itemAlignedReferenceRect(trigger, null, null), trigger);
  handle.unmount();
});

test("scroll buttons appear only while the viewport can scroll and scroll on hover", async () => {
  const up = shallowRef<{ update: () => void } | null>(null);
  const down = shallowRef<{ update: () => void } | null>(null);
  const viewport = shallowRef<{ element: HTMLDivElement | null } | null>(null);
  const handle = mountInteraction(SelectRoot, {
    props: { defaultOpen: true },
    slots: {
      default: () => [
        h(SelectTrigger, { ariaLabel: "Long" }, () => h(SelectValue)),
        h(SelectContent, { portalDisabled: true }, () => [
          h(SelectScrollButton, { direction: "up", ref: up }, () => "▲"),
          h(SelectViewport, { ref: viewport }, () =>
            Array.from({ length: 20 }, (_, index) =>
              h(SelectItem<number>, { key: index, value: index }, () => `Row ${index}`),
            ),
          ),
          h(SelectScrollButton, { direction: "down", ref: down }, () => "▼"),
        ]),
      ],
    },
  });
  await settle();
  const element = viewport.value?.element;
  if (!element) assert.fail("viewport must render");
  Object.defineProperty(element, "clientHeight", { configurable: true, value: 100 });
  Object.defineProperty(element, "scrollHeight", { configurable: true, value: 400 });
  up.value?.update();
  down.value?.update();
  await nextTick();

  const buttons = () => [
    ...handle.root().querySelectorAll("[data-vize-ui='select-scroll-button']:not([hidden])"),
  ];
  assert.deepEqual(
    buttons().map((button) => button.getAttribute("data-direction")),
    ["down"],
  );
  const downButton = buttons()[0];
  assert.equal(downButton?.getAttribute("aria-hidden"), "true");
  downButton?.dispatchEvent(new PointerEvent("pointerenter"));
  downButton?.dispatchEvent(new PointerEvent("pointerleave"));
  element.dispatchEvent(new Event("scroll"));
  await nextTick();
  assert.equal(element.scrollTop, 32);
  assert.deepEqual(
    buttons().map((button) => button.getAttribute("data-direction")),
    ["up", "down"],
  );
  handle.unmount();
});

test("viewport scroll predicate matches both edges", () => {
  assert.equal(
    canSelectViewportScroll({ clientHeight: 100, scrollHeight: 400, scrollTop: 0 }, "up"),
    false,
  );
  assert.equal(
    canSelectViewportScroll({ clientHeight: 100, scrollHeight: 400, scrollTop: 0 }, "down"),
    true,
  );
  assert.equal(
    canSelectViewportScroll({ clientHeight: 100, scrollHeight: 400, scrollTop: 300 }, "down"),
    false,
  );
  assert.equal(
    canSelectViewportScroll({ clientHeight: 100, scrollHeight: 400, scrollTop: 300 }, "up"),
    true,
  );
});

interface Row {
  readonly id: number;
  readonly label: string;
}

const rows: readonly Row[] = Object.freeze(
  Array.from({ length: 1000 }, (_, index) => ({
    id: index,
    label: index === 999 ? "Zebra" : `Option ${index}`,
  })),
);

function mountVirtualSelect(props: Record<string, unknown> = {}) {
  return mountInteraction(SelectRoot, {
    props: { by: "id", items: rows, itemText: (row: Row) => row.label, ...props },
    record: ["update:modelValue"],
    slots: {
      default: () => [
        h(SelectTrigger, { ariaLabel: "Rows" }, () => h(SelectValue)),
        h(SelectContent, { portalDisabled: true }, () =>
          h(SelectViewport, null, () =>
            h(
              SelectVirtualizer<Row>,
              { estimateItemSize: 20, initialViewportHeight: 100, items: rows, overscan: 2 },
              {
                default: ({ index, item }: { readonly index: number; readonly item: Row }) =>
                  h(SelectItem<Row>, { index, value: item }, () => item.label),
              },
            ),
          ),
        ),
      ],
    },
  });
}

function activeIndex(handle: ReturnType<typeof mountVirtualSelect>): string | null {
  const id = handle.getByRole("combobox", { name: "Rows" }).getAttribute("aria-activedescendant");
  if (id === null) return null;
  return handle.root().querySelector(`[id="${id}"]`)?.getAttribute("data-index") ?? null;
}

test("virtualized lists render a window and navigate across the whole collection", async () => {
  const handle = mountVirtualSelect();
  const button = handle.getByRole("combobox", { name: "Rows" });
  keydown(button, "ArrowDown");
  await settle();
  const rendered = handle.root().querySelectorAll("[role='option']").length;
  assert.ok(rendered > 0 && rendered < 20, `rendered ${rendered} of 1000 options`);
  assert.equal(activeIndex(handle), "0");
  assert.equal(
    handle.root().querySelector("[data-vize-ui='select-virtualizer']")?.getAttribute("data-count"),
    "1000",
  );

  keydown(button, "End");
  await settle();
  assert.equal(activeIndex(handle), "999");
  keydown(button, "ArrowUp");
  await settle();
  assert.equal(activeIndex(handle), "998");
  keydown(button, "Home");
  await settle();
  assert.equal(activeIndex(handle), "0");
  keydown(button, "PageDown");
  await settle();
  assert.equal(activeIndex(handle), "10");

  keydown(button, "z");
  await settle();
  assert.equal(activeIndex(handle), "999", "typeahead reaches options outside the window");
  keydown(button, "Enter");
  await settle();
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[rows[999]]]);
  assert.equal(button.textContent, "Zebra");
  handle.unmount();
});

test("virtualized lists open on a selected option outside the initial window", async () => {
  const handle = mountVirtualSelect({ defaultValue: rows[500] });
  const button = handle.getByRole("combobox", { name: "Rows" });
  assert.equal(button.textContent, "Option 500", "itemText labels the closed trigger");
  keydown(button, "Enter");
  await settle();
  assert.equal(activeIndex(handle), "500");
  handle.unmount();
});
