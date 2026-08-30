import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, onMounted, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useScrollLock } from "./scroll-lock.ts";
import { preserveDocumentPresentation } from "./scroll-lock-test-utils.ts";

const ScrollLockSsrProbe = defineComponent({
  name: "ScrollLockSsrProbe",
  setup() {
    const root = ref<HTMLElement | null>(null);
    const ownerDocument = ref<Document | null>(null);
    const lock = useScrollLock({ document: ownerDocument, strategy: "overflow" });
    onMounted(() => {
      ownerDocument.value = root.value?.ownerDocument ?? null;
    });
    return () =>
      h(
        "div",
        {
          "data-active": String(lock.isActive.value),
          "data-locked": String(lock.isLocked.value),
          ref: root,
        },
        "Modal",
      );
  },
});

test("renders deterministic inactive markup without global document access", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(ScrollLockSsrProbe)),
    renderToString(createSSRApp(ScrollLockSsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], '<div data-active="false" data-locked="false">Modal</div>');
  assert.doesNotMatch(outputs[0], /overflow|position|function/);
});

test("hydrates in place, locks after mount, and restores on unmount", async () => {
  const restorePresentation = preserveDocumentPresentation(document);
  const serverHtml = await renderToString(createSSRApp(ScrollLockSsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(ScrollLockSsrProbe);
  try {
    app.mount(host);
    await nextTick();
    assert.equal(host.firstElementChild, serverRoot);
    assert.equal(serverRoot?.getAttribute("data-active"), "true");
    assert.equal(serverRoot?.getAttribute("data-locked"), "true");
    assert.equal(document.documentElement.style.getPropertyValue("overflow"), "hidden");
    assert.deepEqual(diagnostics, []);
    app.unmount();
    assert.equal(document.documentElement.style.getPropertyValue("overflow"), "");
    assert.equal(document.documentElement.hasAttribute("data-vize-scroll-locked"), false);
  } finally {
    if ((app as { _container?: Element | null })._container) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
    restorePresentation();
  }
});
