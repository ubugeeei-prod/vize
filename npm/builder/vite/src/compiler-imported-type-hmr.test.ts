/** Type edits must reach every environment, including after an unrequested undo. */
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { setImmediate as nextTurn, setTimeout as pause } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { createServer } from "vite";
import { vize } from "./plugin/index.ts";

const require = createRequire(import.meta.url);
const sequence = process.env.VIZE_TEST_TYPE_HMR_SEQUENCE;
const sequences = ["none", "unrequested", "requested"];
const count = 3;

async function runSequence(priorEdit: string): Promise<Record<string, number>> {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-type-hmr-")));
  const component = `<script setup lang="ts">
import type { ItemProps } from "./types";
const props = defineProps<ItemProps>();
</script>
<template><div>{{ props.label }}</div></template>`;
  const componentFile = path.join(root, "Item0.vue");
  const typesFile = path.join(root, "types.ts");
  const types = "export interface ItemProps { label: string; }";
  fs.writeFileSync(typesFile, types);
  fs.writeFileSync(
    path.join(root, "tsconfig.json"),
    JSON.stringify({ compilerOptions: { module: "ESNext", moduleResolution: "bundler" } }),
  );
  fs.writeFileSync(
    path.join(root, "app.ts"),
    Array.from(
      { length: count },
      (_, i) => `export { default as Item${i} } from "./Item${i}.vue";`,
    ).join("\n"),
  );
  const urls = Array.from({ length: count }, (_, i) => `/Item${i}.vue`);
  for (const url of urls) fs.writeFileSync(path.join(root, url.slice(1)), component);
  let update:
    | { file: string; pending: Set<string>; watcherChanges: number; resolve(): void }
    | undefined;
  const server = await createServer({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [
      vize({ configMode: false, isProduction: false }),
      {
        name: "observe-type-hmr-completion",
        enforce: "post",
        hotUpdate(options) {
          if (update?.file !== options.file) return;
          update.pending.delete(this.environment.name);
          if (update.pending.size === 0) update.resolve();
        },
      },
    ],
    resolve: {
      alias: [
        { find: /^vue$/u, replacement: require.resolve("vue") },
        { find: /^vue\/server-renderer$/u, replacement: require.resolve("vue/server-renderer") },
      ],
    },
    optimizeDeps: { noDiscovery: true, include: [] },
    server: {
      host: "127.0.0.1",
      port: 0,
      watch: {
        useFsEvents: false,
        // A fast edit/undo can finish both hotUpdate hooks inside Chokidar's
        // fs.watch change throttle. Emit each stable write before the next one.
        awaitWriteFinish: { stabilityThreshold: 100, pollInterval: 10 },
      },
    },
  });
  server.watcher.on("change", (file) => {
    if (update?.file === file) update.watcherChanges++;
  });
  const environments = [server.environments.client, server.environments.ssr];
  const requestAll = async () => {
    const results = [];
    for (const environment of environments) {
      results.push(await Promise.all(urls.map((url) => environment.transformRequest(url))));
    }
    return results;
  };
  const waitUntilWatched = async (file: string) => {
    const deadline = Date.now() + 5_000;
    while (!server.watcher.getWatched()[path.dirname(file)]?.includes(path.basename(file))) {
      assert(Date.now() < deadline, `Vite must watch the file before editing it: ${file}`);
      await pause(10);
    }
  };
  const change = async (file: string, source: string) => {
    await waitUntilWatched(file);
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      const completed = new Promise<void>((resolve, reject) => {
        update = { file, pending: new Set(["client", "ssr"]), watcherChanges: 0, resolve };
        timer = setTimeout(
          () =>
            reject(
              new Error(
                `Hot update did not finish: ${JSON.stringify({
                  file,
                  watcherChanges: update?.watcherChanges,
                  pending: [...(update?.pending ?? [])],
                })}`,
              ),
            ),
          15_000,
        );
      });
      fs.writeFileSync(file, source);
      await completed;
      assert(update && update.watcherChanges > 0, "the real watcher must deliver the edit");
      await nextTurn();
    } finally {
      clearTimeout(timer);
      update = undefined;
    }
  };
  try {
    await server.listen();
    for (const environment of environments) await environment.transformRequest("/app.ts");
    await requestAll();
    if (priorEdit !== "none") {
      await change(
        componentFile,
        component.replace("{{ props.label }}", "{{ props.label }} edited"),
      );
      await requestAll();
      await change(componentFile, component);
      if (priorEdit === "requested") await requestAll();
    }
    await change(typesFile, types.replace("label: string;", "label: string; addedProp?: string;"));
    const results = await requestAll();
    return Object.fromEntries(
      environments.map((environment, i) => [
        environment.name,
        results[i].filter((result) => result?.code.includes("addedProp")).length,
      ]),
    );
  } finally {
    await server.close();
    fs.rmSync(root, { recursive: true, force: true });
  }
}

if (sequence) {
  assert.equal(sequences.includes(sequence), true);
  console.log(JSON.stringify(await runSequence(sequence)));
} else {
  for (const priorEdit of sequences) {
    await test(
      `imported type HMR reaches client and SSR (${priorEdit})`,
      { timeout: 60_000 },
      async () => {
        // Native/compiler caches outlive a Vite server. An earlier sequence in
        // the same process can mask the unrequested-undo regression.
        const { stdout } = await promisify(execFile)(
          process.execPath,
          [fileURLToPath(import.meta.url)],
          {
            env: { ...process.env, VIZE_TEST_TYPE_HMR_SEQUENCE: priorEdit },
          },
        );
        const result = JSON.parse(stdout.trim().split("\n").at(-1)!);
        assert.deepEqual(result, { client: count, ssr: count });
      },
    );
  }
}
