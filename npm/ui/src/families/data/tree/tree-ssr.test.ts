import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import TreeRoot from "./tree-root.vue";
import { useTreeVirtualizer } from "./tree-virtualizer.ts";
import { fileProps, renderRows } from "./tree-test-utils.ts";
import type { TreeFlatNode, TreeSlotState } from "./tree-types.ts";
import TreeItem from "./tree-item.vue";

const SsrProbe = defineComponent({
  name: "TreeSsrProbe",
  setup: () => () =>
    h(
      TreeRoot,
      { ...fileProps, checkable: true, defaultChecked: ["lib"], defaultExpanded: ["src"] },
      renderRows,
    ),
});

interface Row {
  readonly id: number;
}

const VirtualProbe = defineComponent({
  name: "TreeVirtualSsrProbe",
  setup() {
    const virtualizer = useTreeVirtualizer({
      itemSize: 20,
      overscan: 0,
      initialRect: { width: 100, height: 40 },
    });
    return () =>
      h(
        TreeRoot,
        {
          ariaLabel: "Rows",
          getKey: (node: Row) => node.id,
          items: Array.from({ length: 50 }, (_, id) => ({ id })),
          virtualizer,
        },
        ({ items }: TreeSlotState<Row, number>) =>
          items.map((item: TreeFlatNode<Row, number>) =>
            h(TreeItem, { key: item.key, item }, () => `Row ${item.key}`),
          ),
      );
  },
});

test("renders byte-identical tree markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div/);
  assert.match(html, /role="tree"/);
  assert.match(html, /id="vize-v-\d+-tree"/);
  assert.match(html, /id="vize-v-\d+-tree-item-key-src"/);
  assert.match(html, /aria-level="2"/);
  assert.match(html, /aria-expanded="true"/);
  assert.match(html, /aria-checked="mixed"/);
  assert.match(html, /tabindex="0"/);
  assert.doesNotMatch(html, /util\.ts/, "collapsed descendants are not rendered");
});

test("server-renders only the virtual window from the initial rect", async () => {
  const html = await renderToString(createSSRApp(VirtualProbe));
  assert.equal((html.match(/role="treeitem"/g) ?? []).length, 2);
  assert.match(html, /aria-setsize="50"/);
});

test("hydrates tree ids and state without mismatch warnings", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverIds = [...host.querySelectorAll("[role='treeitem']")].map((row) => row.id);

  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(
      [...host.querySelectorAll("[role='treeitem']")].map((row) => row.id),
      serverIds,
    );
    assert.equal(host.querySelector("[role='treeitem']")?.getAttribute("aria-checked"), "mixed");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
