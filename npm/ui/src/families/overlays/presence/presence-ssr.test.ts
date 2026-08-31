import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { usePresence } from "./presence.ts";

const SsrProbe = defineComponent({
  name: "PresenceSsrProbe",
  setup() {
    const present = ref(true);
    const presence = usePresence({ present });
    return () =>
      presence.isPresent.value
        ? h(
            "div",
            {
              "data-vize-presence": presence.status.value,
              "data-present": String(presence.isPresent.value),
            },
            "Overlay",
          )
        : null;
  },
});

test("renders byte-identical present SSR output without DOM access", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], '<div data-vize-presence="present" data-present="true">Overlay</div>');
});

test("hydrates presence without replacement or diagnostics", async () => {
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
    await nextTick();
    assert.equal(host.firstElementChild?.getAttribute("data-vize-presence"), "present");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
