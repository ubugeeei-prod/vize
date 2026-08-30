import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Listbox from "./listbox.vue";
import ListboxItem from "./listbox-item.vue";

const SsrProbe = defineComponent({
  name: "ListboxSsrProbe",
  setup: () => () =>
    h(
      Listbox,
      {
        ariaDescribedby: "letters-help",
        ariaLabel: "Letters",
        defaultValue: ["bravo"],
        required: true,
        selectionMode: "multiple",
      },
      () => [
        h(ListboxItem, { id: "alpha-option", textValue: "Alpha", value: "alpha" }, () => "Alpha"),
        h(
          ListboxItem,
          { textValue: "Bravo", value: "bravo" },
          {
            default: () => "Bravo",
            indicator: () => h("span", "Selected"),
          },
        ),
      ],
    ),
});

test("renders byte-identical Listbox markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div/);
  assert.match(html, /id="vize-v-\d+-listbox"/);
  assert.match(html, /role="listbox"/);
  assert.match(html, /aria-label="Letters"/);
  assert.match(html, /aria-describedby="letters-help"/);
  assert.match(html, /aria-orientation="vertical"/);
  assert.match(html, /aria-multiselectable="true"/);
  assert.match(html, /aria-required="true"/);
  assert.match(html, /data-vize-ui="listbox"/);
  assert.match(html, /data-state="selected"/);
  assert.match(html, /data-selection-mode="multiple"/);
  assert.match(html, /data-selection-count="1"/);
  assert.match(html, /role="option"/);
  assert.match(html, /id="alpha-option"/);
  assert.match(html, /aria-selected="true"/);
  assert.match(html, /data-vize-ui="listbox-item"/);
  assert.match(html, /data-state="selected"/);
});

test("hydrates generated ids and selected state without replacing SSR nodes", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverItems = [...host.querySelectorAll<HTMLElement>("[data-vize-ui='listbox-item']")];
  const serverIds = serverItems.map((item) => item.id);

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
    const hydratedItems = [...host.querySelectorAll<HTMLElement>("[data-vize-ui='listbox-item']")];
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(
      hydratedItems.map((item) => item.id),
      serverIds,
    );
    assert.equal(hydratedItems[0]?.getAttribute("aria-selected"), "false");
    assert.equal(hydratedItems[1]?.getAttribute("aria-selected"), "true");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
