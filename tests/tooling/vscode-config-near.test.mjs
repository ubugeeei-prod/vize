import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { createDiscovery } from "./support/vscode-workspace-config.mjs";

const configs = ["vite.config.ts", "tsconfig.json", "jsconfig.json"];

function project(t, files, links = {}) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-near-config-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  for (const file of files) {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(path.join(root, file), "{}\n");
  }
  for (const [link, target] of Object.entries(links)) {
    fs.mkdirSync(path.dirname(path.join(root, link)), { recursive: true });
    fs.symlinkSync(path.join(root, target), path.join(root, link), "junction");
  }
  return root;
}

function near(root, filesExclude, findFiles) {
  return createDiscovery(findFiles, [root], { projectConfigFiles: configs, filesExclude });
}

await test("a config two directory levels down decides startup until the complete search returns", async (t) => {
  const root = project(t, ["packages/app/tsconfig.json", "packages/app/src/App.vue"]);
  let results = [];
  const { discovery, operations } = near(root, undefined, async () => results);
  assert.equal(discovery.hasRootConfig, false);
  assert.equal(await discovery.findNearConfig(), true);
  assert.equal(discovery.hasConfig, true);
  assert.deepEqual(
    operations.filter(([kind]) => kind === "findFiles"),
    [],
    "the nearby listing never starts a workspace search",
  );
  results = [];
  await discovery.refresh(1);
  assert.equal(discovery.hasConfig, false, "the complete search stays authoritative");
  assert.equal(await discovery.findNearConfig(), false, "a completed search is not reopened");
});

await test("tsconfig variants count; deeper, dot, excluded and linked directories do not", async (t) => {
  assert.equal(
    await near(project(t, ["apps/web/tsconfig.app.json"])).discovery.findNearConfig(),
    true,
  );
  const root = project(
    t,
    [
      "a/b/c/tsconfig.json",
      ".config/tsconfig.json",
      "node_modules/pkg/tsconfig.json",
      "dist/tsconfig.json",
      "outside/deep/er/vite.config.ts",
    ],
    { "linked/app": "outside/deep/er" },
  );
  assert.equal(await near(root).discovery.findNearConfig(), false);
});

await test("files.exclude basename keys hide entries; any other key keeps the complete search", async (t) => {
  const root = project(t, ["packages/app/tsconfig.json"]);
  assert.equal(
    await near(root, { "**/.DS_Store": true, "**/PACKAGES": true }).discovery.findNearConfig(),
    false,
  );
  assert.equal(
    await near(root, { "**/.DS_Store": true, "**/packages": false }).discovery.findNearConfig(),
    true,
  );
  for (const exclude of [
    { "packages/**": true },
    { "**/*.json": true },
    { "**/app": { when: "$(basename).ts" } },
  ]) {
    assert.equal(await near(root, exclude).discovery.findNearConfig(), false);
  }
});

await test("a refresh that starts during the listing owns the decision", async (t) => {
  const root = project(t, ["packages/app/tsconfig.json"]);
  const { discovery } = near(root, undefined, async () => []);
  const pending = discovery.findNearConfig();
  const refreshed = discovery.refresh(2);
  assert.equal(await pending, false);
  assert.equal(discovery.hasConfig, false);
  await refreshed;
});
