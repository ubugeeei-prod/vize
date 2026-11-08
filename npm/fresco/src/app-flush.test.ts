import assert from "node:assert/strict";
import { PassThrough } from "node:stream";
import { h, ref } from "vue";
import { test } from "vitest";

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
