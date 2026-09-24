import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, nextTick } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

import { places, renderCascaderTree } from "./cascader-fixture-tree.ts";

const france = places[2];
const paris = france?.children?.[0];

function probe(open: boolean): Component {
  return defineComponent({
    name: "CascaderSsrProbe",
    setup: () => () =>
      renderCascaderTree({
        defaultOpen: open,
        defaultValue: [france, paris],
        id: "ssr-place",
        name: "place",
      }),
  });
}

async function assertHydrates(component: Component): Promise<void> {
  const serverHtml = await renderToString(createSSRApp(component));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(component);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    await nextTick();
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
}

test("renders byte-identical closed Cascader markup with the selected path label", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(probe(false))),
    renderToString(createSSRApp(probe(false))),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /id="ssr-place-trigger"[^>]*role="combobox"/);
  assert.match(html, /aria-expanded="false"/);
  assert.match(html, /France \/ Paris/);
  assert.match(html, /type="hidden"[^>]*name="place"[^>]*value="fr \/ paris"/);
  assert.doesNotMatch(html, /data-vize-ui="cascader-column"/);
});

test("renders byte-identical open Cascader markup with columns for the selected path", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(probe(true))),
    renderToString(createSSRApp(probe(true))),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /id="ssr-place-column-0"[^>]*role="listbox"/);
  assert.match(html, /id="ssr-place-column-1"/);
  assert.match(html, /aria-activedescendant="ssr-place-option-1-0"/);
  assert.match(html, /id="ssr-place-option-1-0"[^>]*role="option"[^>]*aria-selected="true"/);
});

test("hydrates closed and open Cascaders without mismatches", async () => {
  await assertHydrates(probe(false));
  await assertHydrates(probe(true));
});
