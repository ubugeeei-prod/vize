import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { GridList } from "./grid-list.ts";

const Probe = defineComponent({
  setup: () => () =>
    h("div", [
      h(GridList<string>, {
        ariaLabel: "Fruit",
        items: ["apple", "banana", "cherry"],
        getKey: (item: string) => item,
        selectionMode: "multiple",
        defaultSelection: ["banana"],
        reorderable: true,
      }),
    ]),
});

test("renders identical grid-list markup across SSR requests and hydrates cleanly", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /role="grid"/);
  assert.match(left, /data-key="banana"[^>]*|aria-selected="true"/);
  assert.equal(left.match(/tabindex="0"/g)?.length, 1);

  const host = document.createElement("div");
  host.innerHTML = left;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    await nextTick();
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
  }
});
