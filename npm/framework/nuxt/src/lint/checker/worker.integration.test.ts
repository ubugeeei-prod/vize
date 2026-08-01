import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { createRequire } from "node:module";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { NuxtLintCheckerWorker, type NuxtLintCheckerTask } from "./worker.ts";
import type { createNuxtLintCheckerVitePlugin as CreateVitePlugin } from "./vite.ts";

const require = createRequire(import.meta.url);
const here = path.dirname(fileURLToPath(import.meta.url));

void test("real worker re-reads its oxlint + Patina config on every pass", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-nuxt-real-checker-"));
  const worker = new NuxtLintCheckerWorker();
  t.after(async () => {
    await worker.close();
    await rm(root, { force: true, recursive: true });
  });

  const configFile = path.join(root, "oxlint.config.json");
  const source = path.join(root, "App.vue");
  const pluginEntry = fileURLToPath(import.meta.resolve("oxlint-plugin-vize"));
  const pluginSpecifier = path.relative(root, pluginEntry).replaceAll(path.sep, "/");
  const oxlintManifest = require.resolve("oxlint/package.json");
  const oxlintEntrypoint = path.join(path.dirname(oxlintManifest), "bin", "oxlint");
  await writeFile(
    source,
    `<script setup>\nconst items = [1]\n</script>\n<template>\n  <div v-for="item in items">{{ item }}</div>\n</template>\n`,
  );

  const task: NuxtLintCheckerTask = {
    configFile,
    cwd: root,
    emitError: true,
    emitWarning: true,
    exclude: [],
    fix: false,
    formatter: "json",
    oxlintEntrypoint,
    targets: [path.join(root, "**/*.{js,jsx,ts,tsx,vue}")],
  };

  await writeConfig("error");
  const failing = await worker.run(task);
  assert.equal(failing.hasErrors, true);
  assert.equal(failing.diagnosticCount, 1);
  assert.deepEqual(
    (JSON.parse(failing.output) as { diagnostics: Array<{ code: string }> }).diagnostics.map(
      ({ code }) => code,
    ),
    ["vize(vue/require-v-for-key)"],
  );

  await writeConfig("off");
  assert.deepEqual(await worker.run(task), {
    diagnosticCount: 0,
    hasErrors: false,
    hasWarnings: false,
    output: "",
  });
  assert.match(await readFile(configFile, "utf8"), /"off"/u);

  async function writeConfig(severity: "error" | "off"): Promise<void> {
    await writeFile(
      configFile,
      `${JSON.stringify(
        {
          jsPlugins: [{ name: "vize", specifier: pluginSpecifier }],
          plugins: ["vue"],
          rules: { "vize/vue/require-v-for-key": severity },
          settings: { vize: { preset: "incremental" } },
        },
        null,
        2,
      )}\n`,
    );
  }
});

void test("packed checker starts its self-hosted worker chunk", { timeout: 5_000 }, async () => {
  const packed = (await import(new URL("../../../dist/lint/index.mjs", import.meta.url).href)) as {
    createNuxtLintCheckerVitePlugin: typeof CreateVitePlugin;
  };
  const watcher = new EventEmitter() as EventEmitter & {
    add(file: string): void;
  };
  watcher.add = () => {};
  const httpServer = new EventEmitter();
  let resolveOutput = (_output: string): void => {};
  const output = new Promise<string>((resolve) => {
    resolveOutput = resolve;
  });
  const plugin = packed.createNuxtLintCheckerVitePlugin(
    {
      configFile: path.join(here, "fixtures", "unused.config.json"),
      options: {
        cache: true,
        emitError: true,
        emitWarning: true,
        exclude: [],
        fix: false,
        formatter: "json",
        include: [path.join(here, "**/*.vue")],
        lintOnStart: true,
      },
      rootDir: here,
    },
    {
      oxlintEntrypoint: path.join(here, "fixtures", "fake-oxlint.mjs"),
    },
  );
  plugin.configureServer({
    config: { logger: { error: resolveOutput, warn: resolveOutput } },
    httpServer,
    watcher,
    ws: { send() {} },
  } as never);

  try {
    assert.match(await output, /vize\(nuxt\/error\)/u);
  } finally {
    httpServer.emit("close");
  }
});
