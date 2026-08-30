import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import Callout from "./callout.vue";

const SsrProbe = defineComponent({
  name: "CalloutSsrProbe",
  setup() {
    return () =>
      h(
        Callout,
        {
          density: "compact",
          tone: "info",
        },
        {
          default: () => "Uploads continue in the background.",
          description: () => "Large files may take a few minutes.",
          icon: () => "i",
          title: () => "Upload queued",
        },
      );
  },
});

test("renders byte-identical labelled note markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<section/);
  assert.match(html, /role="note"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="callout"/);
  assert.match(html, /data-state="open"/);
  assert.match(html, /data-tone="info"/);
  assert.match(html, /data-density="compact"/);
  assert.match(html, /data-aria-state="note"/);
  assert.match(html, /data-live="off"/);
  assert.match(html, /aria-labelledby="vize-v-[^"]+-callout-title"/);
  assert.match(html, /aria-describedby="vize-v-[^"]+-callout-description"/);
  assert.match(html, /data-vize-ui="callout-icon" aria-hidden="true"/);
  assert.match(html, /data-vize-ui="callout-title"/);
  assert.match(html, /data-vize-ui="callout-description"/);
  assert.match(html, /Upload queued/);
  assert.match(html, /Large files may take a few minutes/);
  assert.doesNotMatch(html, /class=|style=|tabindex=|function/);
  assert.doesNotMatch(html, /aria-live|aria-atomic/);
});

test("hydrates generated title and description references without replacing the root", async () => {
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
    const callout = host.querySelector("[data-vize-ui='callout']");
    const title = host.querySelector("[data-vize-ui='callout-title']");
    const description = host.querySelector("[data-vize-ui='callout-description']");

    assert.ok(callout instanceof HTMLElement);
    assert.ok(title instanceof HTMLElement);
    assert.ok(description instanceof HTMLElement);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(callout.getAttribute("aria-labelledby"), title.id);
    assert.equal(callout.getAttribute("aria-describedby"), description.id);
    assert.equal(callout.getAttribute("data-live"), "off");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});

test("renders assertive server markup when alert semantics are requested", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "CalloutAlertSsrProbe",
      setup() {
        return () =>
          h(
            Callout,
            {
              ariaLabel: "Deploy failed",
              role: "alert",
              tone: "danger",
            },
            {
              default: () => "Check the release key.",
            },
          );
      },
    }),
  );

  assert.match(html, /^<section/);
  assert.match(html, /role="alert"/);
  assert.match(html, /aria-label="Deploy failed"/);
  assert.match(html, /aria-live="assertive"/);
  assert.match(html, /aria-atomic="true"/);
  assert.match(html, /data-live="assertive"/);
  assert.match(html, /data-tone="danger"/);
  assert.match(html, /Check the release key/);
  assert.doesNotMatch(html, /aria-labelledby=|aria-describedby=/);
});

test("renders decorative closed markup without live-region attributes", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "CalloutDecorativeSsrProbe",
      setup() {
        return () =>
          h(
            Callout,
            {
              ariaHidden: true,
              ariaLabel: "Ignored",
              open: false,
              role: "status",
            },
            {
              title: () => "Hidden status",
            },
          );
      },
    }),
  );

  assert.match(html, /^<section/);
  assert.match(html, /hidden/);
  assert.match(html, /aria-hidden="true"/);
  assert.match(html, /data-state="closed"/);
  assert.match(html, /data-aria-state="decorative"/);
  assert.match(html, /data-live="off"/);
  assert.doesNotMatch(html, /role="status"/);
  assert.doesNotMatch(html, /aria-label=|aria-live|aria-atomic/);
});
