import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";
import type { PropType } from "vue";

import DashboardGridItem from "./dashboard-grid-item.vue";
import type { DashboardLayout } from "./dashboard-grid-layout.ts";
import type { DashboardGridItemSlotState } from "./dashboard-grid-types.ts";
import DashboardGrid from "./dashboard-grid.vue";

const initial: DashboardLayout = [
  { id: "sales", x: 0, y: 0, w: 2, h: 2 },
  { id: "users", x: 2, y: 0, w: 2, h: 1 },
  { id: "pinned", x: 0, y: 4, w: 4, h: 1, static: true },
];

function widget(slot: DashboardGridItemSlotState) {
  return [
    h(
      "h3",
      { ...slot.handleProps },
      `${slot.item.id}:${slot.item.x},${slot.item.y},${slot.item.w}x${slot.item.h}:${String(slot.dragging)}`,
    ),
    h("span", { ...slot.resizeHandleProps }),
  ];
}

const Board = defineComponent({
  props: {
    layout: { type: Array as PropType<DashboardLayout | undefined>, default: undefined },
    disabled: { type: Boolean, default: false },
  },
  emits: ["update:layout"],
  setup(props, { emit }) {
    return () =>
      h(
        DashboardGrid,
        {
          layout: props.layout,
          defaultLayout: initial,
          columns: 4,
          rowHeight: 50,
          gap: 10,
          disabled: props.disabled,
          label: "Metrics",
          "onUpdate:layout": (next: DashboardLayout) => emit("update:layout", next),
        },
        () => [
          h(DashboardGridItem, { id: "sales", label: "Sales" }, { default: widget }),
          h(DashboardGridItem, { id: "users", label: "Users" }, { default: widget }),
          h(DashboardGridItem, { id: "pinned", label: "Pinned" }, { default: widget }),
          h(
            DashboardGridItem,
            { id: "new", label: "New", defaultPosition: { x: 2, y: 1, w: 1, h: 1 } },
            { default: widget },
          ),
        ],
      );
  },
});

function heading(wrapper: ReturnType<typeof mount>, label: string): string {
  return wrapper.get(`[aria-label="${label}"] h3`).text();
}

function pointer(type: string, x: number, y: number): PointerEvent {
  return new PointerEvent(type, {
    bubbles: true,
    cancelable: true,
    clientX: x,
    clientY: y,
    button: 0,
  });
}

test("renders a labelled CSS grid with widgets on native grid lines", () => {
  const wrapper = mount(Board);
  const grid = wrapper.get('[data-vize-ui="dashboard-grid"]');
  assert.equal(grid.attributes("role"), "region");
  assert.equal(grid.attributes("aria-label"), "Metrics");
  assert.match(
    grid.attributes("style") ?? "",
    /grid-template-columns: repeat\(4, minmax\(0, 1fr\)\); grid-auto-rows: 50px; gap: 10px/,
  );
  const sales = wrapper.get('[aria-label="Sales"]');
  assert.equal(sales.attributes("aria-roledescription"), "dashboard widget");
  assert.match(sales.attributes("style") ?? "", /grid-column: 1 \/ span 2; grid-row: 1 \/ span 2/);
  assert.equal(wrapper.get('[aria-label="Pinned"]').attributes("data-static"), "");
  assert.equal(heading(wrapper, "New"), "new:2,1,1x1:false");
  wrapper.unmount();
});

test("keyboard moves and resizes widgets and pushes collisions", async () => {
  const wrapper = mount(Board);
  const handle = wrapper.get('[aria-label="Users"] [data-part="drag-handle"]');
  await handle.trigger("keydown", { key: "ArrowDown", shiftKey: true });
  assert.equal(heading(wrapper, "Users"), "users:2,0,2x2:false");
  await handle.trigger("keydown", { key: "ArrowLeft" });
  // The moved widget keeps its target cell; the widget it lands on is pushed below it.
  assert.equal(heading(wrapper, "Users"), "users:1,0,2x2:false");
  assert.equal(heading(wrapper, "Sales"), "sales:0,2,2x2:false");
  assert.equal(heading(wrapper, "Pinned"), "pinned:0,4,4x1:false");
  const layouts = wrapper.emitted("update:layout") as DashboardLayout[][];
  assert.equal(layouts.length, 2);

  const pinned = wrapper.get('[aria-label="Pinned"] [data-part="drag-handle"]');
  await pinned.trigger("keydown", { key: "ArrowUp" });
  assert.equal(heading(wrapper, "Pinned"), "pinned:0,4,4x1:false");
  await handle.trigger("keydown", { key: "Enter" });
  assert.equal(wrapper.emitted("update:layout")?.length, 2);
  wrapper.unmount();
});

test("pointer drags snap to cells using the measured cell size", async () => {
  const wrapper = mount(Board, { attachTo: document.body });
  const grid = wrapper.get('[data-vize-ui="dashboard-grid"]').element as HTMLElement;
  Object.defineProperty(grid, "clientWidth", { configurable: true, value: 390 });
  const handle = wrapper.get('[aria-label="Sales"] [data-part="drag-handle"]').element;
  handle.dispatchEvent(pointer("pointerdown", 0, 0));
  await nextTick();
  assert.equal(heading(wrapper, "Sales"), "sales:0,0,2x2:true");
  assert.equal(wrapper.get('[aria-label="Sales"]').attributes("data-dragging"), "");
  document.dispatchEvent(pointer("pointermove", 210, 5));
  document.dispatchEvent(pointer("pointerup", 210, 5));
  await nextTick();
  assert.equal(heading(wrapper, "Sales"), "sales:2,0,2x2:false");

  const grip = wrapper.get('[aria-label="Sales"] [data-part="resize-handle"]').element;
  grip.dispatchEvent(pointer("pointerdown", 0, 0));
  document.dispatchEvent(pointer("pointermove", -100, 60));
  document.dispatchEvent(pointer("pointerup", -100, 60));
  await nextTick();
  assert.equal(heading(wrapper, "Sales"), "sales:2,0,1x3:false");
  wrapper.unmount();
});

test("disabled grids ignore input and controlled layouts only change via props", async () => {
  const disabled = mount(Board, { props: { disabled: true } });
  await disabled
    .get('[aria-label="Users"] [data-part="drag-handle"]')
    .trigger("keydown", { key: "ArrowDown" });
  assert.equal(heading(disabled, "Users"), "users:2,0,2x1:false");
  assert.equal(disabled.get('[data-vize-ui="dashboard-grid"]').attributes("data-disabled"), "");
  disabled.unmount();

  const controlled = mount(Board, { props: { layout: [{ id: "sales", x: 1, y: 0, w: 1, h: 1 }] } });
  await controlled
    .get('[aria-label="Sales"] [data-part="drag-handle"]')
    .trigger("keydown", { key: "ArrowRight" });
  assert.equal(heading(controlled, "Sales"), "sales:1,0,1x1:false");
  const emitted = controlled.emitted("update:layout") as DashboardLayout[][];
  assert.deepEqual(
    emitted[0]?.[0]?.find((item) => item.id === "sales"),
    { id: "sales", x: 2, y: 0, w: 1, h: 1 },
  );
  controlled.unmount();
});

test("widgets outside a grid throw the context diagnostic", () => {
  assert.throws(
    () => mount(DashboardGridItem, { props: { id: "x" } }),
    /VIZE_UI_CONTEXT_MISSING: DashboardGrid/,
  );
});
