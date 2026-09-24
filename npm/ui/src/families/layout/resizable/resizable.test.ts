import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { ResizableResizeEvent, ResizableRootExpose, ResizableSize } from "./resizable.ts";
import {
  constrainResizableSize,
  resizeByDelta,
  resolveResizableEdge,
} from "./resizable-geometry.ts";
import ResizableHandle from "./resizable-handle.vue";
import ResizableRoot from "./resizable-root.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function pointer(type: string, x: number, y: number): PointerEvent {
  const event = new PointerEvent(type, {
    bubbles: true,
    button: 0,
    cancelable: true,
    clientX: x,
    clientY: y,
    isPrimary: true,
    pointerId: 3,
    pointerType: "mouse",
  });
  Object.defineProperties(event, { pageX: { value: x }, pageY: { value: y } });
  return event;
}

function key(target: Element, value: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    key: value,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

function mountResizable(props: Record<string, unknown> = {}, edges: readonly string[] = ["e"]) {
  return mountInteraction(ResizableRoot, {
    props: { id: "panel", defaultSize: { width: 300, height: 200 }, ...props },
    slots: {
      default: () => [
        h("p", "Content"),
        ...edges.map((edge) => h(ResizableHandle, { key: edge, edge })),
      ],
    },
    record: ["resize-start", "resize", "resize-end"],
  });
}

function handle(host: ReturnType<typeof mountResizable>, edge: string): HTMLElement {
  const element = host
    .root()
    .querySelector(`[data-vize-ui="resizable-handle"][data-edge="${edge}"]`);
  assert.ok(element instanceof HTMLElement);
  return element;
}

function sizeOf(host: ReturnType<typeof mountResizable>): ResizableSize {
  const root = host.root();
  return {
    height: Number.parseFloat(root.style.height),
    width: Number.parseFloat(root.style.width),
  };
}

test("renders a sized root and focusable separator handles with value semantics", () => {
  const host = mountResizable({ minWidth: 100, maxWidth: 600 }, ["e", "s", "se"]);
  const root = host.root();
  const east = handle(host, "e");
  const south = handle(host, "s");
  const corner = handle(host, "se");

  assert.equal(root.getAttribute("data-vize-ui"), "resizable-root");
  assert.equal(root.style.width, "300px");
  assert.equal(root.style.getPropertyValue("--vize-resizable-height"), "200px");
  assert.equal(root.getAttribute("data-state"), "idle");
  assert.equal(east.getAttribute("role"), "separator");
  assert.equal(east.tabIndex, 0);
  assert.equal(east.getAttribute("aria-orientation"), "vertical");
  assert.equal(east.getAttribute("aria-valuenow"), "300");
  assert.equal(east.getAttribute("aria-valuemin"), "100");
  assert.equal(east.getAttribute("aria-valuemax"), "600");
  assert.equal(east.getAttribute("aria-controls"), "panel");
  assert.equal(east.getAttribute("aria-label"), "Resize from right edge");
  assert.equal(south.getAttribute("aria-orientation"), "horizontal");
  assert.equal(south.getAttribute("aria-valuenow"), "200");
  assert.equal(south.getAttribute("aria-valuemax"), null, "infinite maxima are omitted");
  assert.equal(corner.getAttribute("aria-valuetext"), "Width 300 pixels, height 200 pixels");

  host.unmount();
});

test("arrow keys resize by step, Shift by the large step, and Home/End jump to limits", async () => {
  const host = mountResizable({ minWidth: 100, maxWidth: 500, step: 10, largeStep: 40 }, [
    "e",
    "w",
  ]);
  const east = handle(host, "e");
  const west = handle(host, "w");

  assert.equal(key(east, "ArrowRight").defaultPrevented, true);
  await nextTick();
  assert.equal(sizeOf(host).width, 310);
  key(east, "ArrowLeft", { shiftKey: true });
  await nextTick();
  assert.equal(sizeOf(host).width, 270);
  key(west, "ArrowLeft");
  await nextTick();
  assert.equal(sizeOf(host).width, 280, "the west handle grows toward the left");
  assert.equal(key(east, "ArrowDown").defaultPrevented, false, "off-axis arrows are ignored");
  key(east, "End");
  await nextTick();
  assert.equal(sizeOf(host).width, 500);
  key(east, "Home");
  await nextTick();
  assert.equal(sizeOf(host).width, 100);
  assert.equal(east.getAttribute("aria-valuenow"), "100");

  const events = host.recorded().map((entry) => entry.event);
  assert.deepEqual(events.slice(0, 3), ["resize-start", "resize", "resize-end"]);
  const first = host.recorded()[1]?.payload[0] as ResizableResizeEvent;
  assert.equal(first.source, "keyboard");
  assert.equal(first.edge, "e");
  assert.deepEqual(first.initialSize, { height: 200, width: 300 });
  assert.deepEqual(first.size, { height: 200, width: 310 });
  assert.deepEqual(host.wrapper.emitted("update:size")?.[0], [{ height: 200, width: 310 }]);

  host.unmount();
});

test("pointer drag resizes from the pressed edge with capture and clamps to constraints", async () => {
  const host = mountResizable({ minHeight: 150, maxHeight: 260 }, ["s", "nw"]);
  const south = handle(host, "s");
  const captured: number[] = [];
  south.setPointerCapture = (id: number) => {
    captured.push(id);
  };

  south.dispatchEvent(pointer("pointerdown", 10, 10));
  document.dispatchEvent(pointer("pointermove", 10, 40));
  await nextTick();
  assert.equal(host.root().getAttribute("data-state"), "resizing");
  assert.equal(south.getAttribute("data-state"), "resizing");
  assert.equal(sizeOf(host).height, 230);
  document.dispatchEvent(pointer("pointermove", 10, 200));
  await nextTick();
  assert.equal(sizeOf(host).height, 260, "height clamps at maxHeight");
  document.dispatchEvent(pointer("pointerup", 10, 200));
  await nextTick();
  assert.equal(host.root().getAttribute("data-state"), "idle");
  assert.deepEqual(captured, [3]);
  const names = host.recorded().map((entry) => entry.event);
  assert.equal(names[0], "resize-start");
  assert.equal(names.at(-1), "resize-end");
  const last = host.recorded().at(-1)?.payload[0] as ResizableResizeEvent;
  assert.equal(last.source, "pointer");

  const corner = handle(host, "nw");
  corner.dispatchEvent(pointer("pointerdown", 100, 100));
  document.dispatchEvent(pointer("pointermove", 80, 110));
  await nextTick();
  assert.deepEqual(sizeOf(host), { height: 250, width: 320 });
  document.dispatchEvent(pointer("pointerup", 80, 110));

  host.unmount();
});

test("aspect-ratio lock keeps width and height proportional", async () => {
  const host = mountResizable({ lockAspectRatio: true }, ["e", "se"]);
  key(handle(host, "e"), "ArrowRight", { shiftKey: true });
  await nextTick();
  assert.equal(sizeOf(host).width, 350);
  assert.ok(Math.abs(sizeOf(host).height - 350 / 1.5) < 0.001);
  host.unmount();

  const fixed = mountResizable({ lockAspectRatio: 2, maxWidth: 360 }, ["s"]);
  key(handle(fixed, "s"), "ArrowDown", { shiftKey: true });
  await nextTick();
  assert.deepEqual(
    sizeOf(fixed),
    { height: 180, width: 360 },
    "ratio-derived width respects maxWidth",
  );
  fixed.unmount();
});

test("logical start and end handles follow the reading direction", async () => {
  const host = mountResizable({ dir: "rtl" }, ["end", "start"]);
  const handles = host.root().querySelectorAll('[data-vize-ui="resizable-handle"]');

  assert.equal(handles[0]?.getAttribute("data-edge"), "w");
  assert.equal(handles[1]?.getAttribute("data-edge"), "e");
  assert.equal(host.root().getAttribute("dir"), "rtl");
  const end = handles[0];
  assert.ok(end instanceof HTMLElement);
  key(end, "ArrowLeft");
  await nextTick();
  assert.equal(sizeOf(host).width, 310, "in RTL the end edge grows toward the left");

  host.unmount();
});

test("disabled roots ignore keyboard and pointer resizing", async () => {
  const host = mountResizable({ disabled: true }, ["e"]);
  const east = handle(host, "e");

  assert.equal(east.getAttribute("tabindex"), null);
  assert.equal(east.getAttribute("aria-disabled"), "true");
  assert.equal(key(east, "ArrowRight").defaultPrevented, false);
  east.dispatchEvent(pointer("pointerdown", 0, 0));
  document.dispatchEvent(pointer("pointermove", 50, 0));
  document.dispatchEvent(pointer("pointerup", 50, 0));
  await nextTick();
  assert.equal(sizeOf(host).width, 300);
  assert.equal(host.recorded().length, 0);

  host.unmount();
});

test("controlled size emits requests and follows the parent", async () => {
  const host = mountResizable({ size: { width: 300, height: 200 } }, ["e"]);
  key(handle(host, "e"), "ArrowRight");
  await nextTick();
  assert.equal(sizeOf(host).width, 300);
  assert.deepEqual(host.wrapper.emitted("update:size"), [[{ height: 200, width: 310 }]]);
  await host.wrapper.setProps({ size: { width: 310, height: 200 } });
  assert.equal(sizeOf(host).width, 310);

  host.unmount();
});

test("root exposes size, state, and a clamped setSize", async () => {
  let exposed: ResizableRootExpose | null = null;
  const Probe = defineComponent({
    name: "ResizableExposeProbe",
    setup: () => () =>
      h(
        ResizableRoot,
        {
          maxWidth: 400,
          ref: (value) => {
            exposed = value as ResizableRootExpose | null;
          },
        },
        () => h(ResizableHandle),
      ),
  });
  const host = mountInteraction(Probe);
  if (exposed === null) assert.fail("ResizableRoot must expose its API");
  const root: ResizableRootExpose = exposed;

  assert.deepEqual(root.size, { height: 240, width: 320 });
  assert.equal(root.state, "idle");
  assert.equal(root.setSize({ height: 100, width: 900 }), true);
  await nextTick();
  assert.deepEqual(root.size, { height: 100, width: 400 });
  assert.ok(root.element instanceof HTMLDivElement);

  host.unmount();
});

test("geometry helpers resolve edges, deltas, and ratio-aware clamping", () => {
  const free = { aspectRatio: null, maxHeight: 500, maxWidth: 500, minHeight: 10, minWidth: 10 };
  assert.equal(resolveResizableEdge("start", "ltr"), "w");
  assert.equal(resolveResizableEdge("end", "rtl"), "w");
  assert.deepEqual(resizeByDelta({ height: 100, width: 100 }, "sw", 20, 30, free), {
    height: 130,
    width: 80,
  });
  assert.deepEqual(
    constrainResizableSize({ height: 1, width: 1000 }, { ...free, aspectRatio: 1 }),
    { height: 500, width: 500 },
  );
});

test("handles require a Resizable root", () => {
  assert.throws(() => mountInteraction(ResizableHandle), /VIZE_UI_CONTEXT_MISSING/);
});
