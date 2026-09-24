import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import ConfirmProvider from "./confirm-provider.vue";
import { useConfirm } from "./confirm.ts";

function probe(requestDuringSetup: boolean) {
  const Consumer = defineComponent({
    name: "ConfirmSsrConsumer",
    setup() {
      const api = useConfirm();
      if (requestDuringSetup) void api.confirm({ title: "Resume draft?" });
      return () => h("main", null, "Application");
    },
  });
  return defineComponent({
    name: "ConfirmSsrProbe",
    setup: () => () => h(ConfirmProvider, null, () => h(Consumer)),
  });
}

test("renders byte-identical idle markup with no dialog layer", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(probe(false))),
    renderToString(createSSRApp(probe(false))),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="confirm-provider"/);
  assert.match(left, /data-state="idle"/);
  assert.match(left, /<main>Application<\/main>/);
  assert.doesNotMatch(left, /role="alertdialog"|data-pending/);
});

test("requests made while rendering on the server never change markup; idle markup hydrates", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(probe(true))),
    renderToString(createSSRApp(probe(true))),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-state="idle"/);
  assert.doesNotMatch(left, /role="alertdialog"/);

  const html = await renderToString(createSSRApp(probe(false)));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(probe(false));
  let mounted = false;
  try {
    const serverRoot = host.firstElementChild;
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
