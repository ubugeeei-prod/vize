import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useLongPress } from "./long-press.ts";

const SsrProbe = defineComponent({
  name: "LongPressSsrProbe",
  setup() {
    const activations = ref(0);
    const longPress = useLongPress({
      accessibilityDescription: "Hold for actions",
      threshold: 0,
      onLongPress: () => activations.value++,
    });
    return () =>
      h(
        "button",
        {
          ...longPress.longPressProps,
          "data-long-pressed": String(longPress.isLongPressed.value),
          "data-pressed": String(longPress.isPressed.value),
          type: "button",
        },
        `Actions ${activations.value}`,
      );
  },
});

function pointer(type: string): PointerEvent {
  return new PointerEvent(type, {
    bubbles: true,
    button: 0,
    isPrimary: true,
    pointerId: 11,
    pointerType: "touch",
  });
}

test("renders byte-identical long-press SSR output without DOM access", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);

  assert.equal(outputs[0], outputs[1]);
  assert.match(
    outputs[0],
    /^<button aria-description="Hold for actions" data-long-pressed="false" data-pressed="false" type="button">Actions 0<\/button>$/,
  );
  assert.doesNotMatch(outputs[0], /onContextmenu|pointerdown|function/);
});

test("hydrates long-press state and callbacks without replacement or diagnostics", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverButton = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    assert.equal(host.firstElementChild, serverButton);
    const button = host.querySelector("button")!;
    button.dispatchEvent(pointer("pointerdown"));
    await new Promise<void>((resolve) => setTimeout(resolve, 5));
    await nextTick();
    assert.equal(button.dataset.longPressed, "true");
    assert.equal(button.textContent, "Actions 1");
    button.dispatchEvent(pointer("pointerup"));
    await nextTick();
    assert.equal(button.dataset.longPressed, "false");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
