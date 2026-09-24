import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { defineComponent, effectScope, h, nextTick, shallowRef } from "vue";
import type { PropType } from "vue";

import FloatingWindow from "./floating-window.vue";
import type { WindowLayout } from "./window-manager-model.ts";
import { useWindowLayoutPersistence } from "./window-manager-persistence.ts";
import type { FloatingWindowSlotState } from "./window-manager-types.ts";
import WindowDock from "./window-dock.vue";
import WindowManager from "./window-manager.vue";

function chrome(slot: FloatingWindowSlotState) {
  return [
    h("header", { ...slot.handleProps }, slot.active ? "active" : "idle"),
    h("button", { type: "button", "data-action": "min", onClick: slot.minimize }, "_"),
    h("button", { type: "button", "data-action": "max", onClick: slot.toggleMaximize }, "□"),
    h("button", { type: "button", "data-action": "close", onClick: slot.close }, "×"),
  ];
}

const Desktop = defineComponent({
  props: { layout: { type: Object as PropType<WindowLayout | undefined>, default: undefined } },
  emits: ["update:layout", "close"],
  setup(props, { emit }) {
    return () =>
      h(
        WindowManager,
        {
          layout: props.layout,
          snapThreshold: 0,
          "onUpdate:layout": (next: WindowLayout) => emit("update:layout", next),
        },
        () => [
          h(
            FloatingWindow,
            {
              id: "editor",
              title: "Editor",
              defaultRect: { x: 10, y: 20, width: 300, height: 200 },
              onClose: () => emit("close", "editor"),
            },
            { default: chrome },
          ),
          h(
            FloatingWindow,
            {
              id: "terminal",
              title: "Terminal",
              defaultRect: { x: 50, y: 60, width: 200, height: 150 },
            },
            { default: chrome },
          ),
          h(WindowDock, { label: "Taskbar" }),
        ],
      );
  },
});

function windowOf(wrapper: ReturnType<typeof mount>, title: string) {
  return wrapper.get(`[role="dialog"][aria-label="${title}"]`);
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

test("renders labelled non-modal windows with default geometry and stacking", () => {
  const wrapper = mount(Desktop);
  const editor = windowOf(wrapper, "Editor");
  assert.equal(editor.attributes("aria-modal"), "false");
  assert.match(
    editor.attributes("style") ?? "",
    /position: absolute; z-index: 1; left: 10px; top: 20px; width: 300px; height: 200px/,
  );
  assert.match(windowOf(wrapper, "Terminal").attributes("style") ?? "", /z-index: 2/);
  assert.equal(windowOf(wrapper, "Terminal").attributes("data-active"), "");
  assert.equal(editor.findAll('[data-part="resize-handle"]').length, 8);
  wrapper.unmount();
});

test("pointer focus raises a window and dragging the handle moves it", async () => {
  const wrapper = mount(Desktop, { attachTo: document.body });
  const editor = windowOf(wrapper, "Editor");
  const handle = editor.get('[data-part="drag-handle"]');
  handle.element.dispatchEvent(pointer("pointerdown", 100, 100));
  await nextTick();
  assert.equal(editor.attributes("data-active"), "");
  assert.match(editor.attributes("style") ?? "", /z-index: 2/);
  document.dispatchEvent(pointer("pointermove", 140, 130));
  document.dispatchEvent(pointer("pointerup", 140, 130));
  await nextTick();
  assert.match(editor.attributes("style") ?? "", /left: 50px; top: 50px/);
  document.dispatchEvent(pointer("pointermove", 400, 400));
  await nextTick();
  assert.match(editor.attributes("style") ?? "", /left: 50px; top: 50px/);
  const layouts = wrapper.emitted("update:layout") as WindowLayout[][];
  assert.deepEqual(layouts.at(-1)?.[0]?.order, ["terminal", "editor"]);
  wrapper.unmount();
});

test("resize handles and keyboard move/resize respect constraints", async () => {
  const wrapper = mount(Desktop, { attachTo: document.body });
  const terminal = windowOf(wrapper, "Terminal");
  const corner = terminal.get('[data-edge="se"]');
  corner.element.dispatchEvent(pointer("pointerdown", 250, 210));
  document.dispatchEvent(pointer("pointermove", 200, 150));
  document.dispatchEvent(pointer("pointerup", 200, 150));
  await nextTick();
  assert.match(terminal.attributes("style") ?? "", /width: 160px; height: 96px/);

  const handle = terminal.get('[data-part="drag-handle"]');
  await handle.trigger("keydown", { key: "ArrowRight" });
  await handle.trigger("keydown", { key: "ArrowDown", shiftKey: true });
  await handle.trigger("keydown", { key: "Tab" });
  assert.match(
    terminal.attributes("style") ?? "",
    /left: 66px; top: 60px; width: 160px; height: 112px/,
  );
  assert.equal(handle.attributes("tabindex"), "0");
  wrapper.unmount();
});

test("minimize, maximize, restore, close, and dock activation", async () => {
  const wrapper = mount(Desktop, { attachTo: document.body });
  const editor = windowOf(wrapper, "Editor");
  await editor.get('[data-action="max"]').trigger("click");
  assert.equal(editor.attributes("data-mode"), "maximized");
  assert.match(editor.attributes("style") ?? "", /left: 0px; top: 0px; width: 100%; height: 100%/);
  assert.equal(editor.find('[data-part="resize-handle"]').exists(), false);
  await editor.get('[data-action="max"]').trigger("click");
  assert.equal(editor.attributes("data-mode"), "normal");

  await editor.get('[data-action="min"]').trigger("click");
  assert.equal(editor.attributes("hidden"), "");
  const dockButtons = wrapper.findAll('[data-part="dock-item"]');
  assert.equal(wrapper.get('[data-vize-ui="window-dock"]').attributes("aria-label"), "Taskbar");
  assert.deepEqual(
    dockButtons.map((button) => button.text()),
    ["Editor", "Terminal"],
  );
  assert.equal(dockButtons[0]?.attributes("data-mode"), "minimized");
  assert.equal(windowOf(wrapper, "Terminal").attributes("data-active"), "");

  await dockButtons[0]?.trigger("click");
  assert.equal(editor.attributes("hidden"), undefined);
  assert.equal(editor.attributes("data-active"), "");
  assert.equal(dockButtons[0]?.attributes("aria-pressed"), "true");
  await dockButtons[0]?.trigger("click");
  assert.equal(editor.attributes("data-mode"), "minimized");

  await windowOf(wrapper, "Terminal").get('[data-action="close"]').trigger("click");
  await editor.get('[data-action="close"]').trigger("click");
  assert.deepEqual(wrapper.emitted("close"), [["editor"]]);
  wrapper.unmount();
});

test("controlled layouts drive geometry and snapping clamps to measured bounds", async () => {
  const layout: WindowLayout = {
    windows: { editor: { x: 400, y: 5, width: 300, height: 200, mode: "normal" } },
    order: ["editor", "terminal"],
  };
  const wrapper = mount(
    defineComponent(
      () => () =>
        h(WindowManager, { layout, snapThreshold: 10 }, () => [
          h(FloatingWindow, { id: "editor", title: "Editor" }, { default: chrome }),
        ]),
    ),
    { attachTo: document.body },
  );
  const editor = windowOf(wrapper, "Editor");
  assert.match(editor.attributes("style") ?? "", /left: 400px; top: 5px/);
  const manager = wrapper.get('[data-vize-ui="window-manager"]').element as HTMLElement;
  Object.defineProperty(manager, "clientWidth", { configurable: true, value: 800 });
  Object.defineProperty(manager, "clientHeight", { configurable: true, value: 600 });
  const exposed = wrapper.findComponent(WindowManager).vm as unknown as { measure: () => void };
  exposed.measure();
  const handle = editor.get('[data-part="drag-handle"]');
  handle.element.dispatchEvent(pointer("pointerdown", 0, 0));
  document.dispatchEvent(pointer("pointermove", 95, -3));
  document.dispatchEvent(pointer("pointerup", 95, -3));
  await nextTick();
  const emitted = wrapper.findComponent(WindowManager).emitted("update:layout") as WindowLayout[][];
  assert.deepEqual(emitted.at(-1)?.[0]?.windows["editor"], {
    x: 500,
    y: 0,
    width: 300,
    height: 200,
    mode: "normal",
  });
  assert.match(editor.attributes("style") ?? "", /left: 400px/);
  wrapper.unmount();
});

test("persists layouts after mount and ignores malformed storage", async () => {
  const store = new Map<string, string>([
    [
      "desk",
      JSON.stringify({
        windows: { editor: { x: 7, y: 8, width: 200, height: 100, mode: "normal" } },
        order: ["editor"],
      }),
    ],
  ]);
  const storage = {
    getItem: (key: string) => store.get(key) ?? null,
    setItem: (key: string, value: string) => void store.set(key, value),
  };
  const App = defineComponent(() => {
    const layout = useWindowLayoutPersistence({ key: "desk", storage });
    return () =>
      h(
        WindowManager,
        { layout: layout.value, "onUpdate:layout": (next: WindowLayout) => (layout.value = next) },
        () => [h(FloatingWindow, { id: "editor", title: "Editor" }, { default: chrome })],
      );
  });
  const wrapper = mount(App, { attachTo: document.body });
  await nextTick();
  const editor = windowOf(wrapper, "Editor");
  assert.match(editor.attributes("style") ?? "", /left: 7px; top: 8px/);
  await editor.get('[data-action="max"]').trigger("click");
  await nextTick();
  assert.match(store.get("desk") ?? "", /"mode":"maximized"/);
  wrapper.unmount();

  store.set("desk", "not json");
  const scope = effectScope();
  const loaded = scope.run(() =>
    useWindowLayoutPersistence({ key: "desk", storage, immediate: true }),
  );
  assert.equal(loaded?.value, undefined);
  scope.stop();
  assert.throws(
    () => useWindowLayoutPersistence({ key: "x" }),
    /VIZE_UI_WINDOW_MANAGER_PERSISTENCE_SETUP/,
  );
  const blocked = effectScope().run(() =>
    useWindowLayoutPersistence({ key: "x", storage: () => null, immediate: true }),
  );
  assert.equal(blocked?.value, undefined);
  void shallowRef;
});

test("windows and docks outside a manager throw the context diagnostic", () => {
  assert.throws(
    () => mount(FloatingWindow, { props: { id: "x", title: "X" } }),
    /VIZE_UI_CONTEXT_MISSING: WindowManager/,
  );
  assert.throws(() => mount(WindowDock), /VIZE_UI_CONTEXT_MISSING: WindowManager/);
});
