import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, h } from "vue";
import { renderToString } from "vue/server-renderer";

import { createDevtoolsTraceRecorder } from "../runtime.ts";
import DevtoolsTracePanel from "./DevtoolsTracePanel.vue";

test("renders trace trees in SSR without browser globals", async () => {
  const recorder = createDevtoolsTraceRecorder({ now: () => 42, sessionId: "ssr" });
  recorder.record({
    kind: "render:end",
    componentId: "app",
    componentName: "App",
    source: "src/App.vue",
    durationMs: 1,
  });
  recorder.record({
    kind: "reactivity:track",
    sourceId: "ref:count",
    sourceLabel: "count",
    effectId: "effect:app",
    effectLabel: "App render",
    source: "src/App.vue",
  });
  recorder.record({
    kind: "provide:set",
    providerId: "provider:theme",
    key: "theme",
    ownerComponentId: "app",
    ownerComponentName: "App",
    source: "src/App.vue",
  });
  recorder.record({
    kind: "suspense:resolve",
    boundaryId: "boundary:route",
    componentId: "route",
    componentName: "RouteView",
    source: "src/RouteView.vue",
  });

  const html = await renderToString(
    createSSRApp({
      name: "DevtoolsPanelSsrProbe",
      setup: () => () => h(DevtoolsTracePanel, { snapshot: recorder.snapshot() }),
    }),
  );

  assert.match(html, /data-vize-devtools-panel="trace"/);
  assert.match(html, /Render updates/);
  assert.match(html, /Reactive graph/);
  assert.match(html, /Provide tree/);
  assert.match(html, /Suspense tree/);
  assert.match(html, /App render/);
  assert.match(html, /RouteView/);
});

test("renders an SSR-safe empty state", async () => {
  const recorder = createDevtoolsTraceRecorder({ enabled: false, sessionId: "off" });
  const html = await renderToString(
    createSSRApp({
      name: "DevtoolsPanelEmptyProbe",
      setup: () => () => h(DevtoolsTracePanel, { snapshot: recorder.snapshot() }),
    }),
  );

  assert.match(html, /data-vize-devtools-empty/);
  assert.match(html, /No trace events/);
});

test("keeps landmark IDs unique when two trace panels render together", async () => {
  const recorder = createDevtoolsTraceRecorder({ sessionId: "shared" });
  recorder.record({
    kind: "render:end",
    componentId: "app",
    componentName: "App",
    source: "src/App.vue",
    durationMs: 1,
  });
  const snapshot = recorder.snapshot();
  const html = await renderToString(
    createSSRApp({
      setup: () => () =>
        h("div", [h(DevtoolsTracePanel, { snapshot }), h(DevtoolsTracePanel, { snapshot })]),
    }),
  );

  const headingIds = [...html.matchAll(/<h3 id="([^"]+)">/g)].map((match) => match[1]);
  assert.equal(headingIds.length, 8);
  assert.equal(new Set(headingIds).size, headingIds.length);
  for (const id of headingIds) {
    assert.ok(html.includes(`aria-labelledby="${id}"`));
  }
});
