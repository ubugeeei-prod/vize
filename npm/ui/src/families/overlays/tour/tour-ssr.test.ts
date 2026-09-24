import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import TourArrow from "./tour-arrow.vue";
import TourClose from "./tour-close.vue";
import TourContent from "./tour-content.vue";
import TourDescription from "./tour-description.vue";
import TourNext from "./tour-next.vue";
import TourPrev from "./tour-prev.vue";
import TourProgress from "./tour-progress.vue";
import TourRoot from "./tour-root.vue";
import TourSpotlight from "./tour-spotlight.vue";
import TourStep from "./tour-step.vue";
import TourTitle from "./tour-title.vue";

const steps = [{ value: "welcome" }, { value: "search", target: "#ssr-tour-search" }] as const;

function createProbe(defaultOpen: boolean) {
  return defineComponent({
    name: "TourSsrProbe",
    setup: () => () =>
      h(
        TourRoot,
        {
          steps,
          defaultOpen,
          defaultStep: "search",
          // Hooks only run for client navigation requests, never while rendering.
          beforeEnter: () => new Promise<boolean>(() => undefined),
          messages: { progress: (current: number, total: number) => `${current} / ${total}` },
        },
        () => [
          h(TourSpotlight),
          h(TourContent, { portalDisabled: true }, () => [
            h(TourTitle, null, () => "Search"),
            h(TourDescription, null, () => "Find anything"),
            h(TourStep, { value: "search" }, () => "Search step"),
            h(TourProgress),
            h(TourPrev, null, () => "Back"),
            h(TourNext, null, () => "Next"),
            h(TourClose, null, () => "Close"),
            h(TourArrow),
          ]),
        ],
      ),
  });
}

test("renders byte-identical open tour markup across isolated SSR requests", async () => {
  const Probe = createProbe(true);
  const outputs = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div id="vize-v-\d+-tour"/);
  assert.match(html, /data-vize-ui="tour-root"[^>]*data-state="open"[^>]*data-step="search"/);
  assert.match(html, /data-target="pending"/);
  assert.match(html, /role="dialog"/);
  assert.match(html, /id="vize-v-\d+-tour-content"/);
  assert.match(html, /aria-labelledby="vize-v-\d+-tour-title"/);
  assert.match(html, /aria-describedby="vize-v-\d+-tour-description"/);
  assert.match(html, /data-placement="center"/);
  assert.match(html, /data-vize-ui="tour-step"[^>]*data-value="search"/);
  assert.match(html, /2 \/ 2/);
  assert.match(html, /data-vize-ui="tour-spotlight"/);
  assert.doesNotMatch(html, /--vize-ui-tour-target/);
  assert.doesNotMatch(html, /data-pending/);
});

test("renders a closed tour without dialog content", async () => {
  const html = await renderToString(createSSRApp(createProbe(false)));
  assert.match(html, /data-vize-ui="tour-root"[^>]*data-state="closed"/);
  assert.match(html, /data-vize-ui="tour-content-host"[^>]*hidden/);
  assert.doesNotMatch(html, /role="dialog"/);
});

test("hydrates the open tour without replacement or diagnostics, then resolves targets", async () => {
  const target = document.createElement("input");
  target.id = "ssr-tour-search";
  target.scrollIntoView = () => undefined;
  document.body.append(target);
  const Probe = createProbe(true);
  const serverHtml = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverDialog = host.querySelector("[role='dialog']");
  assert.ok(serverRoot && serverDialog);

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
    assert.ok(host.querySelector("[role='dialog']") === serverDialog);
    assert.deepEqual(diagnostics, []);
    await nextTick();
    await nextTick();
    assert.equal(serverDialog.getAttribute("data-target"), "resolved");
    assert.equal(target.getAttribute("data-vize-tour-target"), "active");
  } finally {
    if (mounted) app.unmount();
    host.remove();
    target.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
