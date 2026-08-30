import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useInertOutside } from "./inert-outside.ts";

const InertOutsideSsrProbe = defineComponent({
  name: "InertOutsideSsrProbe",
  setup() {
    const root = ref<Element | null>(null);
    const isolation = useInertOutside({ root });
    return () => h("div", { "data-active": String(isolation.isActive.value), ref: root }, "Modal");
  },
});

test("renders deterministic inactive markup without document access", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(InertOutsideSsrProbe)),
    renderToString(createSSRApp(InertOutsideSsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], '<div data-active="false">Modal</div>');
  assert.doesNotMatch(outputs[0], /aria-hidden|inert|function/);
});

test("hydrates in place, isolates after mount, and restores on unmount", async () => {
  const serverHtml = await renderToString(createSSRApp(InertOutsideSsrProbe));
  const outside = document.createElement("div");
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(outside, host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(InertOutsideSsrProbe);
  try {
    app.mount(host);
    await nextTick();
    assert.equal(host.firstElementChild, serverRoot);
    assert.equal(outside.getAttribute("aria-hidden"), "true");
    assert.equal(outside.hasAttribute("inert"), true);
    assert.deepEqual(diagnostics, []);
    app.unmount();
    assert.equal(outside.hasAttribute("aria-hidden"), false);
    assert.equal(outside.hasAttribute("inert"), false);
  } finally {
    if ((app as { _container?: Element | null })._container) app.unmount();
    outside.remove();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
