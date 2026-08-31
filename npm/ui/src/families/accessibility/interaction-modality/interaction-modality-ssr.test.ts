import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick, onMounted, ref } from "vue";
import { renderToString } from "vue/server-renderer";

import { useInteractionModality } from "./interaction-modality.ts";

const SsrProbe = defineComponent({
  name: "InteractionModalitySsrProbe",
  setup() {
    const ownerDocument = ref<Document | null>(null);
    const tracker = useInteractionModality({ document: ownerDocument });
    onMounted(() => {
      ownerDocument.value = document;
    });
    return () =>
      h(
        "output",
        {
          "data-attached": String(tracker.document.value !== null),
          "data-focus-visible": String(tracker.isFocusVisible.value),
        },
        tracker.modality.value ?? "none",
      );
  },
});

test("renders byte-identical output without touching a server document", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);

  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0], /data-attached="false"/);
  assert.match(outputs[0], /data-focus-visible="false"/);
  assert.match(outputs[0], />none<\/output>/);
});

test("attaches after hydration without mismatch diagnostics", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);

  try {
    app.mount(host);
    await nextTick();
    assert.equal(host.querySelector("output")?.dataset.attached, "true");
    assert.equal(host.querySelector("output")?.textContent, "none");
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
