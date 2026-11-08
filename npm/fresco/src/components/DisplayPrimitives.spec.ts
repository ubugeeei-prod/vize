import assert from "node:assert/strict";
import { h, nextTick, ref } from "vue";
import { test, vi } from "vitest";

import { firstChild, mountComponent, mountFresco } from "../testing/index.js";
import Badge from "./FrescoBadge.vue";
import Link from "./FrescoLink.vue";
import Spinner from "./FrescoSpinner.vue";
import Static from "./FrescoStatic.vue";

test("Static renders append-only item containers through its typed slot", () => {
  const mounted = mountFresco(() =>
    h(
      Static,
      { items: ["done", "next"] },
      {
        default: ({ item, index }: { item: string; index: number }) =>
          h("text", { text: `${index}:${item}` }),
      },
    ),
  );
  const root = firstChild(mounted);

  assert.equal(root.props.internal_static, true);
  assert.deepEqual(root.props.style, { position: "absolute", flexDirection: "column" });
  assert.deepEqual(
    root.children
      .filter((item) => item.props.internal_static_item === true)
      .map((item) => ({
        static: item.props.internal_static_item,
        text: item.children.find((child) => typeof child.props.text === "string")?.props.text,
      })),
    [
      { static: true, text: "0:done" },
      { static: true, text: "1:next" },
    ],
  );
  mounted.unmount();
});

test("Link preserves default styling and its optional URL suffix", () => {
  const mounted = mountComponent(Link, {
    text: "Documentation",
    url: "https://example.test",
    showUrl: true,
  });
  const root = firstChild(mounted);

  assert.deepEqual(root.props.style, { flexDirection: "row" });
  assert.deepEqual(
    root.children.map((node) => ({ text: node.text, props: node.props })),
    [
      { text: "Documentation", props: { fg: "blue", underline: true } },
      { text: " (https://example.test)", props: { dim: true } },
    ],
  );
  mounted.unmount();
});

test("Badge renders both compact and bordered variants", () => {
  const compact = mountComponent(Badge, { label: "Ready", variant: "success" });
  assert.equal(firstChild(compact).text, "[Ready]");
  assert.deepEqual(firstChild(compact).props, { fg: "green", bold: true });
  compact.unmount();

  const bordered = mountComponent(Badge, { label: "Blocked", variant: "error", border: true });
  const root = firstChild(bordered);
  assert.equal(root.props.border, "single");
  assert.equal(root.props.fg, "red");
  assert.deepEqual(root.props.style, { paddingLeft: 1, paddingRight: 1 });
  assert.equal(root.children[0]?.text, "Blocked");
  bordered.unmount();
});

test("Spinner advances, follows a shorter frame set, and clears its timer", async () => {
  vi.useFakeTimers();
  const clearIntervalSpy = vi.spyOn(globalThis, "clearInterval");
  try {
    const frames = ref(["a", "b", "c"]);
    const mounted = mountFresco(() =>
      h(Spinner, {
        frames: frames.value,
        interval: 20,
        label: "Loading",
        fg: "cyan",
      }),
    );

    assert.equal(firstChild(mounted).props.text, "a Loading");
    assert.equal(firstChild(mounted).props.fg, "cyan");
    await vi.advanceTimersByTimeAsync(40);
    await nextTick();
    assert.equal(firstChild(mounted).props.text, "c Loading");
    frames.value = ["x"];
    await nextTick();
    assert.equal(firstChild(mounted).props.text, "x Loading");
    assert.equal(firstChild(mounted).props.fg, "cyan");
    mounted.unmount();
    assert.equal(clearIntervalSpy.mock.calls.length, 1);
  } finally {
    vi.clearAllTimers();
    clearIntervalSpy.mockRestore();
    vi.useRealTimers();
  }
});

test("Spinner keeps a valid frame when its frame set shrinks", async () => {
  vi.useFakeTimers();
  try {
    const frames = ref(["a", "b", "c"]);
    const mounted = mountFresco(() => h(Spinner, { frames: frames.value, interval: 20 }));

    await vi.advanceTimersByTimeAsync(40);
    await nextTick();
    assert.equal(firstChild(mounted).props.text, "c");

    frames.value = ["x"];
    await nextTick();
    assert.equal(firstChild(mounted).props.text, "x");

    frames.value = [];
    await nextTick();
    assert.equal(firstChild(mounted).props.text, "⠹");
    mounted.unmount();
  } finally {
    vi.clearAllTimers();
    vi.useRealTimers();
  }
});
