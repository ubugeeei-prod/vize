import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { TooltipContent, TooltipRoot, TooltipTrigger } from "./tooltip.ts";

const TooltipSsrProbe = defineComponent({
  name: "TooltipSsrProbe",
  setup: () => () =>
    h(TooltipRoot, { defaultOpen: true }, () => [
      h(TooltipTrigger, null, () => "More info"),
      h(TooltipContent, null, () => "Extra context"),
    ]),
});

test("renders deterministic open tooltip markup on the server", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(TooltipSsrProbe)),
    renderToString(createSSRApp(TooltipSsrProbe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /data-vize-ui="tooltip-root"/);
  assert.match(left, /data-vize-ui="tooltip-trigger"/);
  assert.match(left, /aria-describedby="vize-v-\d+-tooltip-content"/);
  assert.match(left, /data-vize-ui="tooltip-content"/);
  assert.match(left, /role="tooltip"/);
  assert.match(left, /data-vize-ui="positioner"/);
  assert.match(left, /data-vize-dismissable-layer/);
  assert.doesNotMatch(left, /pointerenter|focusin|keydown|function/);
});

test("hydrates without replacing the root and then teleports the content", async () => {
  const serverHtml = await renderToString(createSSRApp(TooltipSsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverTrigger = host.querySelector<HTMLButtonElement>("[data-vize-ui='tooltip-trigger']");
  const serverContent = host.querySelector<HTMLDivElement>("[data-vize-ui='tooltip-content']");
  assert.ok(serverRoot);
  assert.ok(serverTrigger);
  assert.ok(serverContent);
  const contentId = serverContent.id;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(TooltipSsrProbe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    await nextTick();
    await nextTick();
    assert.equal(host.firstElementChild, serverRoot);
    assert.deepEqual(diagnostics, []);
    const hydratedTrigger = document.body.querySelector<HTMLButtonElement>(
      "[data-vize-ui='tooltip-trigger']",
    );
    const hydratedContent = document.body.querySelector<HTMLDivElement>(
      "[data-vize-ui='tooltip-content']",
    );
    assert.ok(hydratedTrigger);
    assert.ok(hydratedContent);
    assert.equal(hydratedTrigger.getAttribute("aria-describedby"), contentId);
    assert.equal(hydratedContent.id, contentId);
    assert.equal(hydratedContent.getAttribute("role"), "tooltip");
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
