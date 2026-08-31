import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import CopyButton from "./copy-button.vue";

let writerCalls = 0;

const SsrProbe = defineComponent({
  name: "CopyButtonSsrProbe",
  setup() {
    return () =>
      h(
        CopyButton,
        {
          ariaDescribedby: "copy-help",
          copiedLabel: "Copied link",
          idleLabel: "Copy link",
          value: "https://vize.dev/docs",
          writer: () => {
            writerCalls += 1;
          },
        },
        {
          default: ({ label, state }) =>
            h("span", { "data-rendered-state": state, id: "copy-label" }, label),
        },
      );
  },
});

test("renders byte-identical copy-button markup without invoking the writer", async () => {
  writerCalls = 0;
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(writerCalls, 0);
  const html = outputs[0] ?? "";

  assert.match(html, /^<button/);
  assert.match(html, /type="button"/);
  assert.match(html, /aria-describedby="copy-help"/);
  assert.match(html, /data-vize-ui="copy-button"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-rendered-state="idle"/);
  assert.match(html, /Copy link/);
  assert.doesNotMatch(html, /data-writing=|data-disabled=|aria-busy=|aria-disabled=|data-value=/);
});

test("hydrates copy-button markup without warnings, root replacement, or eager writes", async () => {
  writerCalls = 0;
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
    const button = host.querySelector('[data-vize-ui="copy-button"]');
    assert.ok(button instanceof HTMLButtonElement);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(button.getAttribute("data-state"), "idle");
    assert.equal(button.textContent, "Copy link");
    assert.equal(writerCalls, 0);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
