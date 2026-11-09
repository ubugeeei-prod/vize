import assert from "node:assert/strict";
import { PassThrough } from "node:stream";
import type { InputEventNapi } from "@vizejs/fresco-native";
import { defineComponent, h, ref, type Ref } from "vue";
import { test, vi } from "vitest";

import { useFocus } from "./composables/useFocus.js";
import type { FrescoRoutedInputEvent } from "./renderer.js";

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
    pollEvent: vi.fn<() => InputEventNapi | null>(() => null),
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

test("interactive input targets the active focus host and can retain Tab ownership", async () => {
  const stdout = new PassThrough();
  Object.assign(stdout, { columns: 80, rows: 24, isTTY: true });
  const calls: string[] = [];
  let firstFocused: Ref<boolean> | undefined;
  let secondFocused: Ref<boolean> | undefined;
  const Root = defineComponent({
    name: "FocusedInputProbe",
    setup() {
      const first = useFocus({ id: "first", autoFocus: true });
      const second = useFocus({ id: "second" });
      firstFocused = first.isFocused;
      secondFocused = second.isFocused;
      return () =>
        h("box", [
          h("input", {
            focusId: first.id,
            onFocus: () => calls.push("focus"),
            onKey: (event: FrescoRoutedInputEvent) => {
              calls.push("key");
              event.preventDefault();
            },
          }),
          h("input", { focusId: second.id }),
        ]);
    },
  });

  nativeFixture.release(nativeFixture.module);
  nativeFixture.module.pollEvent.mockReset();
  nativeFixture.module.pollEvent
    .mockReturnValueOnce({ eventType: "focus", key: "gained" })
    .mockReturnValueOnce({
      eventType: "key",
      key: "tab",
      modifiers: {
        ctrl: false,
        alt: false,
        shift: false,
        meta: false,
        super: false,
        hyper: false,
        capsLock: false,
        numLock: false,
      },
    })
    .mockReturnValue(null);

  const app = createApp(Root, {
    interactive: true,
    maxFps: 1_000,
    patchConsole: false,
    stdout: stdout as unknown as NodeJS.WriteStream,
  });
  await app.mount();
  for (let attempt = 0; calls.length < 2 && attempt < 50; attempt += 1) {
    await new Promise((resolve) => setTimeout(resolve, 2));
  }

  assert.deepEqual(calls, ["focus", "key"]);
  assert.equal(firstFocused?.value, true);
  assert.equal(secondFocused?.value, false);
  await app.unmount();
});
