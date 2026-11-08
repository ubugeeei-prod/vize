import assert from "node:assert/strict";
import { h, nextTick, ref } from "vue";
import { test } from "vitest";
import { treeToScreenReaderString } from "../accessibility.js";
import { createVirtualViewport } from "../composables/useVirtualViewport.js";
import { updateLastRenderLayouts } from "../layoutMetrics.js";
import { findNodes, firstChild, mountComponent, renderTui } from "../testing/index.js";
import ScrollBar from "./ScrollBar.vue";
import ScrollBox from "./ScrollBox.vue";

test("scrollbar maps a bounded offset onto its track", () => {
  const mounted = mountComponent(ScrollBar, {
    itemCount: 100,
    viewportSize: 10,
    offset: 45,
    length: 5,
    orientation: "horizontal",
  });
  const root = firstChild(mounted);

  assert.equal(root.props["aria-role"], "scrollbar");
  assert.deepEqual(root.props["aria-state"], {
    orientation: "horizontal",
    valueMin: 0,
    valueMax: 90,
    valueNow: 45,
  });
  const glyphNodes = root.children.filter((node) => typeof node.props.text === "string");
  assert.deepEqual(
    glyphNodes.map((node) => node.props.text),
    ["│", "│", "█", "│", "│"],
  );
  assert.ok(glyphNodes.every((node) => node.props["aria-hidden"] === true));
  assert.doesNotMatch(treeToScreenReaderString(root), /[│█]/u);
  mounted.unmount();
});

test("scrollbar normalizes invalid offsets and fractional bounds", () => {
  for (const offset of [-5, Number.NaN]) {
    const mounted = mountComponent(ScrollBar, {
      itemCount: 10.9,
      viewportSize: 2.9,
      offset,
      length: 4,
    });
    const root = firstChild(mounted);

    assert.deepEqual(root.props["aria-state"], {
      orientation: "vertical",
      valueMin: 0,
      valueMax: 8,
      valueNow: 0,
    });
    assert.deepEqual(
      root.children.flatMap((node) =>
        typeof node.props.text === "string" ? [node.props.text] : [],
      ),
      ["█", "│", "│", "│"],
    );
    mounted.unmount();
  }
});

test("scrollbar drags its thumb through the latest native layout", async () => {
  const offset = ref(20);
  const updates: number[] = [];
  const rendered = renderTui(() =>
    h(ScrollBar, {
      itemCount: 100,
      viewportSize: 10,
      offset: offset.value,
      length: 10,
      "onUpdate:offset": (value: number) => {
        updates.push(value);
        offset.value = value;
      },
    }),
  );
  const scrollbar = firstChild(rendered);
  updateLastRenderLayouts([{ id: scrollbar.id, x: 5, y: 2, width: 1, height: 10 }]);

  try {
    await rendered.input.mouse({ kind: "down", button: "left", x: 5, y: 4 });
    await rendered.input.mouse({ kind: "drag", button: "left", x: 5, y: 10 });
    await rendered.input.mouse({ kind: "up", button: "left", x: 5, y: 10 });

    assert.equal(offset.value, 80);
    assert.deepEqual(updates, [80]);

    await rendered.input.mouse({ kind: "drag", button: "left", x: 5, y: 2 });
    assert.equal(offset.value, 80, "releasing the thumb ends pointer capture");
  } finally {
    updateLastRenderLayouts([]);
    rendered.unmount();
  }
});

test("scroll box accepts track clicks but ignores disabled pointer input", async () => {
  const offset = ref(0);
  const disabled = ref(false);
  const rendered = renderTui(() =>
    h(
      ScrollBox,
      {
        itemCount: 100,
        viewportSize: 10,
        modelValue: offset.value,
        disabled: disabled.value,
        "onUpdate:modelValue": (value: number) => {
          offset.value = value;
        },
      },
      {
        default: ({ index }: { index: number }) => h("text", { text: `row-${index}` }),
      },
    ),
  );
  const scrollbar = findNodes(rendered.root, (node) => node.props["aria-role"] === "scrollbar")[0];
  assert.ok(scrollbar);
  updateLastRenderLayouts([{ id: scrollbar.id, x: 9, y: 3, width: 1, height: 10 }]);

  try {
    await rendered.input.mouse({ kind: "down", button: "left", x: 9, y: 12 });
    assert.equal(offset.value, 90, "a track click moves the thumb without requiring focus");

    disabled.value = true;
    await nextTick();
    await rendered.input.mouse({ kind: "down", button: "left", x: 9, y: 3 });
    assert.equal(offset.value, 90);
  } finally {
    updateLastRenderLayouts([]);
    rendered.unmount();
  }
});

test("100k items retain only viewport plus overscan and scroll by keyboard and wheel", async () => {
  const offset = ref(0);
  let slotCalls = 0;
  const rendered = renderTui(() =>
    h(
      ScrollBox,
      {
        itemCount: 100_000,
        viewportSize: 5,
        overscan: 2,
        focused: true,
        modelValue: offset.value,
        "onUpdate:modelValue": (value: number) => {
          offset.value = value;
        },
      },
      {
        default: ({ index }: { index: number }) => {
          slotCalls += 1;
          return h("text", { text: `item-${index}` });
        },
      },
    ),
  );

  assert.equal(slotCalls, 7, "initial render keeps five visible and two trailing rows");
  const retained = () =>
    findNodes(rendered.root, (node) => typeof node.props["data-scroll-index"] === "number");
  const visible = () => retained().filter((node) => node.props["aria-hidden"] === false);
  assert.equal(retained().length, 7);
  assert.deepEqual(
    visible().map((node) => node.props["data-scroll-index"]),
    [0, 1, 2, 3, 4],
  );

  await rendered.input.key({ key: "pagedown" });
  assert.equal(offset.value, 4);
  assert.ok(retained().length <= 9, "retention is bounded by viewport plus two-sided overscan");
  assert.deepEqual(
    visible().map((node) => node.props["data-scroll-index"]),
    [4, 5, 6, 7, 8],
  );

  await rendered.input.mouse({ kind: "scroll-down", x: 0, y: 0 });
  assert.equal(offset.value, 7);
  await rendered.input.key({ key: "end" });
  assert.equal(offset.value, 99_995);
  assert.deepEqual(
    visible().map((node) => node.props["data-scroll-index"]),
    [99_995, 99_996, 99_997, 99_998, 99_999],
  );

  rendered.unmount();
});

test("normalizes an unsafe wheel step before scrolling", async () => {
  const offset = ref(0);
  const rendered = renderTui(() =>
    h(
      ScrollBox,
      {
        itemCount: 10,
        viewportSize: 2,
        focused: true,
        wheelStep: Number.POSITIVE_INFINITY,
        modelValue: offset.value,
        "onUpdate:modelValue": (value: number) => {
          offset.value = value;
        },
      },
      {
        default: ({ index }: { index: number }) => h("text", { text: `row-${index}` }),
      },
    ),
  );

  await rendered.input.mouse({ kind: "scroll-down", x: 0, y: 0 });
  assert.equal(offset.value, 1);
  rendered.unmount();
});

test("simultaneous collection and model updates clamp against the new item count", async () => {
  const itemCount = ref(5);
  const offset = ref(0);
  const updates: number[] = [];
  const rendered = renderTui(() =>
    h(
      ScrollBox,
      {
        itemCount: itemCount.value,
        viewportSize: 2,
        modelValue: offset.value,
        showScrollbar: false,
        "onUpdate:modelValue": (value: number) => {
          updates.push(value);
          offset.value = value;
        },
      },
      {
        default: ({ index }: { index: number }) => h("text", { text: `row-${index}` }),
      },
    ),
  );
  const visibleIndexes = () =>
    findNodes(
      rendered.root,
      (node) =>
        node.props["aria-hidden"] === false && typeof node.props["data-scroll-index"] === "number",
    ).map((node) => node.props["data-scroll-index"]);

  itemCount.value = 100;
  offset.value = 80;
  await nextTick();
  assert.deepEqual(visibleIndexes(), [80, 81]);
  assert.deepEqual(updates, [], "a valid controlled offset is not echoed back");

  itemCount.value = 10;
  await nextTick();
  assert.equal(offset.value, 8);
  assert.deepEqual(updates, [8], "a now-out-of-range model is synchronized to its clamp");
  assert.deepEqual(visibleIndexes(), [8, 9]);
  rendered.unmount();
});

test("sticky slots stay mounted while followOutput tracks a growing tail", async () => {
  const itemCount = ref(3);
  const offset = ref(0);
  const rendered = renderTui(() =>
    h(
      ScrollBox,
      {
        itemCount: itemCount.value,
        viewportSize: 2,
        modelValue: offset.value,
        followOutput: true,
        showScrollbar: false,
        "onUpdate:modelValue": (value: number) => {
          offset.value = value;
        },
      },
      {
        start: () => h("text", { text: "header" }),
        default: ({ index }: { index: number }) => h("text", { text: `row-${index}` }),
        end: () => h("text", { text: "footer" }),
      },
    ),
  );

  assert.equal(offset.value, 0);
  itemCount.value = 5;
  await nextTick();
  assert.equal(offset.value, 0, "followOutput does not jump when the viewport was not at the tail");

  offset.value = 3;
  await nextTick();
  itemCount.value = 7;
  await nextTick();
  assert.equal(offset.value, 5, "a viewport already at the tail follows appended items");
  assert.equal(findNodes(rendered.root, (node) => node.props.text === "header").length, 1);
  assert.equal(findNodes(rendered.root, (node) => node.props.text === "footer").length, 1);
  rendered.unmount();
});

test("virtual viewport clamps resize, aligns targets, and rejects invalid counts", () => {
  const viewport = createVirtualViewport({
    itemCount: 100,
    viewportSize: 10,
    overscan: 3,
    initialOffset: 90,
  });
  assert.equal(viewport.atEnd.value, true);
  assert.deepEqual(viewport.range.value, {
    visibleStart: 90,
    visibleEnd: 100,
    overscanStart: 87,
    overscanEnd: 100,
  });

  assert.equal(viewport.scrollIntoView(50, "center"), true);
  assert.equal(viewport.offset.value, 45);
  assert.equal(viewport.scrollIntoView(47), false, "an already visible row does not move");
  viewport.setViewportSize(60);
  assert.equal(viewport.offset.value, 40, "growing the viewport clamps its old offset");
  viewport.setItemCount(12);
  assert.equal(viewport.offset.value, 0);
  assert.deepEqual(viewport.range.value, {
    visibleStart: 0,
    visibleEnd: 12,
    overscanStart: 0,
    overscanEnd: 12,
  });

  assert.throws(
    () => createVirtualViewport({ itemCount: -1, viewportSize: 1 }),
    /itemCount must be a non-negative safe integer/u,
  );
  assert.throws(() => viewport.scrollBy(0.5), /delta must be a safe integer/u);
  assert.equal(viewport.scrollIntoView(12), false, "an index outside the collection is refused");
});
