/* eslint-disable script/one-component-per-file -- integration cases need isolated provider roots */
import assert from "node:assert/strict";
import { PassThrough } from "node:stream";
import { defineComponent, h, resolveComponent } from "vue";
import { test } from "vitest";

import { createApp } from "./app.js";
import { defineFrescoPlugin, defineFrescoProvider } from "./plugins.js";

test("installs global components and typed providers before mount", async () => {
  const accent = defineFrescoProvider("test-accent", "white");
  const ExternalLabel = defineComponent({
    name: "ExternalLabel",
    props: { text: { type: String, required: true } },
    setup(props) {
      const color = accent.use();
      return () => h("text", { text: `${props.text}:${color}`, fg: color });
    },
  });
  const plugin = defineFrescoPlugin({
    components: { ExternalLabel },
    setup(app) {
      accent.provide(app, "cyan");
    },
  });
  const Root = defineComponent({
    setup() {
      const label = resolveComponent("ExternalLabel");
      return () => h(label, { text: "plugin" });
    },
  });
  const output: string[] = [];
  const stdout = new PassThrough();
  Object.assign(stdout, { columns: 80, rows: 24, isTTY: false });
  stdout.setEncoding("utf8");
  stdout.on("data", (chunk: string) => output.push(chunk));

  const app = createApp(Root, {
    interactive: false,
    patchConsole: false,
    stdout: stdout as unknown as NodeJS.WriteStream,
  });
  assert.equal(app.use(plugin), app);
  await app.mount();
  await app.unmount();

  assert.equal(output.join(""), "plugin:cyan\n");
  assert.throws(() => app.use(plugin), /before the first successful mount/u);
});

test("retains plugins and accepts more configuration after a failed mount", async () => {
  let attempts = 0;
  let lateInstallations = 0;
  const flaky = defineFrescoPlugin({
    setup() {
      attempts += 1;
      if (attempts === 1) throw new Error("installation failed");
    },
  });
  const late = defineFrescoPlugin({
    setup() {
      lateInstallations += 1;
    },
  });
  const stdout = new PassThrough();
  Object.assign(stdout, { columns: 80, rows: 24, isTTY: false });
  const app = createApp(
    { render: () => h("text", { text: "ready" }) },
    {
      interactive: false,
      patchConsole: false,
      stdout: stdout as unknown as NodeJS.WriteStream,
    },
  ).use(flaky);

  await assert.rejects(app.mount(), /installation failed/u);
  assert.equal(app.use(late), app);
  await app.mount();
  await app.unmount();

  assert.equal(attempts, 2);
  assert.equal(lateInstallations, 1);
});

test("provider fallbacks remain available without a plugin", () => {
  const density = defineFrescoProvider("test-density", "compact");
  const Probe = defineComponent({
    setup() {
      return () => h("text", { text: density.use() });
    },
  });
  const output: string[] = [];
  const stdout = new PassThrough();
  Object.assign(stdout, { columns: 80, rows: 24, isTTY: false });
  stdout.setEncoding("utf8");
  stdout.on("data", (chunk: string) => output.push(chunk));
  const app = createApp(Probe, {
    interactive: false,
    patchConsole: false,
    stdout: stdout as unknown as NodeJS.WriteStream,
  });

  return app
    .mount()
    .then(() => app.unmount())
    .then(() => {
      assert.equal(output.join(""), "compact\n");
    });
});
