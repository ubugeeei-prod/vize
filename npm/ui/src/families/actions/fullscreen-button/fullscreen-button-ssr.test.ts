import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import FullscreenButton from "./fullscreen-button.vue";
import type { FullscreenButtonController } from "./fullscreen-button.ts";

let controllerCalls = 0;

const inertController: FullscreenButtonController = {
  getFullscreenElement() {
    controllerCalls += 1;
    return null;
  },
  requestFullscreen() {
    controllerCalls += 1;
  },
  exitFullscreen() {
    controllerCalls += 1;
  },
};

const SsrProbe = defineComponent({
  name: "FullscreenButtonSsrProbe",
  setup() {
    return () =>
      h(
        FullscreenButton,
        {
          ariaDescribedby: "fullscreen-help",
          controller: inertController,
          enterLabel: "Enter report fullscreen",
          exitLabel: "Exit report fullscreen",
        },
        {
          default: ({ label, state }) =>
            h("span", { "data-rendered-state": state, id: "fullscreen-label" }, label),
        },
      );
  },
});

const DefaultControllerSsrProbe = defineComponent({
  name: "FullscreenButtonDefaultControllerSsrProbe",
  setup() {
    return () => h(FullscreenButton, { enterLabel: "Enter fullscreen" });
  },
});

test("renders byte-identical fullscreen-button markup without invoking the controller", async () => {
  controllerCalls = 0;
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(controllerCalls, 0);
  const html = outputs[0] ?? "";

  assert.match(html, /^<button/);
  assert.match(html, /type="button"/);
  assert.match(html, /aria-describedby="fullscreen-help"/);
  assert.match(html, /aria-pressed="false"/);
  assert.match(html, /data-vize-ui="fullscreen-button"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-rendered-state="idle"/);
  assert.match(html, /Enter report fullscreen/);
  assert.doesNotMatch(html, /data-pending=|data-active=|data-disabled=|aria-busy=|data-target=/);
});

test("server rendering with the default controller does not read document globals", async () => {
  const documentDescriptor = Object.getOwnPropertyDescriptor(globalThis, "document");
  Object.defineProperty(globalThis, "document", {
    configurable: true,
    get() {
      throw new Error("document must not be read during fullscreen-button SSR");
    },
  });

  try {
    const html = await renderToString(createSSRApp(DefaultControllerSsrProbe));
    assert.match(html, /data-vize-ui="fullscreen-button"/);
    assert.match(html, /Enter fullscreen/);
  } finally {
    if (documentDescriptor === undefined) {
      delete (globalThis as Record<string, unknown>).document;
    } else {
      Object.defineProperty(globalThis, "document", documentDescriptor);
    }
  }
});

test("hydrates fullscreen-button markup without warnings, root replacement, or eager actions", async () => {
  controllerCalls = 0;
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
    const button = host.querySelector('[data-vize-ui="fullscreen-button"]');
    assert.ok(button instanceof HTMLButtonElement);
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(button.getAttribute("data-state"), "idle");
    assert.equal(button.getAttribute("aria-pressed"), "false");
    assert.equal(button.textContent, "Enter report fullscreen");
    assert.equal(controllerCalls, 0);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
