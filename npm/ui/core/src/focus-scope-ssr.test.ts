import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useFocusScope } from "./focus-scope.ts";

const FocusScopeSsrProbe = defineComponent({
  name: "FocusScopeSsrProbe",
  setup() {
    const root = ref<Element | null>(null);
    const scope = useFocusScope({ autoFocus: true, contain: true, restoreFocus: true, root });
    return () =>
      h(
        "div",
        { "data-active": String(scope.isActive.value), ref: root },
        h("button", { type: "button" }, "Inside"),
      );
  },
});

test("renders deterministic inactive markup without DOM access or handlers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(FocusScopeSsrProbe)),
    renderToString(createSSRApp(FocusScopeSsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], '<div data-active="false"><button type="button">Inside</button></div>');
  assert.doesNotMatch(outputs[0], /focus|keydown|function/);
});

test("hydrates in place, activates after mount, and restores on unmount", async () => {
  const serverHtml = await renderToString(createSSRApp(FocusScopeSsrProbe));
  const trigger = document.createElement("button");
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(trigger, host);
  trigger.focus();
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  let app: ReturnType<typeof createSSRApp> | undefined;
  try {
    app = createSSRApp(FocusScopeSsrProbe);
    app.mount(host);
    await nextTick();
    assert.equal(host.firstElementChild, serverRoot);
    assert.equal(serverRoot?.getAttribute("data-active"), "true");
    assert.equal(document.activeElement, serverRoot?.querySelector("button"));
    assert.deepEqual(diagnostics, []);
    app.unmount();
    assert.equal(document.activeElement, trigger);
  } finally {
    if (app && (app as { _container?: Element | null })._container) app.unmount();
    trigger.remove();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
