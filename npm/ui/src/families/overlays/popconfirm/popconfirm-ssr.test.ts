import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import PopconfirmCancel from "./popconfirm-cancel.vue";
import PopconfirmConfirm from "./popconfirm-confirm.vue";
import PopconfirmContent from "./popconfirm-content.vue";
import PopconfirmRoot from "./popconfirm-root.vue";
import PopconfirmTrigger from "./popconfirm-trigger.vue";

function createProbe(rootProps: Record<string, unknown>) {
  return defineComponent({
    name: "PopconfirmSsrProbe",
    setup() {
      return () =>
        h(PopconfirmRoot, rootProps, () => [
          h(PopconfirmTrigger, null, () => "Delete"),
          h(
            PopconfirmContent,
            { title: "Delete this file?", description: "This cannot be undone." },
            () => [
              h(PopconfirmCancel, null, () => "Keep"),
              h(PopconfirmConfirm, null, () => "Delete"),
            ],
          ),
        ]);
    },
  });
}

test("renders byte-identical closed popconfirm markup across isolated requests", async () => {
  const Probe = createProbe({});
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-vize-ui="popconfirm-root"/);
  assert.match(html, /data-vize-ui="popconfirm-trigger"/);
  assert.match(html, /aria-expanded="false"/);
  assert.doesNotMatch(html, /Delete this file\?/);
});

test("renders default-open alertdialog markup and hydrates without diagnostics", async () => {
  const Probe = createProbe({ defaultOpen: true, id: "confirm-delete" });
  const serverHtml = await renderToString(createSSRApp(Probe));
  assert.match(serverHtml, /id="confirm-delete-title"/);
  assert.match(serverHtml, /aria-labelledby="confirm-delete-title"/);
  assert.match(serverHtml, /aria-describedby="confirm-delete-description"/);
  assert.match(serverHtml, /role="alertdialog"/);

  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverTrigger = host.querySelector('[data-vize-ui="popconfirm-trigger"]');
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
    await nextTick();
    await nextTick();
    assert.ok(host.querySelector('[data-vize-ui="popconfirm-trigger"]') === serverTrigger);
    assert.deepEqual(diagnostics, []);
    const dialog = document.querySelector('[aria-labelledby="confirm-delete-title"]');
    assert.equal(dialog?.getAttribute("role"), "alertdialog");
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
