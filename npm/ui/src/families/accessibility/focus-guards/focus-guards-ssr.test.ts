import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, onMounted, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { focusGuardPreset, useFocusGuards } from "./focus-guards.ts";

const FocusGuardsSsrProbe = defineComponent({
  name: "FocusGuardsSsrProbe",
  setup() {
    const root = ref<HTMLElement | null>(null);
    const guards = useFocusGuards({ root });
    onMounted(() => guards.refresh());
    return () =>
      h("div", [
        h("span", { ...guards.beforeProps, style: focusGuardPreset }),
        h("div", { ref: root }, [h("button", { type: "button" }, "Inside")]),
        h("span", { ...guards.afterProps, style: focusGuardPreset }),
      ]);
  },
});

test("renders deterministic inactive sentinels without global DOM access", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(FocusGuardsSsrProbe)),
    renderToString(createSSRApp(FocusGuardsSsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0]!, /data-vize-focus-guard="before" tabindex="-1"/);
  assert.match(outputs[0]!, /data-vize-focus-guard="after" tabindex="-1"/);
  assert.doesNotMatch(outputs[0]!, /onFocus|function/);
});

test("hydrates sentinels in place and activates their reactive tabindex", async () => {
  const serverHtml = await renderToString(createSSRApp(FocusGuardsSsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const before = host.querySelector('[data-vize-focus-guard="before"]');
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(FocusGuardsSsrProbe);
  try {
    app.mount(host);
    await nextTick();
    assert.equal(host.querySelector('[data-vize-focus-guard="before"]'), before);
    assert.equal(before?.getAttribute("tabindex"), "0");
    assert.deepEqual(diagnostics, []);
    app.unmount();
  } finally {
    if ((app as { _container?: Element | null })._container) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
