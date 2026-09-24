import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, h, nextTick } from "vue";

import SafeArea from "./safe-area.vue";
import { useSafeAreaInsets } from "./safe-area-runtime.ts";
import type { SafeAreaEdgeInsets } from "./safe-area-runtime.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function stubInsets(values: Partial<Record<string, string>>): () => void {
  const original = window.getComputedStyle.bind(window);
  window.getComputedStyle = (element: Element) => {
    const style = original(element);
    if (element.getAttribute("data-vize-ui") !== "safe-area-probe") return style;
    return new Proxy(style, {
      get: (target, key) =>
        typeof key === "string" && key in values ? values[key] : Reflect.get(target, key),
    });
  };
  return () => {
    window.getComputedStyle = original;
  };
}

test("exposes env() insets as CSS variables and applies only the selected edges", () => {
  const handle = mountInteraction(SafeArea, {
    props: { edges: ["top", "bottom"], apply: "padding" },
  });
  const root = handle.root();

  assert.equal(root.getAttribute("data-vize-ui"), "safe-area");
  assert.equal(root.getAttribute("data-edges"), "top bottom");
  assert.equal(
    root.style.getPropertyValue("--vize-safe-area-inset-top"),
    "env(safe-area-inset-top, 0px)",
  );
  assert.equal(
    root.style.getPropertyValue("--vize-safe-area-inset-left"),
    "env(safe-area-inset-left, 0px)",
  );
  assert.equal(root.style.getPropertyValue("padding-top"), "var(--vize-safe-area-inset-top)");
  assert.equal(root.style.getPropertyValue("padding-bottom"), "var(--vize-safe-area-inset-bottom)");
  assert.equal(root.style.getPropertyValue("padding-left"), "");
  handle.unmount();

  const plain = mountInteraction(SafeArea, { props: { as: "footer" } });
  assert.equal(plain.root().tagName, "FOOTER");
  assert.equal(plain.root().style.getPropertyValue("padding-top"), "");
  plain.unmount();
});

test("measures insets after a microtask, publishes non-zero edges, and re-measures on resize", async () => {
  const restore = stubInsets({ paddingTop: "47px", paddingBottom: "34px" });
  try {
    const handle = mountInteraction(SafeArea, {
      slots: {
        default: ({ insets }: { insets: SafeAreaEdgeInsets }) => `${insets.top}/${insets.bottom}`,
      },
    });
    assert.equal(handle.root().textContent, "0/0", "first render matches the server");
    await Promise.resolve();
    await nextTick();
    assert.equal(handle.root().textContent, "47/34");
    assert.equal(handle.root().getAttribute("data-insets"), "top bottom");
    handle.unmount();
    assert.equal(
      document.querySelector('[data-vize-ui="safe-area-probe"]'),
      null,
      "the probe is removed",
    );
  } finally {
    restore();
  }
});

test("useSafeAreaInsets works in any scope and is inert without a document", async () => {
  let restore = stubInsets({ paddingLeft: "10px" });
  const scope = effectScope();
  const controller = scope.run(() => useSafeAreaInsets());
  assert.ok(controller);
  await Promise.resolve();
  assert.equal(controller.insets.value.left, 10);
  restore();
  restore = stubInsets({ paddingLeft: "20px" });
  window.dispatchEvent(new Event("resize"));
  assert.equal(controller.insets.value.left, 20);
  scope.stop();
  window.dispatchEvent(new Event("resize"));
  restore();

  const inert = useSafeAreaInsets({ document: null });
  inert.measure();
  assert.deepEqual(inert.insets.value, { top: 0, right: 0, bottom: 0, left: 0 });
  inert.stop();
});

test("renders slot content with typed insets", () => {
  const handle = mountInteraction(SafeArea, { slots: { default: () => h("span", "content") } });
  assert.equal(handle.root().textContent, "content");
  handle.unmount();
});
