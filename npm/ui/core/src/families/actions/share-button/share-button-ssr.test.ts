import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ShareButton from "./share-button.vue";
import type { ShareButtonAction } from "./share-button.ts";

let actionCalls = 0;

const inertAction: ShareButtonAction = () => {
  actionCalls += 1;
};

const SsrProbe = defineComponent({
  name: "ShareButtonSsrProbe",
  setup() {
    return () =>
      h(
        ShareButton,
        {
          action: inertAction,
          ariaDescribedby: "share-help",
          idleLabel: "Share report",
          text: "Read the report",
          title: "Report",
          url: "https://vize.dev/report",
        },
        {
          default: ({ label, state }) =>
            h("span", { "data-rendered-state": state, id: "share-label" }, label),
        },
      );
  },
});

const DefaultActionSsrProbe = defineComponent({
  name: "ShareButtonDefaultActionSsrProbe",
  setup() {
    return () => h(ShareButton, { idleLabel: "Share link", title: "SSR share" });
  },
});

test("renders byte-identical share-button markup without invoking the action", async () => {
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
  assert.match(html, /aria-describedby="share-help"/);
  assert.match(html, /data-vize-ui="share-button"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-rendered-state="idle"/);
  assert.match(html, /Share report/);
  assert.doesNotMatch(
    html,
    /data-sharing=|data-disabled=|aria-busy=|aria-disabled=|data-title=|data-text=|data-url=/,
  );
});

test("server rendering with the default action does not read navigator globals", async () => {
  const navigatorDescriptor = Object.getOwnPropertyDescriptor(globalThis, "navigator");
  Object.defineProperty(globalThis, "navigator", {
    configurable: true,
    get() {
      throw new Error("navigator must not be read during share-button SSR");
    },
  });

  try {
    const html = await renderToString(createSSRApp(DefaultActionSsrProbe));
    assert.match(html, /data-vize-ui="share-button"/);
    assert.match(html, /Share link/);
  } finally {
    if (navigatorDescriptor === undefined) {
      delete (globalThis as Record<string, unknown>).navigator;
    } else {
      Object.defineProperty(globalThis, "navigator", navigatorDescriptor);
    }
  }
});

test("hydrates share-button markup without warnings, root replacement, or eager actions", async () => {
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
    const button = host.querySelector('[data-vize-ui="share-button"]');
    assert.ok(button instanceof HTMLButtonElement);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(button.getAttribute("data-state"), "idle");
    assert.equal(button.getAttribute("data-sharing"), null);
    assert.equal(button.textContent, "Share report");
    assert.equal(actionCalls, 0);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
