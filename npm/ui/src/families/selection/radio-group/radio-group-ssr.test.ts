import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import RadioGroup from "./radio-group.vue";
import RadioGroupItem from "./radio-group-item.vue";

const SsrProbe = defineComponent({
  name: "RadioGroupSsrProbe",
  setup: () => () =>
    h(
      RadioGroup,
      {
        ariaDescribedby: "frequency-help",
        ariaLabel: "Email frequency",
        defaultValue: "weekly",
        name: "frequency",
        required: true,
      },
      () => [
        h("label", [h(RadioGroupItem, { value: "daily" }), "Daily"]),
        h("label", [h(RadioGroupItem, { value: "weekly" }), "Weekly"]),
      ],
    ),
});

test("renders byte-identical radio group markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div/);
  assert.match(html, /id="vize-v-\d+-radio-group"/);
  assert.match(html, /role="radiogroup"/);
  assert.match(html, /aria-label="Email frequency"/);
  assert.match(html, /aria-describedby="frequency-help"/);
  assert.match(html, /aria-orientation="vertical"/);
  assert.match(html, /aria-required="true"/);
  assert.match(html, /data-vize-ui="radio-group"/);
  assert.match(html, /data-state="selected"/);
  assert.match(html, /data-orientation="vertical"/);
  assert.match(html, /type="radio"/);
  assert.match(html, /name="frequency"/);
  assert.match(html, /value="weekly"/);
  assert.match(html, /checked/);
});

test("hydrates generated ids without changing the server contract", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverItems = [
    ...host.querySelectorAll<HTMLInputElement>("[data-vize-ui='radio-group-item']"),
  ];
  assert.ok(serverRoot);
  assert.equal(serverItems.length, 2);
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
    const hydratedItems = [
      ...host.querySelectorAll<HTMLInputElement>("[data-vize-ui='radio-group-item']"),
    ];
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(
      hydratedItems.map((item) => item.id),
      serverIds,
    );
    assert.equal(hydratedItems[0]?.checked, false);
    assert.equal(hydratedItems[1]?.checked, true);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
