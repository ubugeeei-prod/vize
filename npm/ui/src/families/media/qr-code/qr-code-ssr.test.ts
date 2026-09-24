import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import QrCode from "./qr-code.vue";

const SsrProbe = defineComponent({
  name: "QrCodeSsrProbe",
  setup: () => () =>
    h("div", null, [
      h(
        QrCode,
        { value: "https://vizejs.dev/ui", errorCorrection: "H" },
        {
          overlay: ({ dimension }: { readonly dimension: number }) =>
            h("circle", { cx: dimension / 2, cy: dimension / 2, r: 3 }),
        },
      ),
      h(QrCode, { value: "x".repeat(40), version: 1 }, { fallback: () => "Too long" }),
    ]),
});

test("renders byte-identical QR code markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /<svg[^>]*data-vize-ui="qr-code"/);
  assert.match(html, /role="img"/);
  assert.match(html, /aria-label="https:\/\/vizejs.dev\/ui"/);
  assert.match(html, /<title[^>]*>https:\/\/vizejs.dev\/ui<\/title>/);
  assert.match(html, /data-error-correction="H"/);
  assert.match(html, /<path[^>]*d="M4 4h7v1h-7z/);
  assert.match(html, /<circle/);
  assert.match(
    html,
    /<span[^>]*data-state="error"[^>]*data-error="VIZE_UI_QR_DATA_TOO_LONG"[^>]*>(?:<!--\[-->)?Too long/,
  );
});

test("hydrates the QR code without replacing server nodes or warning", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverSvg = host.querySelector("svg");
  const serverPath = serverSvg?.querySelector("path")?.getAttribute("d");
  assert.ok(serverSvg);

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
    assert.ok(host.querySelector("svg") === serverSvg);
    assert.equal(serverSvg.querySelector("path")?.getAttribute("d"), serverPath);
    assert.equal(host.querySelector('[data-state="error"]')?.textContent, "Too long");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
