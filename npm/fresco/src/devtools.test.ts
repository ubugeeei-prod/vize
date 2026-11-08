import assert from "node:assert/strict";
import { PassThrough } from "node:stream";
import { test } from "vitest";
import { h } from "vue";

import { createApp } from "./app.js";
import { createFrescoDevtools, formatFrescoDebugOverlay, inspectFrescoTree } from "./devtools.js";
import { firstChild, mountFresco } from "./testing/mount.js";

test("joins stable host nodes with native layout rectangles", () => {
  const mounted = mountFresco(() =>
    h("box", { style: { flexDirection: "column" } }, [
      h("text", { text: "title" }),
      h("text", { text: "body" }),
    ]),
  );
  const root = firstChild(mounted);
  const [title] = root.children;
  assert.ok(title);

  const inspected = inspectFrescoTree(root, [
    { id: root.id, x: 0, y: 0, width: 20, height: 2 },
    { id: title.id, x: 0, y: 0, width: 5, height: 1 },
    { id: 999_999, x: 0, y: 0, width: 1, height: 1 },
  ]);

  assert.equal(inspected.rootId, root.id);
  assert.equal(inspected.nodeCount, 3);
  assert.equal(inspected.layoutCount, 2);
  assert.deepEqual(inspected.unmatchedLayoutIds, [999_999]);
  assert.deepEqual(inspected.nodes[0], {
    id: root.id,
    type: "box",
    parentId: -1,
    childIds: root.children.map((child) => child.id),
    depth: 0,
    props: { style: { flexDirection: "column" } },
    layout: { x: 0, y: 0, width: 20, height: 2 },
  });
  assert.equal(inspected.nodes[1]?.parentId, root.id);
  assert.equal(inspected.nodes[1]?.depth, 1);
  mounted.unmount();
});

test("retains a bounded lifecycle, input, and render trace", () => {
  const devtools = createFrescoDevtools({ eventLimit: 4 });
  const mounted = mountFresco(() => h("text", { text: "ready" }), {
    lifecycle: devtools.lifecycle,
  });

  devtools.recordInput({
    type: "key",
    key: "enter",
    ctrl: false,
    alt: false,
    shift: false,
    meta: false,
    super: false,
    hyper: false,
    capsLock: false,
    numLock: false,
    eventType: "press",
  });
  devtools.recordRender(mounted.root, [], {
    renderTime: 1.25,
    nodeCount: 2,
    layoutCount: 0,
  });

  const snapshot = devtools.snapshot();
  assert.equal(snapshot.events.length, 4);
  assert.deepEqual(
    snapshot.events.map((event) => event.sequence),
    snapshot.events.map((event) => event.sequence).sort((left, right) => left - right),
  );
  assert.equal(snapshot.events.at(-2)?.type, "input");
  assert.equal(snapshot.events.at(-1)?.type, "render");
  assert.deepEqual(snapshot.latestRender, {
    renderTime: 1.25,
    nodeCount: 2,
    layoutCount: 0,
  });

  devtools.clearEvents();
  assert.deepEqual(devtools.snapshot().events, []);
  assert.ok(devtools.snapshot().tree, "clearing the trace preserves the inspected tree");
  mounted.unmount();
});

test("formats tree, layout, metrics, and recent events as a terminal overlay", () => {
  const devtools = createFrescoDevtools();
  const mounted = mountFresco(() => h("text", { text: "hello" }), {
    lifecycle: devtools.lifecycle,
  });
  const child = firstChild(mounted);
  devtools.recordRender(mounted.root, [{ id: child.id, x: 2, y: 3, width: 5, height: 1 }], {
    renderTime: 0.5,
    nodeCount: 2,
    layoutCount: 1,
  });

  const overlay = formatFrescoDebugOverlay(devtools.snapshot());
  assert.match(overlay, /Fresco 2 nodes · 1 layouts · 0\.50ms/u);
  assert.match(overlay, new RegExp(`#${child.id} text 5x1@2,3`, "u"));
  assert.match(overlay, /events/u);
  assert.match(overlay, /render 0\.50ms/u);
  mounted.unmount();
});

test("connects application lifecycle and render metrics through public options", async () => {
  const devtools = createFrescoDevtools();
  const renderedMetrics: Array<{ nodeCount: number; layoutCount: number }> = [];
  const stdout = new PassThrough();
  Object.assign(stdout, { columns: 80, rows: 24, isTTY: false });
  const app = createApp(() => h("text", { text: "profiled" }), {
    devtools,
    interactive: false,
    patchConsole: false,
    stdout: stdout as unknown as NodeJS.WriteStream,
    onRender: ({ nodeCount, layoutCount }) => renderedMetrics.push({ nodeCount, layoutCount }),
  });

  await app.mount();
  app.render();

  const snapshot = devtools.snapshot();
  assert.equal(snapshot.tree?.nodeCount, 2);
  assert.equal(snapshot.tree?.nodes[1]?.props.text, "profiled");
  assert.deepEqual(renderedMetrics.at(-1), { nodeCount: 2, layoutCount: 0 });
  assert.ok(snapshot.events.some((event) => event.type === "create"));
  assert.ok(snapshot.events.some((event) => event.type === "render"));
  await app.unmount();
});
