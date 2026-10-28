/** A real Vite watcher must route an included-file edit to its owning SFC. */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { setTimeout as pause } from "node:timers/promises";
import { createServer, type HMRPayload } from "vite";
import { vize } from "./plugin/index.ts";

const require = createRequire(import.meta.url);

for (const scenario of [
  { name: "template only", script: "", selfAccepting: false },
  {
    name: "script setup",
    script: "<script setup>const authored = 1;</script>\n",
    selfAccepting: true,
  },
]) {
  await test(
    `Pug include edits reach the owning module through the real Vite watcher (${scenario.name})`,
    { timeout: 30_000 },
    async () => {
      const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pug-watch-"));
      const partial = path.join(root, "partials", "body.pug");
      fs.mkdirSync(path.dirname(partial));
      fs.writeFileSync(partial, "p.first First\n");
      fs.writeFileSync(
        path.join(root, "Card.vue"),
        scenario.script +
          '<template lang="pug">\n.card\n  include partials/body.pug\n</template>\n',
      );
      const server = await createServer({
        root,
        configFile: false,
        logLevel: "silent",
        plugins: [vize({ configMode: false, isProduction: false })],
        resolve: { alias: [{ find: /^vue$/u, replacement: require.resolve("vue") }] },
        optimizeDeps: { noDiscovery: true, include: [] },
        server: { host: "127.0.0.1", port: 0 },
      });
      let timer: ReturnType<typeof setTimeout> | undefined;
      let watcherChanged = false;
      const packets: HMRPayload[] = [];
      const onChange = (file: string) => {
        if (file === partial) watcherChanged = true;
      };
      server.watcher.on("change", onChange);
      try {
        await server.listen();
        const address = server.httpServer?.address();
        assert(address && typeof address === "object");
        const url = `http://127.0.0.1:${address.port}/Card.vue`;
        const first = await fetch(url);
        assert.equal(first.status, 200);
        assert.match(await first.text(), /First/);
        const client = server.environments.client;
        assert(client, "Vite must have a real client environment");
        const owner = path.join(root, "Card.vue");
        const ownerModules = [...(client.moduleGraph.getModulesByFile(owner) ?? [])];
        const ownerUrls = new Set(ownerModules.map((module) => module.url));
        assert(ownerUrls.size > 0, "the fetched SFC must be present in the client graph");
        assert.equal(
          ownerModules.some((module) => module.isSelfAccepting),
          scenario.selfAccepting,
          "the actual client graph must establish this fixture's HMR boundary",
        );
        const previousHmr = new Map(
          ownerModules.map((module) => [module, module.lastHMRTimestamp]),
        );
        const ownerInvalidated = () =>
          ownerModules.some(
            (module) => module.lastHMRTimestamp > (previousHmr.get(module) ?? Infinity),
          );
        const matchesOwner = (payload: HMRPayload) => {
          if (payload.type === "update")
            return payload.updates.some(
              (item) => ownerUrls.has(item.path) || ownerUrls.has(item.acceptedPath),
            );
          return (
            !scenario.selfAccepting &&
            payload.type === "full-reload" &&
            payload.triggeredBy === partial &&
            ownerInvalidated()
          );
        };
        const watchedUntil = Date.now() + 5_000;
        while (
          !server.watcher.getWatched()[path.dirname(partial)]?.includes(path.basename(partial))
        ) {
          assert(Date.now() < watchedUntil, "Vite must watch the actual include before its edit");
          await pause(10);
        }
        const send = client.hot.send.bind(client.hot);
        const update = new Promise<HMRPayload>((resolve, reject) => {
          timer = setTimeout(
            () =>
              reject(
                new Error(
                  `No owning SFC HMR update: ${JSON.stringify({
                    watcherChanged,
                    ownerUrls: [...ownerUrls],
                    packets,
                  })}`,
                ),
              ),
            15_000,
          );
          client.hot.send = (payload: HMRPayload | string, data?: unknown) => {
            if (typeof payload === "string") {
              send(payload, data);
              return;
            }
            send(payload);
            packets.push(payload);
            if (payload.type === "error") reject(new Error(JSON.stringify(payload)));
            if (matchesOwner(payload)) resolve(payload);
          };
        });
        fs.writeFileSync(partial, "p.second Second\n");
        const packet = await update;
        if (scenario.selfAccepting) assert.equal(packet.type, "update");
        assert.equal(ownerInvalidated(), true, "the exact owning graph module must be invalidated");
        assert.equal(
          matchesOwner({ type: "full-reload", path: "*", triggeredBy: partial + ".other" }),
          false,
        );
        assert.equal(matchesOwner({ type: "full-reload", path: "*" }), false);
        assert.equal(
          matchesOwner({
            type: "update",
            updates: [
              {
                type: "js-update",
                path: "/Unrelated.vue",
                acceptedPath: "/Unrelated.vue",
                timestamp: Date.now(),
              },
            ],
          }),
          false,
        );
        assert.equal(watcherChanged, true, "the actual include watcher event must precede HMR");
        const second = await fetch(url);
        assert.equal(second.status, 200);
        const code = await second.text();
        assert.match(code, /Second/);
        assert.doesNotMatch(code, /First/);
      } finally {
        if (timer) clearTimeout(timer);
        server.watcher.off("change", onChange);
        await server.close();
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  );
}
