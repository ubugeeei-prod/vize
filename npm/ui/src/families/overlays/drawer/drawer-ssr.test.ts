import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import DrawerContent from "./drawer-content.vue";
import DrawerHandle from "./drawer-handle.vue";
import DrawerRoot from "./drawer-root.vue";
import { DrawerDescription, DrawerTitle, DrawerTrigger } from "./drawer.ts";

function probe(defaultOpen: boolean) {
  return defineComponent({
    name: "DrawerSsrProbe",
    setup: () => () =>
      h(DrawerRoot, { defaultOpen, snapPoints: [0.5, 1], side: "right" }, () => [
        h(DrawerTrigger, null, () => "Open filters"),
        h(DrawerContent, null, () => [
          h(DrawerHandle),
          h(DrawerTitle, null, () => "Filters"),
          h(DrawerDescription, null, () => "Narrow the results."),
        ]),
      ]),
  });
}

test("renders byte-identical closed drawer markup without opening the native dialog", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(probe(false))),
    renderToString(createSSRApp(probe(false))),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="drawer-root"/);
  assert.match(left, /<dialog id="vize-v-\d+-drawer-content"/);
  assert.match(left, /data-side="right"/);
  assert.match(left, /data-snap-point="0.5"/);
  assert.match(left, /aria-controls="vize-v-\d+-drawer-content"/);
  assert.doesNotMatch(left, /<dialog[^>]*\sopen[\s>=]/);
  assert.doesNotMatch(
    left,
    /data-vize-ui="dialog-title"/,
    "closed drawers do not render slot contents",
  );
  assert.doesNotMatch(left, /--vize-drawer/);
});

test("renders open drawers deterministically and hydrates before calling showModal", async () => {
  const Probe = probe(true);
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /aria-labelledby="vize-v-\d+-drawer-title"/);
  assert.match(left, /data-vize-ui="dialog-title"/);
  assert.doesNotMatch(left, /<dialog[^>]*\sopen[\s>=]/);

  const host = document.createElement("div");
  host.innerHTML = left;
  document.body.append(host);
  const serverDialog = host.querySelector("dialog");
  assert.ok(serverDialog);
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
    const dialog = host.querySelector("dialog");
    assert.ok(dialog === serverDialog, "hydration reuses the server dialog");
    assert.equal(dialog.open, true);
    assert.equal(dialog.style.getPropertyValue("--vize-drawer-snap-offset"), "0px");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
