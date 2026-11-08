import assert from "node:assert/strict";
import { PassThrough } from "node:stream";
import { h } from "vue";
import { test } from "vitest";

import { createApp } from "../../src/app.js";
import App from "./App.vue";
import ExternalMeter from "./ExternalMeter.vue";
import { externalMeterPlugin } from "./plugin.js";

function outputStream(output: string[]): NodeJS.WriteStream {
  const stdout = new PassThrough();
  Object.assign(stdout, { columns: 80, rows: 24, isTTY: false });
  stdout.setEncoding("utf8");
  stdout.on("data", (chunk: string) => output.push(chunk));
  return stdout as unknown as NodeJS.WriteStream;
}

test("the external plugin renders its registered SFC and provider value", async () => {
  const output: string[] = [];
  const app = createApp(App, {
    interactive: false,
    patchConsole: false,
    stdout: outputStream(output),
  }).use(externalMeterPlugin);

  await app.mount();
  await app.unmount();

  assert.equal(output.join(""), `External provider\n${"■".repeat(17)}${"·".repeat(7)}\n`);
});

test("the external meter clamps invalid width and progress inputs", async () => {
  const output: string[] = [];
  const app = createApp(
    { render: () => h(ExternalMeter, { value: Number.NaN, width: -4 }) },
    {
      interactive: false,
      patchConsole: false,
      stdout: outputStream(output),
    },
  );

  await app.mount();
  await app.unmount();

  assert.equal(output.join(""), "");
});
