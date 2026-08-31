import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import NativeSelect from "./native-select.vue";

const SsrProbe = defineComponent({
  name: "NativeSelectSsrProbe",
  setup: () => () =>
    h(NativeSelect, {
      ariaDescribedby: "status-help",
      ariaLabel: "Statuses",
      defaultValue: ["todo", "done"],
      multiple: true,
      name: "status",
      options: [
        { label: "Todo", value: "todo" },
        { label: "Doing", value: "doing" },
        { disabled: true, label: "Done", value: "done" },
      ],
      required: true,
      size: 3,
    }),
});

test("renders byte-identical NativeSelect markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<select/);
  assert.match(html, /id="vize-v-\d+-native-select"/);
  assert.match(html, /name="status"/);
  assert.match(html, /multiple/);
  assert.match(html, /size="3"/);
  assert.match(html, /required/);
  assert.match(html, /aria-label="Statuses"/);
  assert.match(html, /aria-describedby="status-help"/);
  assert.match(html, /data-vize-ui="native-select"/);
  assert.match(html, /data-state="selected"/);
  assert.match(html, /data-selection-mode="multiple"/);
  assert.match(html, /data-selection-count="2"/);
  assert.match(html, /data-direction="ltr"/);
  assert.match(html, /<option[^>]+value="todo"[^>]+selected/);
  assert.match(html, /<option[^>]+value="doing"/);
  assert.match(html, /<option[^>]+value="done"[^>]+disabled[^>]+selected/);
});

test("hydrates generated ids and selected state without replacing SSR nodes", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;

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
    const select = host.querySelector('[data-vize-ui="native-select"]');
    assert.ok(host.firstElementChild === serverRoot);
    assert.ok(select instanceof HTMLSelectElement);
    assert.deepEqual(
      [...select.selectedOptions].map((option) => option.value),
      ["todo", "done"],
    );
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
