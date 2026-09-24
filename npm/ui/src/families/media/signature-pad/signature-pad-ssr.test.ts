import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import SignaturePadCanvas from "./signature-pad-canvas.vue";
import SignaturePadClear from "./signature-pad-clear.vue";
import SignaturePadGuide from "./signature-pad-guide.vue";
import SignaturePadRoot from "./signature-pad-root.vue";
import SignaturePadUndo from "./signature-pad-undo.vue";

const strokes = [
  {
    points: [
      { x: 10, y: 10, pressure: 0.5, time: 0 },
      { x: 60, y: 40, pressure: 0.8, time: 16 },
    ],
  },
];

const SsrProbe = defineComponent({
  name: "SignaturePadSsrProbe",
  setup: () => () =>
    h(SignaturePadRoot, { defaultValue: strokes, name: "signature" }, () => [
      h(SignaturePadGuide, null, () => "Sign above"),
      h(SignaturePadCanvas),
      h(SignaturePadUndo, null, () => "Undo"),
      h(SignaturePadClear, null, () => "Clear"),
    ]),
});

test("renders byte-identical signature markup across isolated SSR requests", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /^<div id="vize-v-\d+-signature-pad" data-vize-ui="signature-pad-root"/);
  assert.match(left, /data-state="filled"/);
  assert.match(
    left,
    /<svg xmlns="http:\/\/www\.w3\.org\/2000\/svg" role="img" aria-label="Signature" viewBox="0 0 400 200"/,
  );
  assert.match(left, /<path d="M [^"]+Z" fill="currentColor" data-vize-ui="signature-pad-stroke"/);
  assert.match(left, /<input type="hidden" name="signature" value="\[\{&quot;points&quot;/);
  assert.doesNotMatch(left, /touch-action/);
});

test("hydrates signature markup without warnings or node replacement", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverPath = host.querySelector("path");
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    assert.ok(host.querySelector("path") === serverPath);
    assert.equal(host.querySelector("svg")?.style.touchAction, "none");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
