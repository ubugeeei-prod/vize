import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { usePress } from "./press.ts";

const SsrProbe = defineComponent({
  name: "PressSsrProbe",
  setup() {
    const presses: string[] = [];
    const press = usePress({ onPress: (event) => presses.push(event.pointerType) });
    return () =>
      h(
        "button",
        {
          ...press.pressProps,
          "data-pressed": String(press.isPressed.value),
          type: "button",
        },
        `Activate ${presses.length}`,
      );
  },
});

test("renders byte-identical SSR output without DOM access or serialized handlers", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);

  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0], /^<button data-pressed="false" type="button">Activate 0<\/button>$/);
  assert.doesNotMatch(outputs[0], /onClick|pointerdown|function/);
});

test("hydrates without diagnostics and activates through the same bound props", async () => {
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
    host
      .querySelector("button")
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true, detail: 0 }));
    await nextTick();
    assert.equal(host.querySelector("button")?.textContent, "Activate 1");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
