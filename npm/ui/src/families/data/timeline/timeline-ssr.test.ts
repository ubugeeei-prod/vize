import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import TimelineContent from "./timeline-content.vue";
import TimelineIndicator from "./timeline-indicator.vue";
import TimelineItem from "./timeline-item.vue";
import TimelineRoot from "./timeline-root.vue";
import TimelineTime from "./timeline-time.vue";

const SsrProbe = defineComponent({
  name: "TimelineSsrProbe",
  setup: () => () =>
    h(TimelineRoot, { ariaLabel: "Release history", value: "beta" }, () =>
      ["alpha", "beta", "stable"].map((value) =>
        h(TimelineItem, { key: value, value }, () => [
          h(TimelineIndicator),
          h(TimelineContent, null, () => [
            value,
            h(TimelineTime, { datetime: new Date(Date.UTC(2026, 8, 1)) }, () => "Sep 1"),
          ]),
        ]),
      ),
    ),
});

test("renders byte-identical timelines with progress derived during SSR", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /^<ol/);
  assert.match(html, /aria-label="Release history"/);
  const states = [...html.matchAll(/data-vize-ui="timeline-item"[^>]*data-state="([a-z]+)"/g)].map(
    (match) => match[1],
  );
  assert.deepEqual(states, ["complete", "current", "upcoming"]);
  assert.match(html, /aria-current="step"/);
  assert.match(html, /datetime="2026-09-01T00:00:00.000Z"/);
});

test("hydrates timeline progress without mismatch warnings", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
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
    assert.equal(host.querySelector('[aria-current="step"]')?.getAttribute("data-value"), "beta");
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
