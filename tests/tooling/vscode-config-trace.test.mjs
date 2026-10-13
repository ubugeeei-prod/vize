import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { stripTypeScriptTypes } from "node:module";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const source = fs.readFileSync(path.join(root, "editors/vscode/src/workspace-config.ts"), "utf8");
const body = stripTypeScriptTypes(source, { mode: "transform" })
  .replace(/^import[\s\S]*?;\n/gm, "")
  .replace("export class WorkspaceLspConfigDiscovery", "class WorkspaceLspConfigDiscovery");

function createDiscovery(findFiles = async () => []) {
  const operations = [];
  const trace = [];
  const callbacks = {};
  const disposable = (name) => ({ dispose: () => operations.push(["dispose", name]) });
  const watcher = {
    dispose: () => operations.push(["dispose", "watcher"]),
    onDidCreate: (callback) => {
      callbacks.create = callback;
      return disposable("create");
    },
    onDidChange: (callback) => {
      callbacks.change = callback;
      return disposable("change");
    },
    onDidDelete: (callback) => {
      callbacks.delete = callback;
      return disposable("delete");
    },
  };
  const workspace = {
    workspaceFolders: [],
    createFileSystemWatcher: (glob) => {
      operations.push(["watch", glob]);
      return watcher;
    },
    onDidChangeWorkspaceFolders: (callback) => {
      callbacks.folders = callback;
      return disposable("folders");
    },
    findFiles: (...arguments_) => {
      operations.push(["findFiles", ...arguments_]);
      return findFiles();
    },
  };
  const context = vm.createContext({
    fs,
    path,
    workspace,
    WORKSPACE_PROJECT_CONFIG_FILES: [],
    WORKSPACE_PROJECT_CONFIG_GLOB: "original-project-config-glob",
    describeCapabilities: () => "original description",
    trace,
  });
  const discovery = new vm.Script(
    `${body}\nnew WorkspaceLspConfigDiscovery([], message => trace.push(message));`,
  ).runInContext(context);
  return { discovery, operations, callbacks, trace };
}

await test("actual watcher traces original accepted/rejected create/change/delete callbacks and preserves all subscriptions", () => {
  const { discovery, callbacks, operations, trace } = createDiscovery();
  const reasons = [];
  const subscriptions = discovery.watch((reason) => reasons.push(reason));
  for (const kind of ["create", "change", "delete"]) {
    for (const pathname of [
      "/workspace/packages/app/tsconfig.app.json",
      "/workspace/node_modules/tsconfig.json",
      "/workspace/target/tsconfig.json",
      "/workspace/dist/tsconfig.json",
      "/workspace/build/tsconfig.json",
      "/workspace/.git/tsconfig.json",
    ]) {
      callbacks[kind]({ path: pathname, toString: () => `file://${pathname}` });
    }
  }
  callbacks.folders();
  assert.deepEqual(reasons, [
    "project configuration changed",
    "project configuration changed",
    "project configuration changed",
    "workspace changed",
  ]);
  assert.equal(trace.length, 19);
  for (const kind of ["create", "change", "delete"]) {
    assert.ok(
      trace.includes(
        `watch callback kind=${kind} accepted=true uri=file:///workspace/packages/app/tsconfig.app.json`,
      ),
    );
    assert.ok(
      trace.includes(
        `watch callback kind=${kind} accepted=false uri=file:///workspace/node_modules/tsconfig.json`,
      ),
    );
  }
  for (const subscription of subscriptions) subscription.dispose();
  assert.deepEqual(operations, [
    ["watch", "original-project-config-glob"],
    ["dispose", "watcher"],
    ["dispose", "create"],
    ["dispose", "change"],
    ["dispose", "delete"],
    ["dispose", "folders"],
  ]);
});

await test("actual findFiles keeps original arguments, commits only completed result, and rethrows identical rejection", async () => {
  let resolve;
  const pending = new Promise((complete) => {
    resolve = complete;
  });
  const { discovery, operations, trace } = createDiscovery(() => pending);
  const refresh = discovery.refresh(7);
  assert.equal(discovery.hasConfig, false);
  assert.deepEqual(trace, ["findFiles enter generation=7"]);
  resolve([{ path: "actual found config" }]);
  await refresh;
  assert.equal(discovery.hasConfig, true);
  assert.deepEqual(operations, [
    ["findFiles", "original-project-config-glob", "**/{node_modules,.git,target,dist,build}/**", 1],
  ]);
  assert.deepEqual(trace, [
    "findFiles enter generation=7",
    "findFiles return generation=7 discovered=true",
  ]);
  const originalError = new Error("original search refusal");
  const refusal = createDiscovery(async () => {
    throw originalError;
  });
  await assert.rejects(refusal.discovery.refresh(8), (error) => error === originalError);
  assert.equal(refusal.discovery.hasConfig, false);
  assert.deepEqual(refusal.trace, ["findFiles enter generation=8", "findFiles error generation=8"]);
});

for (const [olderResult, latestResult] of [
  [[{ path: "deleted nested config" }], []],
  [[], [{ path: "created nested config" }]],
]) {
  await test(`out-of-order nested discovery retains the latest ${latestResult.length ? "creation" : "deletion"}`, async () => {
    const searches = [];
    const { discovery, trace } = createDiscovery(
      () => new Promise((resolve) => searches.push(resolve)),
    );
    const older = discovery.refresh();
    const latest = discovery.refresh();
    searches[1](latestResult);
    await latest;
    assert.equal(discovery.hasConfig, latestResult.length > 0);
    searches[0](olderResult);
    await older;
    assert.equal(discovery.hasConfig, latestResult.length > 0);
    assert.equal(trace.at(-1), "findFiles refused generation=undefined stage=superseded");
  });
}
