import assert from "node:assert/strict";
import { PassThrough } from "node:stream";
import { h, ref } from "vue";
import { test, vi } from "vitest";

const nativeFixture = vi.hoisted(() => {
  let release: (native: object) => void = () => {};
  const ready = new Promise<object>((resolve) => {
    release = resolve;
  });
  const renderTree = vi.fn();
  const module = {
    disableIme: vi.fn(),
    enableIme: vi.fn(),
    flushTerminal: vi.fn(),
    getLastRenderLayouts: vi.fn(() => []),
    getTerminalInfo: vi.fn(() => ({ height: 24, width: 80 })),
    initLayout: vi.fn(),
    initTerminalWithOptions: vi.fn(),
    pollEvent: vi.fn(() => null),
    renderTree,
    restoreTerminal: vi.fn(),
  };
  return { module, ready, release, renderTree };
});

vi.mock("@vizejs/fresco-native", () => nativeFixture.ready);

import { createApp } from "./app.js";

test("waitUntilRenderFlush commits pending Vue updates without waiting for the frame loop", async () => {
  const value = ref("first");
  const renders: string[] = [];
  const stdout = new PassThrough();
  Object.assign(stdout, { columns: 80, rows: 24, isTTY: false });
  const app = createApp(() => h("text", { text: value.value }), {
    interactive: false,
    maxFps: 1,
    patchConsole: false,
    stdout: stdout as unknown as NodeJS.WriteStream,
    onRender: () => renders.push(value.value),
  });

  await app.mount();
  await app.waitUntilRenderFlush();
  const initialRenderCount = renders.length;

  value.value = "second";
  await app.waitUntilRenderFlush();

  assert.equal(renders.length, initialRenderCount + 1);
  assert.equal(renders.at(-1), "second");
  await app.unmount();
});

test("waitUntilRenderFlush waits for an interactive mount before resolving", async () => {
  const stdout = new PassThrough();
  Object.assign(stdout, { columns: 80, rows: 24, isTTY: true });
  const app = createApp(() => h("text", { text: "mounted" }), {
    interactive: true,
    patchConsole: false,
    stdout: stdout as unknown as NodeJS.WriteStream,
  });

  const mountPromise = app.mount();
  assert.equal(app.mount(), mountPromise, "concurrent callers share the pending mount");
  let flushSettled = false;
  const flushPromise = app.waitUntilRenderFlush().then(() => {
    flushSettled = true;
  });
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(flushSettled, false, "the flush must not outrun native loading");

  nativeFixture.release(nativeFixture.module);
  await Promise.all([mountPromise, flushPromise]);
  assert.equal(nativeFixture.renderTree.mock.calls.length, 1);
  await app.unmount();
});
