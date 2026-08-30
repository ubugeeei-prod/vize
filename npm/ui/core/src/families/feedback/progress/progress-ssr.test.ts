import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Progress from "./progress.vue";

const SsrProbe = defineComponent({
  name: "ProgressSsrProbe",
  setup() {
    return () =>
      h(
        Progress,
        {
          ariaDescribedby: "upload-help",
          ariaLabel: "Upload progress",
          ariaValueText: "40 of 100 files",
          id: "upload-progress",
          max: 100,
          value: 40,
        },
        {
          default: () => "40%",
        },
      );
  },
});

test("renders byte-identical native progress markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<progress/);
  assert.match(html, /id="upload-progress"/);
  assert.match(html, /value="40"/);
  assert.match(html, /max="100"/);
  assert.match(html, /aria-label="Upload progress"/);
  assert.match(html, /aria-describedby="upload-help"/);
  assert.match(html, /aria-valuetext="40 of 100 files"/);
  assert.match(html, /data-vize-ui="progress"/);
  assert.match(html, /data-state="loading"/);
  assert.match(html, /40%/);
  assert.doesNotMatch(html, /aria-live|tabindex|function/);
});

test("hydrates native progress markup without replacing the root", async () => {
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
    const hydrated = host.querySelector<HTMLProgressElement>("[data-vize-ui='progress']");
    assert.ok(hydrated instanceof HTMLProgressElement);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(hydrated.value, 40);
    assert.equal(hydrated.max, 100);
    assert.equal(hydrated.getAttribute("data-state"), "loading");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("renders indeterminate native progress markup without value", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "ProgressIndeterminateSsrProbe",
      setup() {
        return () =>
          h(Progress, {
            ariaLabel: "Import progress",
            value: null,
          });
      },
    }),
  );

  assert.match(html, /^<progress/);
  assert.match(html, /aria-label="Import progress"/);
  assert.match(html, /max="100"/);
  assert.match(html, /data-state="indeterminate"/);
  assert.match(html, /data-indeterminate="true"/);
  assert.doesNotMatch(html, /value=|aria-live|tabindex/);
});
