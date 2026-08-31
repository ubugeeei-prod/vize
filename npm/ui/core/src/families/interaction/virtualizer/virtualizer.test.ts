import assert from "node:assert/strict";

import { mount } from "@vue/test-utils";
import { test } from "vite-plus/test";
import { defineComponent, effectScope, h, nextTick, ref } from "vue";

import { createVirtualizer, useVirtualizer, type VirtualRange } from "./virtualizer.ts";

const rect = { width: 320, height: 100 };

function indexesOf(controller: { virtualItems: { value: readonly { index: number }[] } }) {
  return controller.virtualItems.value.map((item) => item.index);
}

test("renders only the visible window plus overscan", () => {
  const ranges: (VirtualRange | null)[] = [];
  const controller = createVirtualizer({
    count: 100,
    itemSize: 20,
    overscan: 2,
    initialRect: rect,
    onRangeChange: (range) => ranges.push(range),
  });

  assert.deepEqual(controller.range.value, { startIndex: 0, endIndex: 4 });
  assert.deepEqual(indexesOf(controller), [0, 1, 2, 3, 4, 5, 6]);
  assert.equal(controller.totalSize.value, 2000);
  assert.equal(controller.viewportSize.value, 100);
  assert.deepEqual(controller.virtualItems.value[1], {
    index: 1,
    key: 1,
    lane: 0,
    start: 20,
    size: 20,
    end: 40,
    isSticky: false,
    isMeasured: false,
  });
  assert.deepEqual(ranges, [{ startIndex: 0, endIndex: 4 }]);
  controller.dispose();
});

test("windows follow scroll offset updates", () => {
  const controller = createVirtualizer({
    count: 100,
    itemSize: 20,
    overscan: 1,
    initialRect: rect,
  });

  controller.scrollToOffset(400);
  assert.equal(controller.scrollOffset.value, 400);
  assert.deepEqual(controller.range.value, { startIndex: 20, endIndex: 24 });
  assert.deepEqual(indexesOf(controller), [19, 20, 21, 22, 23, 24, 25]);

  controller.scrollToOffset(1_000_000);
  assert.equal(controller.scrollOffset.value, 1900, "offsets clamp to the scrollable extent");
  controller.dispose();
});

test("virtualizes the inline axis", () => {
  const controller = createVirtualizer({
    count: 50,
    orientation: "horizontal",
    itemSize: 40,
    overscan: 0,
    initialRect: { width: 200, height: 24 },
  });

  assert.equal(controller.viewportSize.value, 200);
  assert.deepEqual(controller.range.value, { startIndex: 0, endIndex: 4 });
  controller.dispose();
});

test("reads scroll and size from an attached viewport", async () => {
  const controller = createVirtualizer({
    count: 100,
    itemSize: 20,
    overscan: 0,
    initialRect: rect,
  });
  const Consumer = defineComponent({
    name: "VirtualizerConsumerProbe",
    setup() {
      return () =>
        h(
          "div",
          { class: "viewport" },
          controller.virtualItems.value.map((item) =>
            h("div", { key: item.key, "data-index": item.index }, `Row ${item.index}`),
          ),
        );
    },
  });
  const wrapper = mount(Consumer, { attachTo: document.body });
  const viewport = wrapper.element as HTMLElement;
  controller.setViewport(viewport);

  assert.equal(wrapper.findAll("[data-index]").length, 5);
  viewport.scrollTop = 600;
  viewport.dispatchEvent(new Event("scroll"));
  await nextTick();

  assert.equal(controller.scrollOffset.value, 600);
  assert.deepEqual(controller.range.value, { startIndex: 30, endIndex: 34 });
  assert.equal(wrapper.find("[data-index]").attributes("data-index"), "30");

  controller.setViewport(null);
  viewport.scrollTop = 0;
  viewport.dispatchEvent(new Event("scroll"));
  assert.equal(controller.scrollOffset.value, 600, "detached viewports stop reporting");
  controller.dispose();
  wrapper.unmount();
});

test("keeps the active sticky item mounted while scrolled past", () => {
  const controller = createVirtualizer({
    count: 100,
    itemSize: 20,
    overscan: 0,
    initialRect: rect,
    stickyIndexes: [0, 10, 50],
  });

  controller.scrollToOffset(600);
  assert.equal(controller.activeStickyIndex.value, 10);
  assert.deepEqual(indexesOf(controller), [10, 30, 31, 32, 33, 34]);
  const pinned = controller.virtualItems.value[0];
  assert.equal(pinned?.isSticky, true);
  assert.equal(pinned?.start, 200, "the pinned item keeps its own offsets");

  controller.scrollToOffset(0);
  assert.equal(controller.activeStickyIndex.value, 0);
  controller.dispose();
});

test("anchors the viewport when items above it change size", () => {
  const controller = createVirtualizer({
    count: 100,
    estimateItemSize: 20,
    overscan: 0,
    initialRect: rect,
  });

  controller.scrollToOffset(400);
  assert.equal(controller.range.value?.startIndex, 20);

  controller.resizeItem(5, 60);
  assert.equal(controller.scrollOffset.value, 440, "the offset absorbs the delta");
  assert.equal(controller.range.value?.startIndex, 20, "the visible content is unchanged");

  controller.resizeItem(21, 80);
  assert.equal(controller.scrollOffset.value, 440, "items inside the window never adjust");
  controller.dispose();
});

test("opts out of scroll anchoring", () => {
  const controller = createVirtualizer({
    count: 100,
    estimateItemSize: 20,
    anchorScroll: false,
    initialRect: rect,
  });
  controller.scrollToOffset(400);
  controller.resizeItem(0, 120);
  assert.equal(controller.scrollOffset.value, 400);
  assert.equal(controller.range.value?.startIndex, 15, "content shifts under the viewport");
  controller.dispose();
});

test("restores a snapshot through its anchored item", () => {
  const controller = createVirtualizer({
    count: 100,
    estimateItemSize: 20,
    initialRect: rect,
  });
  controller.scrollToOffset(410);
  const snapshot = controller.createScrollSnapshot();
  assert.deepEqual(snapshot, { offset: 410, anchorIndex: 20, anchorGap: 10 });

  controller.resizeItem(0, 220);
  controller.restoreScroll(snapshot);
  assert.equal(controller.scrollOffset.value, 20 * 20 + 200 + 10);
  assert.equal(controller.range.value?.startIndex, 20);

  controller.restoreScroll({ offset: 40, anchorIndex: null, anchorGap: 0 });
  assert.equal(controller.scrollOffset.value, 40, "offset restoration is the fallback");
  controller.dispose();
});

test("invalidates measurements from an index", () => {
  const controller = createVirtualizer({
    count: 100,
    estimateItemSize: 20,
    anchorScroll: false,
    initialRect: rect,
  });
  controller.resizeItem(0, 100);
  controller.resizeItem(50, 100);
  assert.equal(controller.totalSize.value, 2160);

  controller.invalidateMeasurements(10);
  assert.equal(controller.totalSize.value, 2080);
  controller.invalidateMeasurements();
  assert.equal(controller.totalSize.value, 2000);
  assert.equal(controller.virtualItems.value[0]?.isMeasured, false);
  controller.dispose();
});

test("measures rendered elements and recovers disconnected nodes", () => {
  type Callback = (entries: readonly unknown[]) => void;
  const instances: { callback: Callback; unobserved: Element[] }[] = [];
  const previous = globalThis.ResizeObserver;
  globalThis.ResizeObserver = class {
    unobserved: Element[] = [];
    constructor(readonly callback: Callback) {
      instances.push(this);
    }
    observe(_target: Element, _options?: unknown) {}
    unobserve(target: Element) {
      this.unobserved.push(target);
    }
    disconnect() {}
  } as unknown as typeof ResizeObserver;

  try {
    const controller = createVirtualizer({
      count: 100,
      estimateItemSize: 20,
      anchorScroll: false,
      initialRect: rect,
    });
    const node = document.createElement("div");
    document.body.append(node);
    controller.measureElement(node, 0);
    const platform = instances.at(-1);
    assert.ok(platform);

    platform.callback([
      { target: node, borderBoxSize: [{ inlineSize: 320, blockSize: 48 }], contentRect: rect },
    ]);
    assert.equal(controller.virtualItems.value[0]?.size, 48);
    assert.equal(controller.virtualItems.value[0]?.isMeasured, true);

    node.remove();
    platform.callback([
      { target: node, borderBoxSize: [{ inlineSize: 0, blockSize: 0 }], contentRect: rect },
    ]);
    assert.equal(controller.virtualItems.value[0]?.size, 48, "the measurement survives removal");
    assert.deepEqual(platform.unobserved, [node], "the disconnected node is released");

    const replacement = document.createElement("div");
    document.body.append(replacement);
    controller.measureElement(replacement, 0);
    platform.callback([
      {
        target: replacement,
        borderBoxSize: [{ inlineSize: 320, blockSize: 64 }],
        contentRect: rect,
      },
    ]);
    assert.equal(controller.virtualItems.value[0]?.size, 64, "remeasurement resumes seamlessly");

    controller.measureElement(null, 0);
    assert.deepEqual(platform.unobserved.at(-1), replacement);
    replacement.remove();
    controller.dispose();
  } finally {
    globalThis.ResizeObserver = previous;
  }
});

test("prepending shifts measurements and keeps the view anchored", () => {
  const count = ref(100);
  const controller = createVirtualizer({
    count,
    estimateItemSize: 20,
    initialRect: rect,
  });
  controller.resizeItem(2, 40);
  controller.scrollToOffset(220);
  assert.equal(controller.range.value?.startIndex, 10);

  count.value = 105;
  controller.notifyPrepended(5);
  assert.equal(controller.scrollOffset.value, 320, "the offset absorbs the prepended extent");
  assert.deepEqual(controller.range.value, { startIndex: 15, endIndex: 19 });

  controller.scrollToIndex(7, "start");
  assert.equal(controller.scrollOffset.value, 140);
  const shifted = controller.virtualItems.value.find((item) => item.index === 7);
  assert.equal(shifted?.size, 40, "measurements follow their shifted indexes");
  assert.equal(shifted?.isMeasured, true);
  controller.dispose();
});

test("scrolls an index into each alignment", () => {
  const controller = createVirtualizer({ count: 100, itemSize: 20, initialRect: rect });

  controller.scrollToIndex(50, "start");
  assert.equal(controller.scrollOffset.value, 1000);
  controller.scrollToIndex(50, "end");
  assert.equal(controller.scrollOffset.value, 920);
  controller.scrollToIndex(50, "center");
  assert.equal(controller.scrollOffset.value, 960);
  controller.scrollToIndex(51, "auto");
  assert.equal(controller.scrollOffset.value, 960, "visible items do not scroll on auto");
  controller.scrollToIndex(0, "auto");
  assert.equal(controller.scrollOffset.value, 0);
  controller.scrollToIndex(99, "auto");
  assert.equal(controller.scrollOffset.value, 1900);
  controller.dispose();
});

test("validates options and rejects misuse", () => {
  assert.throws(() => createVirtualizer({ count: 10 }), /one of itemSize or estimateItemSize/);
  assert.throws(() => createVirtualizer({ count: 10, itemSize: -1 }), /VIZE_UI_VIRTUALIZER_OPTION/);
  assert.throws(
    () => createVirtualizer({ count: 2.5, itemSize: 10 }),
    /count must resolve to an integer/,
  );
  assert.throws(
    () =>
      createVirtualizer({
        count: 10,
        itemSize: 10,
        orientation: "diagonal" as "vertical",
      }),
    /orientation must resolve to horizontal or vertical/,
  );

  const controller = createVirtualizer({ count: 10, itemSize: 10 });
  assert.throws(() => controller.scrollToIndex(99), /outside the collection/);
  assert.throws(() => controller.scrollToIndex(1, "middle" as "center"), /unknown alignment/);
  assert.throws(() => controller.notifyPrepended(0), /positive integer/);
  assert.throws(() => controller.invalidateMeasurements(-1), /non-negative integer/);
  controller.dispose();
  controller.dispose();
  assert.throws(() => controller.scrollToOffset(0), /VIZE_UI_VIRTUALIZER_DISPOSED/);

  assert.throws(() => useVirtualizer({ count: 10, itemSize: 10 }), /VIZE_UI_VIRTUALIZER_SETUP/);
  const scope = effectScope();
  const scoped = scope.run(() => useVirtualizer({ count: 10, itemSize: 10 }));
  assert.ok(scoped);
  scope.stop();
  assert.throws(() => scoped.scrollToOffset(0), /VIZE_UI_VIRTUALIZER_DISPOSED/);
});
