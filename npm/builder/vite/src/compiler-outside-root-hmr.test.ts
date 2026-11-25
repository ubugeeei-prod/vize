/** Outside-root SFCs must be watched even when our load hook supplies their source. */
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { setTimeout as pause } from "node:timers/promises";
import { createServer, type HMRPayload } from "vite";
import { vize } from "./plugin/index.ts";

const require = createRequire(import.meta.url);
const before = '<template><h1 data-state="before">before</h1></template>';
const after = '<template><h1 data-state="after">after</h1></template>';

async function outsideRootHotUpdate(firstRequest: "virtual" | "plain") {
  const workspace = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-outside-hmr-")));
  const root = path.join(workspace, "apps", "host");
  const component = path.join(workspace, "apps", "shared", "src", "Page.vue");
  fs.mkdirSync(path.join(root, "src"), { recursive: true });
  fs.mkdirSync(path.dirname(component), { recursive: true });
  fs.writeFileSync(component, before);
  fs.writeFileSync(
    path.join(root, "src", "main.ts"),
    `export const page = () => import(${JSON.stringify(component.replaceAll("\\", "/"))});`,
  );
  const messages: HMRPayload[] = [];
  let complete: (() => void) | undefined;
  const server = await createServer({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [
      vize({ configMode: false, isProduction: false }),
      {
        name: "observe-outside-root-hmr",
        enforce: "post",
        hotUpdate(options) {
          if (
            this.environment.name === "client" &&
            options.file.replaceAll("\\", "/") === component.replaceAll("\\", "/")
          )
            complete?.();
        },
      },
    ],
    resolve: { alias: [{ find: /^vue$/u, replacement: require.resolve("vue") }] },
    optimizeDeps: { noDiscovery: true, include: [] },
    server: {
      host: "127.0.0.1",
      port: 0,
      fs: { allow: [workspace] },
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
    const origin = `http://127.0.0.1:${address.port}`;
    const url = `/@fs/${component.replaceAll("\\", "/")}${firstRequest === "virtual" ? "?vue&vize" : ""}`;
    const request = async (url: string) => {
      const response = await fetch(`${origin}${url}`);
      assert.equal(response.status, 200);
      return response.text();
    };
    if (firstRequest === "virtual") await request("/src/main.ts");
    assert.match(await request(url), /data-state.*before/u);
    // A source supplied by a plugin bypasses Vite's fallback filesystem load.
    // It still needs a watcher registration when it is outside the root.
    const deadline = Date.now() + 5_000;
    while (!server.watcher.getWatched()[path.dirname(component)]?.includes("Page.vue")) {
      assert(Date.now() < deadline, "the loaded outside-root SFC must be watched");
      await pause(10);
    }
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      const updated = new Promise<void>((resolve, reject) => {
        complete = resolve;
        timeout = setTimeout(() => reject(new Error("the client HMR hook did not finish")), 5_000);
      });
      fs.writeFileSync(component, after);
      await updated;
      // Vite broadcasts after the last hotUpdate hook has returned.
      const messageDeadline = Date.now() + 5_000;
      while (
        !messages.some((message) => message.type === "update" || message.type === "full-reload")
      ) {
        assert(Date.now() < messageDeadline, "an outside-root edit must reach the browser");
        await pause(10);
      }
    } finally {
      clearTimeout(timeout);
      complete = undefined;
    }
    const reloaded = await request(url);
    assert.match(reloaded, /data-state.*after/u);
    assert.doesNotMatch(reloaded, /data-state.*before/u);
  } finally {
    await server.close();
    fs.rmSync(workspace, { recursive: true, force: true });
  }
}

for (const firstRequest of ["virtual", "plain"] as const) {
  test(`outside-root ${firstRequest} requests hot-update and reload fresh code`, async () => {
    await outsideRootHotUpdate(firstRequest);
  });
}
