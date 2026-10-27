/** A real Vite watcher must route an included-file edit to its owning SFC. */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { createServer, type HMRPayload } from "vite";
import { vize } from "./plugin/index.ts";

const require = createRequire(import.meta.url);

await test(
  "Pug include edits reach the owning module through the real Vite watcher",
  { timeout: 30_000 },
  async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pug-watch-"));
    const partial = path.join(root, "partials", "body.pug");
    fs.mkdirSync(path.dirname(partial));
    fs.writeFileSync(partial, "p.first First\n");
    fs.writeFileSync(
      path.join(root, "Card.vue"),
      '<template lang="pug">\n.card\n  include partials/body.pug\n</template>\n',
    );
    const server = await createServer({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [vize({ configMode: false })],
      resolve: { alias: [{ find: /^vue$/u, replacement: require.resolve("vue") }] },
      optimizeDeps: { noDiscovery: true, include: [] },
      server: { host: "127.0.0.1", port: 0 },
    });
    let timer: ReturnType<typeof setTimeout> | undefined;
    try {
      await server.listen();
      const address = server.httpServer?.address();
      assert(address && typeof address === "object");
      const url = `http://127.0.0.1:${address.port}/Card.vue`;
      const first = await fetch(url);
      assert.equal(first.status, 200);
      assert.match(await first.text(), /First/);
      const send = server.ws.send.bind(server.ws);
      const update = new Promise<HMRPayload>((resolve, reject) => {
        timer = setTimeout(
          () => reject(new Error("No Card.vue HMR update after include edit")),
          15_000,
        );
        server.ws.send = (payload: HMRPayload | string, data?: unknown) => {
          if (typeof payload === "string") {
            send(payload, data);
            return;
          }
          send(payload);
          if (payload.type === "error") reject(new Error(JSON.stringify(payload)));
          if (
            payload.type === "update" &&
            payload.updates.some(
              (item) => item.path === "/Card.vue" || item.acceptedPath === "/Card.vue",
            )
          )
            resolve(payload);
        };
      });
      fs.writeFileSync(partial, "p.second Second\n");
      const packet = await update;
      assert.equal(packet.type, "update");
      const second = await fetch(url);
      assert.equal(second.status, 200);
      const code = await second.text();
      assert.match(code, /Second/);
      assert.doesNotMatch(code, /First/);
    } finally {
      if (timer) clearTimeout(timer);
      await server.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);
