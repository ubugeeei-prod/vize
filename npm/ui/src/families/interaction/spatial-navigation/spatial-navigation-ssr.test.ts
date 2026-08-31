import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useSpatialNavigation } from "./spatial-navigation.ts";
import { rect } from "./spatial-navigation-test-utils.ts";

const SpatialSsrProbe = defineComponent({
  name: "SpatialSsrProbe",
  setup() {
    const registry = createCollectionRegistry<string, string>();
    registry.register({ key: "alpha", value: "Alpha", textValue: "Alpha", order: 0 });
    registry.register({ key: "bravo", value: "Bravo", textValue: "Bravo", order: 1 });
    const navigation = useSpatialNavigation({
      registry,
      focusBehavior: "logical",
      getRect: ({ key }) => (key === "alpha" ? rect(0, 0) : rect(120, 0)),
    });
    return () =>
      h(
        "div",
        {
          ...navigation.spatialNavigationProps,
          "data-active": registry.activeKey.value ?? "none",
          role: "grid",
          tabindex: 0,
        },
        ["alpha", "bravo"].map((key) =>
          h(
            "div",
            { key, role: "gridcell", "aria-selected": registry.activeKey.value === key },
            key,
          ),
        ),
      );
  },
});

test("renders deterministic spatial markup without geometry or handler serialization", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SpatialSsrProbe)),
    renderToString(createSSRApp(SpatialSsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<div data-active="none" role="grid" tabindex="0"><div role="gridcell" aria-selected="false">alpha</div><div role="gridcell" aria-selected="false">bravo</div></div>',
  );
  assert.doesNotMatch(outputs[0], /keydown|function|left|right/);
});

test("hydrates in place and applies virtual geometry on the first arrow", async () => {
  const serverHtml = await renderToString(createSSRApp(SpatialSsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverContainer = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SpatialSsrProbe);
  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverContainer);
    const container = host.firstElementChild as HTMLElement;
    const event = new KeyboardEvent("keydown", {
      bubbles: true,
      cancelable: true,
      key: "ArrowRight",
    });
    container.dispatchEvent(event);
    await nextTick();
    assert.equal(event.defaultPrevented, true);
    assert.equal(container.dataset.active, "bravo");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
