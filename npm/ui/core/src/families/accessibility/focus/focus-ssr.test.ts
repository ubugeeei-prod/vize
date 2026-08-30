import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { useFocusRing } from "./focus.ts";

const SsrProbe = defineComponent({
  name: "FocusSsrProbe",
  setup() {
    const focus = useFocusRing({ autoFocus: true });
    return () =>
      h(
        "button",
        {
          ...focus.focusProps,
          "data-focus-visible": String(focus.isFocusVisible.value),
          "data-focused": String(focus.isFocused.value),
          type: "button",
        },
        "Focus target",
      );
  },
});

test("renders byte-identical focus output without server DOM access or handlers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(
    outputs[0],
    '<button data-focus-visible="false" data-focused="false" type="button">Focus target</button>',
  );
  assert.doesNotMatch(outputs[0], /onFocus|function/);
});

test("hydrates focus-ring state without replacement or diagnostics", async () => {
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
    const target = host.querySelector("button")!;
    target.focus();
    await nextTick();
    assert.equal(target.dataset.focused, "true");
    assert.equal(target.dataset.focusVisible, "true");
    target.blur();
    await nextTick();
    assert.equal(target.dataset.focused, "false");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
