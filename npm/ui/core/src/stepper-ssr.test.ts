import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import StepperContent from "./stepper-content.vue";
import StepperItem from "./stepper-item.vue";
import StepperList from "./stepper-list.vue";
import StepperRoot from "./stepper-root.vue";
import StepperTrigger from "./stepper-trigger.vue";

const SsrProbe = defineComponent({
  name: "StepperSsrProbe",
  setup: () => () =>
    h(StepperRoot, { defaultValue: "shipping" }, () => [
      h(StepperList, { ariaLabel: "Checkout steps" }, () => [
        h(StepperItem, { completed: true, textValue: "Shipping", value: "shipping" }, () =>
          h(StepperTrigger, () => "Shipping"),
        ),
        h(StepperItem, { textValue: "Billing", value: "billing" }, () =>
          h(StepperTrigger, () => "Billing"),
        ),
      ]),
      h(StepperContent, { value: "shipping" }, () => "Shipping panel"),
      h(StepperContent, { value: "billing" }, () => "Billing panel"),
    ]),
});

test("renders byte-identical Stepper markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div/);
  assert.match(html, /id="vize-v-\d+-stepper"/);
  assert.match(html, /role="list"/);
  assert.match(html, /aria-label="Checkout steps"/);
  assert.match(html, /id="vize-v-\d+-stepper-trigger-value-shipping"/);
  assert.match(html, /aria-current="step"/);
  assert.match(html, /aria-controls="vize-v-\d+-stepper-content-value-shipping"/);
  assert.match(html, /data-state="current"/);
  assert.match(html, /data-completed="true"/);
  assert.match(html, /id="vize-v-\d+-stepper-content-value-shipping"/);
  assert.match(html, /role="region"/);
  assert.match(html, /tabindex="0"/);
  assert.match(html, /id="vize-v-\d+-stepper-content-value-billing"[^>]*hidden/);
});

test("hydrates generated Stepper ids without changing the server contract", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverTriggers = [
    ...host.querySelectorAll<HTMLButtonElement>("[data-vize-ui='stepper-trigger']"),
  ];
  const serverPanels = [
    ...host.querySelectorAll<HTMLDivElement>("[data-vize-ui='stepper-content']"),
  ];
  assert.ok(serverRoot);
  assert.equal(serverTriggers.length, 2);
  assert.equal(serverPanels.length, 2);
  const triggerIds = serverTriggers.map((trigger) => trigger.id);
  const panelIds = serverPanels.map((panel) => panel.id);

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
    const hydratedTriggers = [
      ...host.querySelectorAll<HTMLButtonElement>("[data-vize-ui='stepper-trigger']"),
    ];
    const hydratedPanels = [
      ...host.querySelectorAll<HTMLDivElement>("[data-vize-ui='stepper-content']"),
    ];
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(
      hydratedTriggers.map((trigger) => trigger.id),
      triggerIds,
    );
    assert.deepEqual(
      hydratedPanels.map((panel) => panel.id),
      panelIds,
    );
    assert.equal(hydratedTriggers[0]?.getAttribute("aria-current"), "step");
    assert.equal(hydratedPanels[1]?.hidden, true);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
