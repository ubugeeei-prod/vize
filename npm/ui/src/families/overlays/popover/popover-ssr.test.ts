import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { PopoverArrow, PopoverContent, PopoverRoot, PopoverTrigger } from "./popover.ts";

const PopoverSsrProbe = defineComponent({
  name: "PopoverSsrProbe",
  setup: () => () =>
    h(PopoverRoot, { defaultOpen: true }, () => [
      h(PopoverTrigger, null, () => "Filters"),
      h(PopoverContent, { placement: "bottom-start" }, () => [
        h("button", { type: "button" }, "Today"),
        h(PopoverArrow, null, () => h("span", "Arrow")),
      ]),
    ]),
});

test("renders deterministic open popover markup on the server", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(PopoverSsrProbe)),
    renderToString(createSSRApp(PopoverSsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="popover-root"/);
  assert.match(left, /data-vize-ui="popover-trigger"/);
  assert.match(left, /aria-haspopup="dialog"/);
  assert.match(left, /aria-expanded="true"/);
  assert.match(left, /aria-controls="vize-v-\d+-popover-content"/);
  assert.match(left, /data-vize-ui="popover-content"/);
  assert.match(left, /role="dialog"/);
  assert.match(left, /data-side="bottom"/);
  assert.match(left, /data-align="start"/);
  assert.match(left, /data-vize-ui="popover-arrow"/);
  assert.match(left, /data-vize-ui="positioner"/);
  assert.match(left, /data-vize-dismissable-layer/);
  assert.doesNotMatch(left, /data-vize-scroll-locked|pointerdown|focusin|keydown|function/);
});

test("hydrates without replacing the root and then teleports the content", async () => {
  const serverHtml = await renderToString(createSSRApp(PopoverSsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverTrigger = host.querySelector<HTMLButtonElement>("[data-vize-ui='popover-trigger']");
  const serverContent = host.querySelector<HTMLDivElement>("[data-vize-ui='popover-content']");
  assert.ok(serverRoot);
  assert.ok(serverTrigger);
  assert.ok(serverContent);
  const contentId = serverContent.id;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(PopoverSsrProbe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    await nextTick();
    await nextTick();
    assert.equal(host.firstElementChild, serverRoot);
    assert.deepEqual(diagnostics, []);
    const hydratedTrigger = document.body.querySelector<HTMLButtonElement>(
      "[data-vize-ui='popover-trigger']",
    );
    const hydratedContent = document.body.querySelector<HTMLDivElement>(
      "[data-vize-ui='popover-content']",
    );
    assert.ok(hydratedTrigger);
    assert.ok(hydratedContent);
    assert.equal(hydratedTrigger.getAttribute("aria-controls"), contentId);
    assert.equal(hydratedContent.id, contentId);
    assert.equal(hydratedContent.getAttribute("role"), "dialog");
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
