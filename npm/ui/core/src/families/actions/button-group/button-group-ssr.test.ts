import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ButtonGroup from "./button-group.vue";
import ButtonGroupItem from "./button-group-item.vue";

const SsrProbe = defineComponent({
  name: "ButtonGroupSsrProbe",
  setup: () => () =>
    h(
      ButtonGroup,
      {
        ariaDescribedby: "editor-help",
        ariaLabel: "Editor actions",
        role: "toolbar",
      },
      () => [
        h(ButtonGroupItem, { value: "save" }, () => "Save"),
        h(ButtonGroupItem, { value: "publish" }, () => "Publish"),
      ],
    ),
});

test("renders byte-identical button group markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div/);
  assert.match(html, /role="toolbar"/);
  assert.match(html, /aria-label="Editor actions"/);
  assert.match(html, /aria-describedby="editor-help"/);
  assert.match(html, /aria-orientation="horizontal"/);
  assert.match(html, /data-vize-ui="button-group"/);
  assert.match(html, /data-roving-focus="true"/);
  assert.match(html, /data-vize-ui="button-group-item"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /Save/);
});

test("hydrates button group markup without changing the server contract", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  assert.ok(serverRoot);

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
    const save = host.querySelector<HTMLButtonElement>(
      '[data-vize-ui="button-group-item"][data-value="save"]',
    );
    const publish = host.querySelector<HTMLButtonElement>(
      '[data-vize-ui="button-group-item"][data-value="publish"]',
    );
    assert.ok(host.firstElementChild === serverRoot);
    assert.ok(save);
    assert.ok(publish);
    assert.equal(save.getAttribute("tabindex"), "0");
    assert.equal(publish.getAttribute("tabindex"), "-1");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
