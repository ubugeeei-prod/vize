import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ToggleGroup from "./toggle-group.vue";
import ToggleGroupItem from "./toggle-group-item.vue";

const SsrProbe = defineComponent({
  name: "ToggleGroupSsrProbe",
  setup: () => () =>
    h(
      ToggleGroup,
      {
        ariaDescribedby: "formatting-help",
        ariaLabel: "Formatting",
        defaultValue: ["bold"],
        type: "multiple",
      },
      () => [
        h(ToggleGroupItem, { value: "bold" }, () => "Bold"),
        h(ToggleGroupItem, { value: "italic" }, () => "Italic"),
      ],
    ),
});

test("renders byte-identical toggle group markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div/);
  assert.match(html, /role="group"/);
  assert.match(html, /aria-label="Formatting"/);
  assert.match(html, /aria-describedby="formatting-help"/);
  assert.match(html, /data-vize-ui="toggle-group"/);
  assert.match(html, /data-state="selected"/);
  assert.match(html, /data-type="multiple"/);
  assert.match(html, /data-value="bold"/);
  assert.match(html, /data-vize-ui="toggle-group-item"/);
  assert.match(html, /aria-pressed="true"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /Bold/);
});

test("hydrates toggle group markup without changing the server contract", async () => {
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
    const bold = host.querySelector<HTMLButtonElement>(
      "[data-vize-ui='toggle-group-item'][data-value='bold']",
    );
    const italic = host.querySelector<HTMLButtonElement>(
      "[data-vize-ui='toggle-group-item'][data-value='italic']",
    );
    assert.ok(host.firstElementChild === serverRoot);
    assert.ok(bold);
    assert.ok(italic);
    assert.equal(bold.getAttribute("aria-pressed"), "true");
    assert.equal(italic.getAttribute("aria-pressed"), "false");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
