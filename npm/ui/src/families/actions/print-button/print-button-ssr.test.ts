import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import PrintButton from "./print-button.vue";

let actionCalls = 0;

const SsrProbe = defineComponent({
  name: "PrintButtonSsrProbe",
  setup() {
    return () =>
      h(
        PrintButton,
        {
          ariaDescribedby: "print-help",
          idleLabel: "Print report",
          printedLabel: "Report printed",
          action: () => {
            actionCalls += 1;
          },
        },
        {
          default: ({ label, state }) =>
            h("span", { "data-rendered-state": state, id: "print-label" }, label),
        },
      );
  },
});

test("renders byte-identical print-button markup without invoking the action", async () => {
  actionCalls = 0;
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(actionCalls, 0);
  const html = outputs[0] ?? "";

  assert.match(html, /^<button/);
  assert.match(html, /type="button"/);
  assert.match(html, /aria-describedby="print-help"/);
  assert.match(html, /data-vize-ui="print-button"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-rendered-state="idle"/);
  assert.match(html, /Print report/);
  assert.doesNotMatch(html, /data-printing=|data-disabled=|aria-busy=|aria-disabled=|data-action=/);
});

test("hydrates print-button markup without warnings, root replacement, or eager actions", async () => {
  actionCalls = 0;
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
    const button = host.querySelector('[data-vize-ui="print-button"]');
    assert.ok(button instanceof HTMLButtonElement);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(button.getAttribute("data-state"), "idle");
    assert.equal(button.textContent, "Print report");
    assert.equal(actionCalls, 0);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
