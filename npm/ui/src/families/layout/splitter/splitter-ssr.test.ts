import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import SplitterGroup from "./splitter-group.vue";
import SplitterHandle from "./splitter-handle.vue";
import SplitterPanel from "./splitter-panel.vue";

function probe(groupProps: Record<string, unknown>, sized: boolean) {
  return defineComponent({
    name: "SplitterSsrProbe",
    setup: () => () =>
      h(SplitterGroup, groupProps, () => [
        h(SplitterPanel, sized ? { defaultSize: 25, minSize: 10 } : { minSize: 10 }, () => "Nav"),
        h(SplitterHandle, { ariaLabel: "Resize navigation" }),
        h(SplitterPanel, sized ? { defaultSize: 75 } : {}, () => "Content"),
      ]),
  });
}

test("renders byte-identical splitter markup with exact sizes across SSR requests", async () => {
  const Probe = probe({}, true);
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-vize-ui="splitter-group"/);
  assert.match(html, /flex-grow:25/);
  assert.match(html, /flex-grow:75/);
  assert.match(html, /role="separator"/);
  assert.match(html, /aria-valuenow="25"/);
  assert.match(html, /aria-controls="vize-v-\d+-splitter-panel"/);
});

test("server-renders a controlled layout before every panel registers", async () => {
  const html = await renderToString(createSSRApp(probe({ layout: [40, 60] }, false)));
  assert.match(html, /flex-grow:40/);
  assert.match(html, /flex-grow:60/);
  assert.match(html, /aria-valuenow="40"/);
});

test("hydrates splitter ids and sizes without mismatch warnings", async () => {
  const Probe = probe({}, true);
  const serverHtml = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    assert.equal(host.querySelector("[role='separator']")?.getAttribute("aria-valuenow"), "25");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
