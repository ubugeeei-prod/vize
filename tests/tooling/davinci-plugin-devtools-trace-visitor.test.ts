import assert from "node:assert/strict";
import { test } from "node:test";
import { createDevtoolsTraceRecorder } from "../../npm/devtools/src/index.ts";
import {
  VISITOR_API_VERSION,
  VisitorPluginValidationError,
  createDevtoolsTraceVisitorHost,
  defineVisitorPlugin,
  type VisitorEvent,
} from "../../npm/plugin-sdk/visitor.js";

test("the production recorder delivers immutable normalized traces to visitors", () => {
  const visited: VisitorEvent<"devtools-trace">[] = [];
  const plugin = defineVisitorPlugin({
    apiVersion: VISITOR_API_VERSION,
    name: "trace-recorder",
    version: "1",
    capabilities: ["devtools-trace"] as const,
    cacheInputs: [],
    visitors: {
      "devtools-trace"(event) {
        visited.push(event);
      },
    },
  });
  const recorder = createDevtoolsTraceRecorder({
    now: () => 42,
    sessionId: "request-a",
    visitorHosts: [createDevtoolsTraceVisitorHost(plugin)],
  });

  recorder.record({
    kind: "render:end",
    componentId: "app",
    componentName: "App",
    source: "src/App.vue",
    durationMs: 1.25,
  });
  recorder.record({
    kind: "reactivity:track",
    effectId: "render-app",
    effectLabel: "App render",
    sourceId: "count",
    sourceLabel: "count",
  });

  assert.equal(visited.length, 2);
  assert.deepEqual(visited[0], {
    apiVersion: 1,
    capability: "devtools-trace",
    file: "src/App.vue",
    payload: {
      events: [
        {
          id: "request-a:1",
          parentId: null,
          name: "render:end",
          stage: "render",
          startNs: 42_000_000,
          durationNs: 1_250_000,
          data: {
            componentId: "app",
            componentName: "App",
            source: "src/App.vue",
          },
        },
      ],
    },
  });
  assert.equal(visited[1]?.file, "vize-devtools://request-a");
  assert.equal(visited[1]?.payload.events[0]?.id, "request-a:2");
  assert.equal(visited[1]?.payload.events[0]?.stage, "reactivity");
  assert.equal(Object.isFrozen(visited[0]), true);
  assert.equal(Object.isFrozen(visited[0]?.payload.events[0]?.data), true);
  assert.deepEqual(
    recorder.snapshot().events.map(({ sequence, kind }) => [sequence, kind]),
    [
      [1, "render:end"],
      [2, "reactivity:track"],
    ],
  );
});

test("disabled recorders never execute configured visitors", () => {
  let runs = 0;
  const recorder = createDevtoolsTraceRecorder({
    enabled: false,
    visitorHosts: [
      {
        name: "disabled-observer",
        version: "1",
        fingerprint: "disabled-observer-v1",
        run() {
          runs += 1;
          return "";
        },
      },
    ],
  });

  recorder.record({
    kind: "render:start",
    componentId: "app",
    componentName: "App",
    source: "src/App.vue",
  });
  assert.equal(runs, 0);
  assert.deepEqual(recorder.snapshot().events, []);
});

test("devtools visitor hosts fail closed before committing recorder state", () => {
  assert.throws(
    () =>
      createDevtoolsTraceRecorder({
        visitorHosts: [{ name: "", version: "1", fingerprint: "x", run: () => "" }],
      }),
    /VIZE_DEVTOOLS_INVALID_VISITOR_HOST/u,
  );

  const recorder = createDevtoolsTraceRecorder({
    visitorHosts: [
      {
        name: "invalid-result",
        version: "1",
        fingerprint: "invalid-result-v1",
        run: () => "changed",
      },
    ],
  });
  assert.throws(
    () =>
      recorder.record({
        kind: "render:start",
        componentId: "app",
        componentName: "App",
        source: "src/App.vue",
      }),
    /VIZE_DEVTOOLS_INVALID_VISITOR_RESULT/u,
  );
  assert.deepEqual(recorder.snapshot().events, []);
});

test("the SDK adapter rejects malformed deliveries and mixed capabilities", () => {
  const plugin = defineVisitorPlugin({
    apiVersion: VISITOR_API_VERSION,
    name: "trace-only",
    version: "1",
    capabilities: ["devtools-trace"] as const,
    cacheInputs: [],
    visitors: { "devtools-trace"() {} },
  });
  const host = createDevtoolsTraceVisitorHost(plugin);
  assert.throws(
    () => host.run("not json"),
    (error: unknown) =>
      error instanceof VisitorPluginValidationError &&
      error.code === "invalid-event" &&
      error.message.endsWith("delivery: must be valid JSON"),
  );
  assert.throws(
    () =>
      host.run(
        JSON.stringify({
          schemaVersion: 1,
          sessionId: "request-a",
          event: { sequence: 0, timestamp: 1, kind: "render:start" },
        }),
      ),
    /delivery\.event\.sequence: must be a positive integer$/u,
  );

  const mixed = defineVisitorPlugin({
    apiVersion: VISITOR_API_VERSION,
    name: "mixed",
    version: "1",
    capabilities: ["devtools-trace", "sfc"] as const,
    cacheInputs: [],
    visitors: { "devtools-trace"() {}, sfc() {} },
  });
  assert.throws(
    () => createDevtoolsTraceVisitorHost(mixed as never),
    /host\.capabilities: missing "sfc"$/u,
  );
});
