import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import MasterDetail from "./master-detail.vue";

function probe(ssrWidth: number | undefined) {
  return defineComponent({
    name: "MasterDetailSsrProbe",
    setup: () => () =>
      h(
        MasterDetail,
        { ssrWidth, defaultSelected: "inbox", masterLabel: "Mailboxes", detailLabel: "Messages" },
        {
          master: () => h("ul", [h("li", "inbox")]),
          detail: ({ selected }: { selected: string }) => h("h2", selected),
        },
      ),
  });
}

test("renders byte-identical split or stacked markup from ssrWidth", async () => {
  const wide = probe(1200);
  const outputs = await Promise.all([
    renderToString(createSSRApp(wide)),
    renderToString(createSSRApp(wide)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0] ?? "", /data-layout="split"/);
  assert.match(outputs[0] ?? "", /data-part="master"[^]*data-part="detail"/);

  const narrow = await renderToString(createSSRApp(probe(undefined)));
  assert.match(narrow, /data-layout="stacked"/);
  assert.doesNotMatch(narrow, /data-part="master"/);
  assert.match(narrow, /<h2>inbox<\/h2>/);
});

test("hydrates the ssrWidth layout without mismatch warnings", async () => {
  window.happyDOM.setViewport({ width: 1200, height: 800 });
  const Probe = probe(1200);
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    await nextTick();
    assert.equal(
      host.querySelector('[data-vize-ui="master-detail"]')?.getAttribute("data-layout"),
      "split",
    );
  } finally {
    app.unmount();
    console.warn = originalWarn;
    console.error = originalError;
    host.remove();
    window.happyDOM.setViewport({ width: 1024, height: 768 });
  }
  assert.deepEqual(diagnostics, []);
});
