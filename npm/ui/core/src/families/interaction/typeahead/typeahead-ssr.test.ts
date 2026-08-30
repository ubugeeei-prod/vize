import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { createCollectionRegistry } from "../../../collection.ts";
import { useTypeahead } from "./typeahead.ts";

const SsrProbe = defineComponent({
  name: "TypeaheadSsrProbe",
  setup() {
    const registry = createCollectionRegistry<string, string>();
    registry.register({ key: "alpha", value: "Alpha", textValue: "Alpha" });
    registry.register({ key: "bravo", value: "Bravo", textValue: "Bravo" });
    const typeahead = useTypeahead({ registry });
    return () =>
      h(
        "div",
        {
          ...typeahead.typeaheadProps,
          "data-active": registry.activeKey.value ?? "none",
          "data-query": typeahead.query.value,
          tabindex: 0,
        },
        "Typeahead target",
      );
  },
});

test("renders byte-identical typeahead markup without timers or handlers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<div data-active="none" data-query tabindex="0">Typeahead target</div>',
  );
  assert.doesNotMatch(outputs[0], /keydown|function/);
});

test("hydrates buffered matching without replacing the server host", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverTarget = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverTarget);
    const target = host.firstElementChild as HTMLElement;
    target.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "b" }));
    await nextTick();
    assert.equal(target.dataset.query, "b");
    assert.equal(target.dataset.active, "bravo");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
