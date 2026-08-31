import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ProgressBar from "./progress-bar.vue";

const SsrProbe = defineComponent({
  name: "ProgressBarSsrProbe",
  setup() {
    return () =>
      h(
        ProgressBar,
        {
          ariaDescribedby: "upload-help",
          id: "upload-progress",
          label: "Upload progress",
          max: 100,
          min: 20,
          value: 40,
          valueLabel: "25%",
        },
        {
          indicator: () => "25%",
        },
      );
  },
});

test("renders byte-identical labelled progressbar markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div/);
  assert.match(html, /id="upload-progress"/);
  assert.match(html, /role="progressbar"/);
  assert.match(html, /aria-labelledby="upload-progress-label"/);
  assert.match(html, /aria-describedby="upload-help"/);
  assert.match(html, /aria-valuemin="20"/);
  assert.match(html, /aria-valuemax="100"/);
  assert.match(html, /aria-valuenow="40"/);
  assert.match(html, /aria-valuetext="25%"/);
  assert.match(html, /data-vize-ui="progress-bar"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="loading"/);
  assert.match(html, /data-percent="25"/);
  assert.match(html, /--vize-ui-progress-bar-percent:25%/);
  assert.match(html, /data-vize-ui="progress-bar-track"/);
  assert.match(html, /part="indicator"/);
  assert.match(html, /Upload progress/);
  assert.doesNotMatch(html, /aria-live|tabindex|function/);
});

test("hydrates labelled markup without replacing the progressbar root", async () => {
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
    const hydrated = host.querySelector<HTMLElement>("[data-vize-ui='progress-bar']");
    assert.ok(hydrated);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(hydrated.getAttribute("role"), "progressbar");
    assert.equal(hydrated.getAttribute("aria-valuenow"), "40");
    assert.equal(hydrated.getAttribute("data-state"), "loading");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("renders indeterminate server markup without aria-valuenow", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "ProgressBarIndeterminateSsrProbe",
      setup() {
        return () =>
          h(ProgressBar, {
            ariaLabel: "Import progress",
            value: null,
            valueLabel: "Waiting",
          });
      },
    }),
  );

  assert.match(html, /^<div/);
  assert.match(html, /role="progressbar"/);
  assert.match(html, /aria-label="Import progress"/);
  assert.match(html, /aria-valuetext="Waiting"/);
  assert.match(html, /data-state="indeterminate"/);
  assert.match(html, /data-indeterminate="true"/);
  assert.doesNotMatch(html, /aria-valuenow|data-value=/);
});
