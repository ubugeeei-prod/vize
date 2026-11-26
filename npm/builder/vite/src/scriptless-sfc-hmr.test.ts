/**
 * An SFC without `<script>` is a component like any other in dev: it accepts
 * its own hot updates, so a template edit rerenders it in place even when no
 * module imports it. Production and SSR bytes stay as they were.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { snapshot, test } from "node:test";
import { setTimeout as pause } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import { createServer, type HMRPayload } from "vite";
import { compileFile } from "./compiler.ts";
import { generateHmrCode } from "./hmr.ts";
import { vize } from "./plugin/index.ts";
import { SCRIPTLESS_OUTPUT_CASES } from "./test/scriptless-sfc.ts";
import { generateOutput } from "./utils/index.ts";

snapshot.setDefaultSnapshotSerializers([(value) => String(value)]);
snapshot.setResolveSnapshotPath(() =>
  fileURLToPath(new URL("./scriptless-sfc-hmr.test.ts.snapshot", import.meta.url)),
);

const require = createRequire(import.meta.url);

// The snapshots were recorded before script-less SFCs became self-accepting.
for (const [name, { compiled, options }] of Object.entries(SCRIPTLESS_OUTPUT_CASES)) {
  void test(`script-less ${name} output is unchanged`, (t) => {
    t.assert.snapshot(generateOutput(compiled, options));
  });
}

const SOURCES = {
  "template-only": `<template><p class="note">{{ msg }}</p></template>`,
  "scoped style": `<template><p class="note">{{ msg }}</p></template>
<style scoped>.note { color: red; }</style>`,
};

void test("a script-less SFC gets the same dev HMR block as a script setup SFC", () => {
  for (const [kind, source] of Object.entries(SOURCES)) {
    for (const script of ["", "<script setup>\n</script>\n"]) {
      const filePath = `/src/${script ? "Setup" : "Scriptless"}.vue`;
      const compiled = compileFile(
        filePath,
        new Map(),
        { root: "/src", sourceMap: false, ssr: false, vapor: false },
        `${script}${source}`,
      );
      const base = { filePath, root: "/src", isProduction: false };
      const withoutHmr = generateOutput(compiled, { ...base, isDev: false });
      for (const updateType of ["template-only", "style-only", "full-reload"] as const) {
        const dev = generateOutput(compiled, { ...base, isDev: true, hmrUpdateType: updateType });
        assert.equal(
          dev,
          `${withoutHmr}${generateHmrCode(compiled.scopeId, updateType)}`,
          `${filePath} (${kind}) must register, accept and ${updateType === "template-only" ? "rerender" : "reload"} itself`,
        );
      }
      if (kind === "scoped style") {
        assert.ok(
          withoutHmr.includes(`__scopeId = "data-v-${compiled.scopeId}"`) ||
            withoutHmr.includes(`__scopeId: "data-v-${compiled.scopeId}"`),
          `${filePath} keeps its scope id for the rerendered tree`,
        );
      }
    }
  }
});

void test("a template edit hot-updates a script-less component nothing imports", async () => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-scriptless-hmr-")));
  const component = path.join(root, "src", "Note.vue");
  fs.mkdirSync(path.dirname(component), { recursive: true });
  fs.writeFileSync(component, '<template><p data-state="before">before</p></template>');
  const messages: HMRPayload[] = [];
  let complete: (() => void) | undefined;
  const server = await createServer({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [
      vize({ configMode: false, isProduction: false }),
      {
        name: "observe-scriptless-hmr",
        enforce: "post",
        hotUpdate(options) {
          if (this.environment.name === "client" && options.file === component) complete?.();
        },
      },
    ],
    resolve: { alias: [{ find: /^vue$/u, replacement: require.resolve("vue") }] },
    optimizeDeps: { noDiscovery: true, include: [] },
    server: {
      host: "127.0.0.1",
      port: 0,
      watch: {
        useFsEvents: false,
        awaitWriteFinish: { stabilityThreshold: 100, pollInterval: 10 },
      },
    },
  });
  const hot = server.environments.client.hot;
  const originalSend = hot.send.bind(hot);
  hot.send = (payload: HMRPayload | string, data?: unknown) => {
    if (typeof payload === "string") originalSend(payload, data);
    else {
      messages.push(payload);
      originalSend(payload);
    }
  };
  try {
    await server.listen();
    const address = server.httpServer?.address();
    assert(address && typeof address !== "string");
    // Like a component a framework registers globally: the browser loaded
    // it, but no module imports it.
    const response = await fetch(`http://127.0.0.1:${address.port}/src/Note.vue`, {
      headers: { connection: "close" },
    });
    assert.equal(response.status, 200);
    assert.match(await response.text(), /import\.meta\.hot\.accept\(/u);
    const deadline = Date.now() + 5_000;
    while (!server.watcher.getWatched()[path.dirname(component)]?.includes("Note.vue")) {
      assert(Date.now() < deadline, "the loaded SFC must be watched");
      await pause(10);
    }
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      const updated = new Promise<void>((resolve, reject) => {
        complete = resolve;
        timeout = setTimeout(() => reject(new Error("the client HMR hook did not finish")), 5_000);
      });
      fs.writeFileSync(component, '<template><p data-state="after">after</p></template>');
      await updated;
      const messageDeadline = Date.now() + 5_000;
      while (
        !messages.some((message) => message.type === "update" || message.type === "full-reload")
      ) {
        assert(Date.now() < messageDeadline, "the edit must reach the browser");
        await pause(10);
      }
    } finally {
      clearTimeout(timeout);
      complete = undefined;
    }
    assert.deepEqual(
      messages.filter((message) => message.type === "full-reload"),
      [],
      "a template edit must not reload the page",
    );
    const update = messages.find((message) => message.type === "update");
    assert.ok(
      update?.type === "update" &&
        update.updates.some(
          (entry) => entry.type === "js-update" && entry.path === entry.acceptedPath,
        ),
      "the component must accept its own update",
    );
  } finally {
    await server.close();
    fs.rmSync(root, { recursive: true, force: true });
  }
});
