import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import type { SplitterGroupExpose, SplitterLayout, SplitterPanelExpose } from "./splitter.ts";
import { useSplitterPersistence } from "./splitter.ts";
import SplitterGroup from "./splitter-group.vue";
import SplitterHandle from "./splitter-handle.vue";
import SplitterPanel from "./splitter-panel.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountSplitter(
  groupProps: Record<string, unknown> = {},
  sidebarProps: Record<string, unknown> = {},
) {
  return mountInteraction(SplitterGroup, {
    props: { id: "editor", ...groupProps },
    record: ["update:layout", "resize", "resizeStart", "resizeEnd"],
    slots: {
      default: ({ state }: { readonly state: string }) => [
        h(
          SplitterPanel,
          { id: "sidebar", defaultSize: 30, minSize: 20, maxSize: 60, ...sidebarProps },
          ({ size }: { readonly size: number }) => h("span", { "data-sidebar-size": size }),
        ),
        h(SplitterHandle, { ariaLabel: "Resize sidebar" }),
        h(SplitterPanel, { id: "main", minSize: 30 }, () => `main:${state}`),
      ],
    },
  });
}

function panel(handle: { root(): HTMLElement }, id: string): HTMLElement {
  const element = handle.root().querySelector<HTMLElement>(`#${id}`);
  assert.ok(element);
  return element;
}

test("renders the APG window splitter contract with flex-carried sizes", () => {
  const handle = mountSplitter();
  const root = handle.root();
  const separator = handle.getByRole("separator", { name: "Resize sidebar" });
  const sidebar = panel(handle, "sidebar");
  const main = panel(handle, "main");

  assert.equal(root.getAttribute("data-vize-ui"), "splitter-group");
  assert.equal(root.getAttribute("data-orientation"), "horizontal");
  assert.equal(root.style.display, "flex");
  assert.equal(root.style.flexDirection, "row");
  assert.equal(sidebar.style.flexGrow, "30");
  assert.equal(main.style.flexGrow, "70");
  assert.equal(sidebar.getAttribute("data-state"), "expanded");
  assert.equal(separator.getAttribute("aria-orientation"), "vertical");
  assert.equal(separator.getAttribute("aria-valuenow"), "30");
  assert.equal(separator.getAttribute("aria-valuemin"), "20");
  assert.equal(separator.getAttribute("aria-valuemax"), "60");
  assert.equal(separator.getAttribute("aria-controls"), "sidebar");
  assert.equal(separator.getAttribute("tabindex"), "0");
  assert.equal(
    sidebar.querySelector("[data-sidebar-size]")?.getAttribute("data-sidebar-size"),
    "30",
  );
  handle.unmount();
});

test("arrow keys resize by the keyboard step within constraints; Home and End jump", async () => {
  const handle = mountSplitter();
  const separator = handle.getByRole("separator");
  separator.focus();

  const right = await handle.press(separator, "ArrowRight");
  assert.equal(right.keydownPrevented, true);
  assert.equal(separator.getAttribute("aria-valuenow"), "40");
  await handle.press(separator, "ArrowLeft");
  await handle.press(separator, "ArrowLeft");
  assert.equal(separator.getAttribute("aria-valuenow"), "20");
  await handle.press(separator, "ArrowLeft");
  assert.equal(separator.getAttribute("aria-valuenow"), "20", "minSize clamps");
  await handle.press(separator, "End");
  assert.equal(separator.getAttribute("aria-valuenow"), "60");
  await handle.press(separator, "Home");
  assert.equal(separator.getAttribute("aria-valuenow"), "20");
  const vertical = await handle.press(separator, "ArrowDown");
  assert.equal(vertical.keydownPrevented, false, "cross-axis arrows are ignored");
  assert.deepEqual(handle.wrapper.emitted("update:layout")?.[0]?.[0], [40, 60]);
  assert.deepEqual(handle.wrapper.emitted("resize")?.[0]?.[1], "keyboard");
  handle.unmount();
});

test("vertical groups use ArrowUp/ArrowDown and RTL flips horizontal arrows", async () => {
  const vertical = mountSplitter({ orientation: "vertical" });
  const separator = vertical.getByRole("separator");
  assert.equal(separator.getAttribute("aria-orientation"), "horizontal");
  assert.equal(vertical.root().style.flexDirection, "column");
  await vertical.press(separator, "ArrowDown");
  assert.equal(separator.getAttribute("aria-valuenow"), "40");
  vertical.unmount();

  const rtl = mountSplitter({ dir: "rtl" });
  const rtlSeparator = rtl.getByRole("separator");
  await rtl.press(rtlSeparator, "ArrowLeft");
  assert.equal(rtlSeparator.getAttribute("aria-valuenow"), "40");
  rtl.unmount();
});

test("collapsible panels snap closed past half their minimum and Enter toggles them", async () => {
  const handle = mountSplitter({}, { collapsible: true, collapsedSize: 5 });
  const separator = handle.getByRole("separator");
  const sidebar = panel(handle, "sidebar");
  assert.equal(separator.getAttribute("aria-valuemin"), "5");

  await handle.press(separator, "Enter");
  assert.equal(sidebar.getAttribute("data-state"), "collapsed");
  assert.equal(separator.getAttribute("aria-valuenow"), "5");
  await handle.press(separator, "Enter");
  assert.equal(sidebar.getAttribute("data-state"), "expanded");
  assert.equal(separator.getAttribute("aria-valuenow"), "30", "expand restores the prior size");

  await handle.press(separator, "Home");
  assert.equal(sidebar.getAttribute("data-state"), "collapsed", "Home reaches the collapsed size");
  await handle.press(separator, "ArrowRight");
  assert.equal(
    separator.getAttribute("aria-valuenow"),
    "20",
    "growing a collapsed panel jumps to min",
  );
  handle.unmount();
});

test("pointer drags resize from the drag origin and report start and end", async () => {
  const handle = mountSplitter();
  const root = handle.root();
  root.getBoundingClientRect = () => new DOMRect(0, 0, 1000, 400);
  const separator = handle.getByRole("separator");

  separator.dispatchEvent(
    new PointerEvent("pointerdown", { bubbles: true, cancelable: true, button: 0, clientX: 300 }),
  );
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "resizing");
  assert.equal(separator.getAttribute("data-state"), "dragging");
  window.dispatchEvent(new PointerEvent("pointermove", { clientX: 450 }));
  await nextTick();
  assert.equal(separator.getAttribute("aria-valuenow"), "45");
  window.dispatchEvent(new PointerEvent("pointermove", { clientX: 900 }));
  await nextTick();
  assert.equal(separator.getAttribute("aria-valuenow"), "60", "maxSize clamps pointer drags");
  window.dispatchEvent(new PointerEvent("pointerup", { clientX: 900 }));
  await nextTick();
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.deepEqual(handle.wrapper.emitted("resizeStart"), [[0]]);
  assert.deepEqual(handle.wrapper.emitted("resizeEnd"), [[[60, 40]]]);
  assert.equal(handle.wrapper.emitted("resize")?.at(-1)?.[1], "pointer");
  handle.unmount();
});

test("disabled groups and handles ignore keyboard and pointer input", async () => {
  const handle = mountSplitter({ disabled: true });
  const separator = handle.getByRole("separator");
  assert.equal(separator.getAttribute("tabindex"), null);
  assert.equal(separator.getAttribute("aria-disabled"), "true");
  assert.equal(handle.root().getAttribute("data-state"), "disabled");
  await handle.press(separator, "ArrowRight");
  separator.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true, button: 0 }));
  await nextTick();
  assert.equal(separator.getAttribute("aria-valuenow"), "30");
  assert.equal(handle.wrapper.emitted("update:layout"), undefined);
  handle.unmount();
});

test("controlled layouts wait for the parent and invalid layouts fall back to defaults", async () => {
  const handle = mountSplitter({ layout: [50, 50] });
  const separator = handle.getByRole("separator");
  assert.equal(separator.getAttribute("aria-valuenow"), "50");
  await handle.press(separator, "ArrowRight");
  assert.equal(separator.getAttribute("aria-valuenow"), "50");
  assert.deepEqual(handle.wrapper.emitted("update:layout")?.[0]?.[0], [60, 40]);
  await handle.wrapper.setProps({ layout: [60, 40] });
  assert.equal(separator.getAttribute("aria-valuenow"), "60");
  await handle.wrapper.setProps({ layout: [10, 20, 70] });
  assert.equal(separator.getAttribute("aria-valuenow"), "30", "mismatched layouts are ignored");
  handle.unmount();
});

test("exposes group and panel controls for programmatic layout changes", async () => {
  let group: SplitterGroupExpose | null = null;
  let sidebar: SplitterPanelExpose | null = null;
  const Probe = defineComponent({
    name: "SplitterExposeProbe",
    setup: () => () =>
      h(
        SplitterGroup,
        {
          defaultLayout: [25, 75],
          ref: (value: unknown) => {
            group = value as SplitterGroupExpose | null;
          },
        },
        () => [
          h(SplitterPanel, {
            collapsible: true,
            minSize: 10,
            ref: (value: unknown) => {
              sidebar = value as SplitterPanelExpose | null;
            },
          }),
          h(SplitterHandle),
          h(SplitterPanel),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  if (group === null || sidebar === null) assert.fail("splitter refs must expose controls");
  const groupExpose: SplitterGroupExpose = group;
  const sidebarExpose: SplitterPanelExpose = sidebar;
  assert.deepEqual(groupExpose.layout, [25, 75]);
  assert.equal(groupExpose.setLayout([40, 60]), true);
  await nextTick();
  assert.equal(sidebarExpose.size, 40);
  assert.equal(groupExpose.setLayout([100]), false, "wrong panel counts are rejected");
  assert.equal(sidebarExpose.collapse(), true);
  await nextTick();
  assert.equal(sidebarExpose.collapsed, true);
  assert.equal(sidebarExpose.expand(), true);
  await nextTick();
  assert.equal(sidebarExpose.size, 40);
  assert.equal(sidebarExpose.resize(15), true);
  await nextTick();
  assert.deepEqual(groupExpose.layout, [15, 85]);
  assert.equal(groupExpose.reset(), true);
  await nextTick();
  assert.deepEqual(groupExpose.layout, [25, 75]);
  assert.match(sidebarExpose.id, /splitter-panel$/);
  handle.unmount();
});

test("nested groups resize independently", async () => {
  const Probe = defineComponent({
    name: "SplitterNestedProbe",
    setup: () => () =>
      h(SplitterGroup, { id: "outer" }, () => [
        h(SplitterPanel, { defaultSize: 50 }, () =>
          h(SplitterGroup, { id: "inner", orientation: "vertical" }, () => [
            h(SplitterPanel, { defaultSize: 50 }),
            h(SplitterHandle, { ariaLabel: "Inner" }),
            h(SplitterPanel, { defaultSize: 50 }),
          ]),
        ),
        h(SplitterHandle, { ariaLabel: "Outer" }),
        h(SplitterPanel, { defaultSize: 50 }),
      ]),
  });
  const handle = mountInteraction(Probe);
  const inner = handle.getByRole("separator", { name: "Inner" });
  const outer = handle.getByRole("separator", { name: "Outer" });
  await handle.press(inner, "ArrowDown");
  assert.equal(inner.getAttribute("aria-valuenow"), "60");
  assert.equal(outer.getAttribute("aria-valuenow"), "50");
  assert.equal(inner.getAttribute("aria-orientation"), "horizontal");
  handle.unmount();
});

test("useSplitterPersistence loads after mount and writes later layouts", async () => {
  const store = new Map<string, string>([["layout", "[40,60]"]]);
  const storage = {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => void store.set(key, value),
  };
  const seen: (SplitterLayout | undefined)[] = [];
  const Probe = defineComponent({
    name: "SplitterPersistenceProbe",
    setup() {
      const layout = useSplitterPersistence({ key: "layout", storage });
      seen.push(layout.value);
      return () =>
        h(
          SplitterGroup,
          {
            layout: layout.value,
            "onUpdate:layout": (next: SplitterLayout) => {
              layout.value = next;
            },
          },
          () => [
            h(SplitterPanel, { defaultSize: 30 }),
            h(SplitterHandle, { ariaLabel: "Persisted" }),
            h(SplitterPanel, { defaultSize: 70 }),
          ],
        );
    },
  });
  const handle = mountInteraction(Probe);
  assert.deepEqual(seen, [undefined], "setup renders the SSR-stable defaults first");
  await nextTick();
  const separator = handle.getByRole("separator", { name: "Persisted" });
  assert.equal(separator.getAttribute("aria-valuenow"), "40");
  await handle.press(separator, "ArrowRight");
  await nextTick();
  assert.equal(store.get("layout"), "[50,50]");
  handle.unmount();

  store.set("layout", "not json");
  const fallback = mountInteraction(Probe);
  await nextTick();
  assert.equal(fallback.getByRole("separator").getAttribute("aria-valuenow"), "30");
  fallback.unmount();

  assert.throws(() => useSplitterPersistence({ key: "x" }), /VIZE_UI_SPLITTER_PERSISTENCE_SETUP/);
  void ref;
});

test("compound parts require a matching group provider", () => {
  assert.throws(() => mountInteraction(SplitterPanel), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(SplitterHandle), /VIZE_UI_CONTEXT_MISSING/);
});
