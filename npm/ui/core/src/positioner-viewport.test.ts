import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createPositioner } from "./positioner-runtime.ts";
import {
  insetViewport,
  ownerDocumentOf,
  readSafeAreaInsets,
  visualViewportRect,
} from "./positioner-viewport.ts";
import type { Rect, SafeAreaInsets, VirtualElement } from "./positioner-types.ts";

function box(x: number, y: number, width: number, height: number): Rect {
  return { height, width, x, y };
}

function virtual(rect: Rect): VirtualElement {
  return { getBoundingClientRect: () => rect };
}

function withSafeAreaProbe<Result>(insets: SafeAreaInsets, run: () => Result): Result {
  const view = document.defaultView;
  assert.ok(view);
  const original = view.getComputedStyle.bind(view);
  view.getComputedStyle = ((element: Element) => {
    if (element.getAttribute("data-vize-ui") === "safe-area-probe") {
      return {
        paddingBottom: `${String(insets.bottom)}px`,
        paddingLeft: `${String(insets.left)}px`,
        paddingRight: `${String(insets.right)}px`,
        paddingTop: `${String(insets.top)}px`,
      } as CSSStyleDeclaration;
    }
    return original(element);
  }) as typeof view.getComputedStyle;
  try {
    return run();
  } finally {
    view.getComputedStyle = original as typeof view.getComputedStyle;
  }
}

function withVisualViewport<Result>(
  viewport: Pick<Rect, "height" | "width"> & {
    readonly offsetLeft: number;
    readonly offsetTop: number;
  },
  run: () => Result,
): Result {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, "visualViewport");
  Object.defineProperty(globalThis, "visualViewport", {
    configurable: true,
    value: {
      addEventListener: () => {},
      height: viewport.height,
      offsetLeft: viewport.offsetLeft,
      offsetTop: viewport.offsetTop,
      removeEventListener: () => {},
      width: viewport.width,
    },
  });
  try {
    return run();
  } finally {
    if (descriptor) Object.defineProperty(globalThis, "visualViewport", descriptor);
    else Reflect.deleteProperty(globalThis, "visualViewport");
  }
}

test("insets the viewport by per-edge insets", () => {
  const inset = insetViewport(box(0, 0, 1_000, 800), { bottom: 24, left: 10, right: 6, top: 44 });
  assert.deepEqual(inset, { height: 732, width: 984, x: 10, y: 44 });
  const empty = insetViewport(box(0, 0, 40, 40), { bottom: 30, left: 30, right: 30, top: 30 });
  assert.deepEqual(empty, { height: 0, width: 0, x: 30, y: 30 });
});

test("clamps over-inset viewport origins to the source edges", () => {
  const horizontal = insetViewport(box(12, 4, 40, 80), { bottom: 0, left: 60, right: 0, top: 0 });
  assert.deepEqual(horizontal, { height: 80, width: 0, x: 52, y: 4 });
  const vertical = insetViewport(box(12, 4, 40, 80), { bottom: 0, left: 0, right: 0, top: 100 });
  assert.deepEqual(vertical, { height: 0, width: 40, x: 12, y: 84 });
});

test("reads owner documents structurally before falling back to the global document", () => {
  const ownerDocument = { nodeType: 9 } as Document;
  assert.equal(ownerDocumentOf({ ownerDocument }), ownerDocument);
  assert.equal(ownerDocumentOf(virtual(box(0, 0, 0, 0))), document);
});

test("reads safe-area insets through the env probe and leaves no residue", () => {
  const insets = { bottom: 24, left: 2, right: 4, top: 44 };
  const measured = withSafeAreaProbe(insets, () => readSafeAreaInsets());
  assert.deepEqual(measured, insets);
  assert.equal(document.querySelector('[data-vize-ui="safe-area-probe"]'), null);
});

test("tracks the visual viewport for pinch-zoom and keyboard insets", () => {
  const rect = withVisualViewport({ height: 400, offsetLeft: 12, offsetTop: 30, width: 640 }, () =>
    visualViewportRect(),
  );
  assert.deepEqual(rect, { height: 400, width: 640, x: 12, y: 30 });
});

test("keeps the floating box inside the keyboard-shrunk visual viewport", () => {
  withVisualViewport({ height: 400, offsetLeft: 0, offsetTop: 0, width: 1_000 }, () => {
    const controller = createPositioner({
      flip: false,
      hide: false,
      offset: 8,
      placement: "bottom",
    });
    controller.setReference(virtual(box(100, 320, 80, 40)));
    controller.setFloating(virtual(box(0, 0, 200, 100)));
    // An unshrunk 800px-tall viewport would leave the box at y=368; the
    // keyboard-shrunk visual viewport clamps it fully into the visible 400px.
    assert.equal(controller.y.value, 300);
    assert.equal(controller.availableHeight.value, 32);
    controller.dispose();
  });
});

test("applies safe-area insets to collision handling", () => {
  withSafeAreaProbe({ bottom: 24, left: 0, right: 0, top: 44 }, () => {
    const controller = createPositioner({
      flip: false,
      hide: false,
      placement: "top",
      safeArea: true,
      viewport: box(0, 0, 1_000, 800),
    });
    controller.setReference(virtual(box(100, 60, 80, 40)));
    controller.setFloating(virtual(box(0, 0, 200, 50)));
    // Without insets the box would sit at y=10, underneath a notch; the
    // safe-area-inset-top of 44 shifts it below the unsafe band.
    assert.equal(controller.y.value, 44);
    controller.dispose();
  });
});
