import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  createDevtoolsManifest,
  createDevtoolsTraceRecorder,
  hydrateDevtoolsSnapshot,
  serializeDevtoolsSnapshot,
  streamDevtoolsTrace,
} from "./index.ts";

test("records render updates without sharing request state", () => {
  const first = createDevtoolsTraceRecorder({ now: () => 10, sessionId: "request-a" });
  const second = createDevtoolsTraceRecorder({ now: () => 20, sessionId: "request-b" });

  first.record({
    kind: "render:end",
    componentId: "article",
    componentName: "ArticleView",
    source: "src/pages/ArticleView.vue",
    durationMs: 2.5,
  });
  second.record({
    kind: "update:scheduled",
    componentId: "user",
    componentName: "UserCard",
    source: "src/components/UserCard.vue",
    reason: "props",
  });

  assert.deepEqual(
    first.snapshot().renderTree.map((node) => node.componentName),
    ["ArticleView"],
  );
  assert.deepEqual(
    second.snapshot().renderTree.map((node) => node.componentName),
    ["UserCard"],
  );
});

test("derives reactive graph, provide tree, and suspense tree snapshots", async () => {
  const recorder = createDevtoolsTraceRecorder({ now: () => 100, sessionId: "trace" });
  recorder.record({
    kind: "reactivity:track",
    sourceId: "ref:count",
    sourceLabel: "count",
    effectId: "effect:render",
    effectLabel: "render",
    source: "src/App.vue",
  });
  recorder.record({
    kind: "reactivity:trigger",
    sourceId: "ref:count",
    effectId: "effect:render",
    operation: "set",
    source: "src/App.vue",
  });
  recorder.record({
    kind: "provide:set",
    providerId: "provider:theme",
    key: "theme",
    ownerComponentId: "layout",
    ownerComponentName: "LayoutShell",
    source: "src/LayoutShell.vue",
  });
  recorder.record({
    kind: "inject:resolve",
    injectorId: "inject:button",
    injectorComponentId: "button",
    injectorComponentName: "ThemeButton",
    key: "theme",
    providerId: "provider:theme",
    status: "resolved",
    source: "src/ThemeButton.vue",
  });
  recorder.record({
    kind: "suspense:pending",
    boundaryId: "suspense:route",
    componentId: "route",
    componentName: "RouteView",
    asyncDependency: "loadRoute",
    source: "src/RouteView.vue",
  });

  const snapshot = serializeDevtoolsSnapshot(recorder);
  assert.equal(snapshot.reactiveGraph.edges.length, 2);
  assert.equal(snapshot.provideTree[0]?.consumers[0]?.injectorComponentName, "ThemeButton");
  assert.equal(snapshot.suspenseTree[0]?.state, "pending");

  const streamed = [];
  for await (const event of streamDevtoolsTrace(snapshot)) streamed.push(event.kind);
  assert.deepEqual(streamed, [
    "reactivity:track",
    "reactivity:trigger",
    "provide:set",
    "inject:resolve",
    "suspense:pending",
  ]);

  const hydrated = hydrateDevtoolsSnapshot(snapshot, { now: () => 200 });
  hydrated.record({
    kind: "suspense:resolve",
    boundaryId: "suspense:route",
    componentId: "route",
    componentName: "RouteView",
    source: "src/RouteView.vue",
  });
  assert.equal(hydrated.snapshot().suspenseTree[0]?.state, "resolved");
});

test("disabled recorders stay empty and source validation fails closed", () => {
  const disabled = createDevtoolsTraceRecorder({ enabled: false });
  disabled.record({
    kind: "render:start",
    componentId: "app",
    componentName: "App",
    source: "src/App.vue",
  });
  assert.equal(disabled.snapshot().events.length, 0);

  assert.throws(
    () =>
      createDevtoolsTraceRecorder().record({
        kind: "render:start",
        componentId: "broken",
        componentName: "Broken",
        // @ts-expect-error Runtime diagnostics still protect untyped callers.
        source: "src/Broken.ts",
      }),
    /VIZE_DEVTOOLS_SOURCE_NOT_VUE/,
  );
});

test("emits generator manifest metadata for Vue-authored panels", () => {
  assert.deepEqual(createDevtoolsManifest(), {
    schemaVersion: 1,
    panels: [
      {
        id: "trace",
        title: "Trace",
        source: "src/panel/DevtoolsTracePanel.vue",
        surfaces: ["render", "reactivity", "provide", "suspense"],
      },
    ],
  });
  assert.throws(
    () =>
      createDevtoolsManifest([
        {
          id: "broken",
          title: "Broken",
          // @ts-expect-error Runtime diagnostics still protect untyped callers.
          source: "src/panel/Broken.ts",
          surfaces: ["render"],
        },
      ]),
    /VIZE_DEVTOOLS_PANEL_NOT_VUE/,
  );
});
