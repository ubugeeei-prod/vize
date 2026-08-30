import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Spinner from "./spinner.vue";

const SsrProbe = defineComponent({
  name: "SpinnerSsrProbe",
  setup() {
    return () =>
      h(
        Spinner,
        {
          ariaLabel: "Syncing profile",
        },
        {
          default: () => "Syncing",
        },
      );
  },
});

test("renders byte-identical status markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<span/);
  assert.match(html, /id="vize-v-\d+-spinner"/);
  assert.match(html, /role="status"/);
  assert.match(html, /aria-label="Syncing profile"/);
  assert.match(html, /aria-live="polite"/);
  assert.match(html, /aria-atomic="true"/);
  assert.match(html, /data-vize-ui="spinner"/);
  assert.match(html, /data-state="loading"/);
  assert.match(html, /data-progress-state="none"/);
  assert.match(html, /Syncing/);
  assert.doesNotMatch(html, /aria-valuenow|tabindex|function/);
});

test("hydrates generated ids without replacing the spinner root", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  assert.ok(serverRoot);
  const serverId = serverRoot.id;
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
    const hydrated = host.querySelector<HTMLElement>("[data-vize-ui='spinner']");
    assert.ok(hydrated);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(hydrated.id, serverId);
    assert.equal(hydrated.getAttribute("role"), "status");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("renders determinate progressbar markup without live-region attributes", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "SpinnerProgressbarSsrProbe",
      setup() {
        return () =>
          h(Spinner, {
            ariaLabel: "Upload progress",
            ariaValueText: "3 of 4 chunks",
            as: "div",
            max: 4,
            role: "progressbar",
            value: 3,
          });
      },
    }),
  );

  assert.match(html, /^<div/);
  assert.match(html, /role="progressbar"/);
  assert.match(html, /aria-valuemin="0"/);
  assert.match(html, /aria-valuemax="4"/);
  assert.match(html, /aria-valuenow="3"/);
  assert.match(html, /aria-valuetext="3 of 4 chunks"/);
  assert.match(html, /data-progress-state="determinate"/);
  assert.match(html, /data-percent="75"/);
  assert.doesNotMatch(html, /aria-live|aria-atomic/);
});
