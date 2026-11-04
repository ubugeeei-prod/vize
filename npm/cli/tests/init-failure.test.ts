import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { test } from "vite-plus/test";

import { detectProject } from "../src/init/detect.ts";
import { initProject } from "../src/init.ts";
import { manifest, nonTtyStdin, temporaryProject, write, writeManifest } from "./init-support.ts";

const VITE_CONFIG = `import { defineConfig } from "vite";

export default defineConfig({
  plugins: [],
});
`;

test("a package root is still detected", () => {
  const project = temporaryProject("failure-root");
  writeManifest(project, manifest());
  write(project, "vite.config.ts", VITE_CONFIG);
  assert.equal(detectProject(project).framework, "vite");
});

test("init in a monorepo package subdirectory names that package and writes nothing", async () => {
  const workspace = temporaryProject("failure-subdir");
  writeManifest(workspace, manifest({ name: "workspace-root" }));
  const app = path.join(workspace, "packages", "app");
  writeManifest(app, manifest({ name: "app" }));
  const nested = path.join(app, "src");
  fs.mkdirSync(nested, { recursive: true });

  await assert.rejects(
    () => initProject({ root: nested, args: ["--yes"], stdin: nonTtyStdin() }),
    (error: unknown) => {
      assert.ok(error instanceof Error);
      assert.match(error.message, /Run `vize init` from/u);
      assert.ok(error.message.includes(`from ${app} instead`), error.message);
      assert.match(error.message, /Type checking was not configured/u);
      return true;
    },
  );
  assert.deepEqual(fs.readdirSync(nested), []);
});

test("an offline install names vize and does not run type checking", async () => {
  const project = temporaryProject("failure-offline");
  fs.mkdirSync(path.join(project, ".git"));
  writeManifest(project, manifest({ name: "offline-cache" }));
  write(project, "vite.config.ts", VITE_CONFIG);
  const cache = fs.mkdtempSync(path.join(os.tmpdir(), "vize-init-empty-cache-"));
  const previous = {
    npm_config_audit: process.env.npm_config_audit,
    npm_config_cache: process.env.npm_config_cache,
    npm_config_fund: process.env.npm_config_fund,
    npm_config_offline: process.env.npm_config_offline,
    npm_config_update_notifier: process.env.npm_config_update_notifier,
  };
  process.env.npm_config_audit = "false";
  process.env.npm_config_cache = cache;
  process.env.npm_config_fund = "false";
  process.env.npm_config_offline = "true";
  process.env.npm_config_update_notifier = "false";
  try {
    await assert.rejects(
      () =>
        initProject({
          root: project,
          args: ["--yes", "--no-lint", "--vite", "--no-fmt", "--no-editor"],
          stdin: nonTtyStdin(),
        }),
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.match(error.message, /Type checking was not run/u);
        assert.match(error.message, /Install command failed \(npm install -D .*?\bvize\b/u);
        assert.match(error.message, /ENOTCACHED|only-if-cached/u);
        return true;
      },
    );
  } finally {
    for (const [name, value] of Object.entries(previous)) {
      if (value === undefined) delete process.env[name];
      else process.env[name] = value;
    }
    fs.rmSync(cache, { force: true, recursive: true });
  }
});
