import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { createSync, deferred } from "./support/vscode-config-sync.ts";
import { createDiscovery } from "./support/vscode-workspace-config.mjs";

await test("a root project config starts the client before the workspace-wide search returns", async () => {
  const search = deferred();
  let refreshes = 0;
  const current = createSync(
    () => {
      refreshes++;
      return search.promise;
    },
    undefined,
    false,
    undefined,
    { hasRootConfig: true },
  );
  await current.control.syncClientToConfiguration({}, "initial activation");
  assert.equal(refreshes, 1);
  assert.deepEqual(
    current.operations.filter(([kind]) => kind === "start").map((row) => [...row]),
    [["start", 1]],
  );
  search.resolve();
  await search.promise;
  assert.equal(current.control.state().generation, 1);
});

await test("a background search failure after root-config startup is reported, not thrown", async () => {
  const failure = new Error("search unavailable");
  const current = createSync(
    async () => {
      throw failure;
    },
    undefined,
    false,
    undefined,
    { hasRootConfig: true },
  );
  await current.control.syncClientToConfiguration({}, "initial activation");
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(
    current.operations.filter(([kind]) => kind === "start").map((row) => [...row]),
    [["start", 1]],
  );
  assert.ok(
    current.logs.includes("Project configuration discovery failed: Error: search unavailable"),
  );
});

await test("root configs are checked once per refresh", async (t) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-root-config-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.writeFileSync(path.join(root, "tsconfig.json"), "{}\n");
  const { discovery } = createDiscovery(async () => [], [root], {
    projectConfigFiles: ["vite.config.ts", "tsconfig.json"],
  });
  assert.equal(discovery.hasRootConfig, true);
  fs.rmSync(path.join(root, "tsconfig.json"));
  assert.equal(discovery.hasConfig, true, "one refresh observes one root state");
  await discovery.refresh(1);
  assert.equal(discovery.hasRootConfig, false);
  assert.equal(discovery.hasConfig, false);
});
