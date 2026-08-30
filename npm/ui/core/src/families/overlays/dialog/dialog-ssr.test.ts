import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  DialogTrigger,
} from "./dialog.ts";

const DialogSsrProbe = defineComponent({
  name: "DialogSsrProbe",
  setup: () => () =>
    h(DialogRoot, { defaultOpen: true }, () => [
      h(DialogTrigger, null, () => "Open"),
      h(DialogPortal, null, () => [
        h(DialogOverlay),
        h(DialogContent, null, () => [
          h(DialogTitle, null, () => "Preferences"),
          h(DialogDescription, null, () => "Tune the workspace."),
          h(DialogClose, null, () => "Done"),
        ]),
      ]),
    ]),
});

test("renders deterministic in-place modal markup on the server", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(DialogSsrProbe)),
    renderToString(createSSRApp(DialogSsrProbe)),
  ]);

  assert.equal(left, right);
  assert.match(left, /data-vize-ui="dialog-root"/);
  assert.match(left, /data-vize-ui="dialog-portal"/);
  assert.match(left, /data-vize-ui="dialog-overlay"/);
  assert.match(left, /data-vize-ui="dialog-content"/);
  assert.match(left, /aria-haspopup="dialog"/);
  assert.match(left, /aria-expanded="true"/);
  assert.match(left, /aria-modal="true"/);
  assert.match(left, /id="vize-v-\d+-dialog-content"/);
  assert.match(left, /aria-labelledby="vize-v-\d+-dialog-title"/);
  assert.match(left, /data-vize-dismissable-layer/);
  assert.doesNotMatch(left, /data-vize-scroll-locked|pointerdown|focusin|keydown|function/);
});

test("hydrates without replacing the root and then teleports the layer", async () => {
  const serverHtml = await renderToString(createSSRApp(DialogSsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(DialogSsrProbe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    await nextTick();
    await nextTick();
    assert.equal(host.firstElementChild, serverRoot);
    assert.deepEqual(diagnostics, []);
    const content = document.body.querySelector('[data-vize-ui="dialog-content"]');
    const portal = document.body.querySelector('[data-vize-ui="portal"]');
    assert.ok(content instanceof HTMLElement);
    assert.ok(portal instanceof HTMLElement);
    assert.equal(portal.parentElement, document.body);
    assert.equal(content.getAttribute("aria-modal"), "true");
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
