import assert from "node:assert/strict";
import { spawn, type ChildProcess } from "node:child_process";
import { once } from "node:events";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  acquireMuseaWorkspaceLock,
  museaWorkspaceLockPath,
  withMuseaWorkspaceLock,
} from "../../tools/benchmarks/scripts/musea-lock.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const measureModule = pathToFileURL(
  path.join(root, "tools", "benchmarks", "scripts", "musea.mjs"),
).href;

async function withTempRoot(fn: (directory: string) => Promise<void>): Promise<void> {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-musea-lock-"));
  try {
    await fn(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

function child(source: string, env: NodeJS.ProcessEnv = {}) {
  const subprocess = spawn(process.execPath, ["--input-type=module", "-e", source], {
    cwd: root,
    env: { ...process.env, ...env },
    stdio: ["ignore", "pipe", "pipe"],
  });
  let stdout = "";
  let stderr = "";
  subprocess.stdout?.on("data", (chunk) => (stdout += chunk));
  subprocess.stderr?.on("data", (chunk) => (stderr += chunk));
  return {
    process: subprocess,
    completed: once(subprocess, "exit").then(([code, signal]) => ({
      code,
      signal,
      stdout,
      stderr,
    })),
  };
}

async function waitForFile(file: string, process: ChildProcess): Promise<void> {
  const deadline = Date.now() + 10_000;
  while (!fs.existsSync(file)) {
    if (process.exitCode != null) throw new Error(`holder exited before creating ${file}`);
    if (Date.now() >= deadline) throw new Error(`timed out waiting for ${file}`);
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}

function writeFixtureBuild(directory: string): void {
  const pluginDist = path.join(directory, "npm", "builder", "vite-musea", "dist");
  const nuxtDist = path.join(directory, "npm", "framework", "musea-nuxt", "dist");
  const nativeDir = path.join(directory, "npm", "native");
  fs.mkdirSync(pluginDist, { recursive: true });
  fs.mkdirSync(nuxtDist, { recursive: true });
  fs.mkdirSync(nativeDir, { recursive: true });

  fs.writeFileSync(
    path.join(pluginDist, "index.mjs"),
    `import fs from "node:fs";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
export function musea() {
  let root = "";
  return [{
    name: "vite-plugin-musea",
    config(userConfig) {
      return { build: { rollupOptions: { input: {
        "musea-static-entry": userConfig.build.rollupOptions.input,
        "musea-static-runtime": "virtual:musea-static-runtime",
      } } } };
    },
    configResolved(config) { root = config.root; },
    options() { return null; },
    async buildStart() {
      require("@vizejs/native");
      const ready = process.env.VIZE_TEST_MUSEA_HOLD_READY;
      const release = process.env.VIZE_TEST_MUSEA_HOLD_RELEASE;
      if (ready && release && !fs.existsSync(ready)) {
        fs.writeFileSync(ready, "ready");
        while (!fs.existsSync(release)) await delay(10);
      }
      fs.readdirSync(root).filter((name) => name.endsWith(".art.vue"));
    },
    resolveId(id) { return id.endsWith(".art.vue") ? "\\0fixture:" + id : null; },
    load(id) { return id.startsWith("\\0") ? "export default " + JSON.stringify(id) : null; },
    transform(code) { return { code }; },
  }];
}
`,
  );
  fs.writeFileSync(
    path.join(nuxtDist, "index.mjs"),
    `export function nuxtMusea() {
  return {
    name: "fixture-nuxt",
    resolveId(id) { return "\\0nuxt:" + id; },
    load(id) { return "export default " + JSON.stringify(id); },
  };
}
`,
  );
  fs.writeFileSync(path.join(nativeDir, "vize-vitrine.fixture.node"), "fixture-native");
  fs.writeFileSync(
    path.join(nativeDir, "index.js"),
    'module.exports = require("./native-binding");\n',
  );
  fs.writeFileSync(
    path.join(nativeDir, "native-binding.js"),
    `const path = require("node:path");
const bindingPath = path.join(__dirname, "vize-vitrine.fixture.node");
require.cache[bindingPath] = { id: bindingPath, filename: bindingPath, loaded: true, exports: {} };
module.exports = {};
`,
  );
  fs.writeFileSync(
    path.join(nativeDir, "native-targets.js"),
    'module.exports = { nativeTargets: () => ["fixture"] };\n',
  );
  fs.writeFileSync(path.join(nativeDir, "package.json"), '{"name":"@vizejs/native"}\n');
}

const measureSource = `
import { measureMusea } from ${JSON.stringify(measureModule)};
try {
  await measureMusea({ root: process.env.VIZE_TEST_ROOT, files: 2, runs: 1, warmups: 0 });
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
`;

test("a live owner is rejected and release runs after success or failure", async () => {
  await withTempRoot(async (directory) => {
    const release = acquireMuseaWorkspaceLock(directory);
    assert.throws(
      () => acquireMuseaWorkspaceLock(directory),
      new RegExp(`already held by live pid ${process.pid}`),
    );
    release();
    assert.equal(fs.existsSync(museaWorkspaceLockPath(directory)), false);

    await assert.rejects(
      () =>
        withMuseaWorkspaceLock(directory, async () => {
          throw new Error("fixture failure");
        }),
      /fixture failure/,
    );
    assert.equal(fs.existsSync(museaWorkspaceLockPath(directory)), false);
  });
});

test("a lock is reclaimed only after its owner process is dead", async () => {
  await withTempRoot(async (directory) => {
    const exited = spawn(process.execPath, ["-e", ""], { stdio: "ignore" });
    const deadPid = exited.pid;
    await once(exited, "exit");
    assert.ok(deadPid != null);

    const lockPath = museaWorkspaceLockPath(directory);
    fs.mkdirSync(lockPath, { recursive: true });
    fs.writeFileSync(
      path.join(lockPath, "owner.json"),
      `${JSON.stringify({ pid: deadPid, token: "stale", acquiredAt: "2000-01-01T00:00:00.000Z" })}\n`,
    );

    const release = acquireMuseaWorkspaceLock(directory);
    const owner = JSON.parse(fs.readFileSync(path.join(lockPath, "owner.json"), "utf8"));
    assert.equal(owner.pid, process.pid);
    assert.notEqual(owner.token, "stale");
    release();
    assert.equal(fs.existsSync(lockPath), false);
    assert.equal(fs.existsSync(`${lockPath}.reclaimed-stale`), true);
  });
});

test("a second process cannot delete or perturb a running lane's pinned artifacts", async () => {
  await withTempRoot(async (directory) => {
    writeFixtureBuild(directory);
    const ready = path.join(directory, "holder-ready");
    const allowRelease = path.join(directory, "holder-release");
    const holder = child(measureSource, {
      VIZE_TEST_ROOT: directory,
      VIZE_TEST_MUSEA_HOLD_READY: ready,
      VIZE_TEST_MUSEA_HOLD_RELEASE: allowRelease,
    });

    try {
      await waitForFile(ready, holder.process);
      const pinRoot = path.join(
        directory,
        "npm",
        "builder",
        "vite-musea",
        "node_modules",
        ".cache",
        "vize-musea-benchmark",
      );
      const measuredEntry = path.join(
        pinRoot,
        fs.readdirSync(pinRoot)[0],
        "package",
        "dist",
        "index.mjs",
      );
      const measuredBytes = fs.readFileSync(measuredEntry, "utf8");

      const contender = child(measureSource, { VIZE_TEST_ROOT: directory });
      const blocked = await contender.completed;
      assert.equal(blocked.code, 1);
      assert.match(blocked.stderr, /already held by live pid/);
      assert.equal(fs.readFileSync(measuredEntry, "utf8"), measuredBytes);
      assert.equal(fs.existsSync(museaWorkspaceLockPath(directory)), true);
    } finally {
      fs.writeFileSync(allowRelease, "release");
    }

    const completed = await holder.completed;
    assert.equal(completed.code, 0, completed.stderr);
    assert.equal(fs.existsSync(museaWorkspaceLockPath(directory)), false);
  });
});
